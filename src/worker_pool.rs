use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle as TokioJoinHandle;

use crate::message::{ResultJob, SubmitJob};

// TODO -> shutdown semantics: stop accepting new jobs, decide whether queued jobs are
// drained or canceled, and wait for active workers to finish up to some policy.

static WORKER_COUNT: usize = 4;
// static WORKER_COUNT: usize = 1;

#[derive(Debug)]
pub(crate) struct QueuedJob {
    job: SubmitJob,
    reply_tx: oneshot::Sender<ResultJob>,
}

impl QueuedJob {
    pub(crate) fn from_submit_job(job: SubmitJob, reply_tx: oneshot::Sender<ResultJob>) -> Self {
        QueuedJob { job, reply_tx }
    }
}

struct SharedState {
    queue: Mutex<VecDeque<QueuedJob>>,
    is_shutdown: Mutex<bool>,

    // The condvar lets threads sleep until someone will trigger condition re-check.
    // Notifications are hints to wake up and inspect the condition again. It is like one-shot a wakeup event.
    job_available: Condvar,
}

impl SharedState {
    fn begin_shutdown(&self) {
        let mut is_shutdown = self.is_shutdown.lock().unwrap();
        *is_shutdown = true;
        drop(is_shutdown);

        // Wake all sleeping workers so they can re-check the shutdown flag and exit
        // instead of remaining blocked on the condvar forever.
        self.job_available.notify_all();
    }
}

/// A fixed set of long-lived OS threads created once at startup.
pub(crate) struct WorkerPool {
    // Having ownerhsip and lifecycle controll over spawned threads via JoinHandle-s.
    workers: Vec<std::thread::JoinHandle<()>>,
    state: Arc<SharedState>,
    bridge_task: Option<TokioJoinHandle<()>>,
}

impl WorkerPool {
    fn new() -> Result<Self, WorkerPoolErr> {
        let mut workers = Vec::with_capacity(WORKER_COUNT);

        let state = Arc::new(SharedState {
            queue: Mutex::new(VecDeque::new()),
            is_shutdown: Mutex::new(false),
            job_available: Condvar::new(),
        });

        // After boot spawns this OS thread, it immediately enters a long-lived pull loop.
        // The worker blocks on the shared queue until work is available, then grabs the next
        // queued job itself, releases the lock, and processes that job outside the critical section.
        //
        // Create an OS thread now, and it give that new thread a function to run later or immediately,
        // depending on scheduling.
        //
        // Cost of idle worker threads is mostly memory and some scheduler bookkeeping, but no CPU time.
        for _ in 0..WORKER_COUNT {
            // TODO -> use thread builder?
            // https://mara.nl/atomics/basics.html#thread-builder
            // The main practical reason is naming. Giving workers names like asna-worker-0, asna-worker-1,
            // and so on is useful for logs, panics, debugging, and profilers. Once have multiple long-lived
            // worker threads, names become worth it immediately.

            // Design reference: https://mara.nl/atomics/basics.html#condvar
            let task = std::thread::spawn({
                let state = state.clone();

                move || loop {
                    let mut queue = state.queue.lock().unwrap();

                    let qjob = loop {
                        if let Some(job) = queue.pop_front() {
                            break job;
                        }

                        if *state.is_shutdown.lock().unwrap() {
                            return;
                        }

                        queue = state.job_available.wait(queue).unwrap();
                    };

                    drop(queue);

                    let job_result = qjob.job.eval();
                    let _ = qjob.reply_tx.send(job_result);

                    dbg!("processed job");
                }
            });

            workers.push(task);
        }

        Ok(WorkerPool {
            workers,
            state,
            bridge_task: None,
        })
    }

    /// boot itself - is a short-lived routine, which runs to completion and initiates background work
    pub(crate) async fn boot(
        mut shutdown_rx: broadcast::Receiver<()>,
        mut receiver: mpsc::Receiver<QueuedJob>,
    ) -> Result<WorkerPool, WorkerPoolErr> {
        let mut pool = WorkerPool::new()?;

        // Bridge task
        pool.bridge_task = Some(tokio::spawn({
            let state = pool.state.clone();

            async move {
                // Keep forwarding jobs until shutdown
                loop {
                    tokio::select! {
                        maybe_job = receiver.recv() => {
                            match maybe_job {
                                Some(job) => {
                                    state.queue.lock().unwrap().push_back(job);
                                    state.job_available.notify_one();
                                }
                                None => {
                                    state.begin_shutdown();
                                    break;
                                }
                            }
                        }
                        _ = shutdown_rx.recv() => {
                            state.begin_shutdown();
                            break;
                        }
                    }
                }
            }
        }));

        Ok(pool)
    }

    pub(crate) async fn shutdown(self) {
        let WorkerPool {
            workers,
            state,
            mut bridge_task,
        } = self;

        if let Some(bridge_task) = bridge_task.take() {
            let _ = bridge_task.await;
        }
        state.begin_shutdown();

        let _ = tokio::task::spawn_blocking(move || {
            for worker in workers {
                let _ = worker.join();
            }
        })
        .await;
    }
}

#[derive(Debug)]
pub enum WorkerPoolErr {
    WorkerPoolStartError,

    /// Invalid message encoding
    Other(crate::Error),
}
