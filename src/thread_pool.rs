use crate::message::SubmitJob;
use tokio::sync::{mpsc, oneshot};

// TODO -> shutdown semantics: stop accepting new jobs, decide whether queued jobs are
// drained or canceled, and wait for active workers to finish up to some policy.

static WORKER_COUNT: usize = 4;

#[derive(Debug)]
pub(crate) struct JobResult {
    foo: String,
}

#[derive(Debug)]
pub(crate) struct QueuedJob {
    job: SubmitJob,
    reply_tx: oneshot::Sender<JobResult>,
}

impl QueuedJob {
    pub(crate) fn from_submit_job(job: SubmitJob, reply_tx: oneshot::Sender<JobResult>) -> Self {
        QueuedJob { job, reply_tx }
    }
}

/// A fixed set of long-lived OS threads created once at startup.
pub(crate) struct ThreadPool {
    receiver: mpsc::Receiver<QueuedJob>,
    workers: Vec<std::thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub(crate) async fn start(receiver: mpsc::Receiver<QueuedJob>) -> Result<(), WorkerPoolErr> {
        let mut pool = ThreadPool::new(receiver)?;

        while let Some(job) = pool.receiver.recv().await {
            // tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            // let _ = job.reply_tx.send(JobResult { foo: "okay".into() });
        }

        Ok(())
    }

    /*
        TODO

        One nuance: the part you pass to workers must be shared thread-safe state,
        usually behind Arc and synchronization primitives. The handles themselves
        are not shared with workers; they stay owned by the pool object.
        The workers just run. The pool keeps the handles as the management
        side of the lifecycle.
    */

    fn new(receiver: mpsc::Receiver<QueuedJob>) -> Result<Self, WorkerPoolErr> {
        let mut workers = Vec::with_capacity(WORKER_COUNT);

        for i in 0..WORKER_COUNT {
            // std::thread::spawn(...)
        }

        todo!()
    }
}

#[derive(Debug)]
pub enum WorkerPoolErr {
    ThreadPoolStartError,

    /// Invalid message encoding
    Other(crate::Error),
}
