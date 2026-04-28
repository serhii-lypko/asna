use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, broadcast, mpsc, oneshot};
use tokio::time::Duration;

use crate::connection::Connection;
use crate::thread_pool::{QueuedJob, ThreadPool};

// TODO -> normally both that should be configurable and adjastable
const MAX_CONNECTIONS: usize = 64;
const BOUNDED_QUEUE_LIMIT: usize = 64;

/// Orchestration boundary for the server lifecycle.
pub async fn run(tcp_listener: TcpListener, shutdown: impl Future) {
    // Both channels are used for lifecycle signals, not data queue.
    // It's a lifecycle orchestration across a graph of long-lived concurrent components.
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let (bounded_queue_sender, bounded_queue_receiver) = mpsc::channel(BOUNDED_QUEUE_LIMIT);

    // TODO -> should also handle gracefull shutdown: stop workers, etc.
    // Spawning thread pool
    if let Err(err) = ThreadPool::boot(bounded_queue_receiver).await {
        unimplemented!()
    }

    let connection_limit = Arc::new(Semaphore::new(MAX_CONNECTIONS));

    let mut listener = Listener {
        tcp_listener,
        notify_shutdown,
        shutdown_complete_tx,
        connection_limit,
        bounded_queue_sender,
    };

    // NOTE: receiving external shutdown signal won't automatically drop spawned connections.
    // What it will do - is stop accepting new connections.
    //
    // Since once a handler has been spawned with tokio::spawn, it is a separate task owned by the runtime.
    // It no longer lives inside listener.run()’s future state.
    //
    // Therefore need to attach shutdown communication to the handlers so the system would
    // gracefully notify them about system halt and receive completion signals.
    // -> propagate shutdown through your task graph.
    tokio::select! {
        // Listener - is an open-ended future.
        _res = listener.run() => {
            //
        }
        _ = shutdown => {
            println!("\nshutdown");
        }
    }

    let Listener {
        notify_shutdown,
        shutdown_complete_tx,
        ..
    } = listener;

    // When `notify_shutdown` is dropped, all tasks which have `subscribe`d will
    // receive the shutdown signal and can exit.
    drop(notify_shutdown);

    // Drop final `Sender` so the `Receiver` below can complete.
    drop(shutdown_complete_tx);

    // Waiting for all shutdown completion.
    let _ = shutdown_complete_rx.recv().await;
}

struct Listener {
    tcp_listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    connection_limit: Arc<Semaphore>,

    bounded_queue_sender: mpsc::Sender<QueuedJob>,
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        loop {
            // Acquire before accepting so we stop taking new sockets once the
            // server is already at its active-connection limit.
            let connection_permit = self.connection_limit.clone().acquire_owned().await?;
            let socket = self.accept().await?;

            // Per-connection handler state.
            let mut conn_handler = ConnectionHandler {
                connection: Connection::new(socket),

                // Create shutdown receiver for each connection.
                shutdown_signal: Shutdown::new(self.notify_shutdown.subscribe()),

                _notify_shutdown: self.shutdown_complete_tx.clone(),

                bounded_queue_sender: self.bounded_queue_sender.clone(),
            };

            tokio::spawn(async move {
                if let Err(_) = conn_handler.run().await {
                    //
                }

                // Move the permit into the task and drop it after completion.
                // This returns the permit back to the semaphore.
                drop(connection_permit);
            });
        }
    }

    async fn accept(&self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.tcp_listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

struct ConnectionHandler {
    connection: Connection,
    shutdown_signal: Shutdown,
    _notify_shutdown: mpsc::Sender<()>,

    bounded_queue_sender: mpsc::Sender<QueuedJob>,
}

impl ConnectionHandler {
    async fn run(&mut self) -> crate::Result<()> {
        println!("run connection handler");

        // FIXME -> normally should track shutdown: while !self.shutdown_signal.is_shutdown()
        loop {
            let maybe_message = tokio::select! {
                res = self.connection.read_message() => res?,
                _ = self.shutdown_signal.recv() => {
                    // TODO -> logging
                    return Ok(());
                }
            };

            // If `None` is returned from `read()` then the peer closed
            // the socket. There is no further work to do and the task can be
            // terminated.
            let message = match maybe_message {
                Some(msg) => msg,
                None => {
                    return Ok(());
                }
            };

            let (job_result_sender, job_result_receiver) = oneshot::channel();

            match message {
                crate::message::Message::Ping => {}
                crate::message::Message::Pong => {}
                crate::message::Message::SubmitJob(submit_job) => {
                    self.bounded_queue_sender
                        .send(QueuedJob::from_submit_job(submit_job, job_result_sender))
                        .await?;
                }
                crate::message::Message::ResultJob(result_job) => unimplemented!(),
            };

            let job_result = job_result_receiver.await?;

            // TODO -> write back JobResult to the connection
        }
    }
}

struct Shutdown {
    shutdown_signal: broadcast::Receiver<()>,
    is_shutdown: bool,
}

impl Shutdown {
    fn new(shutdown_signal: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown_signal,
            is_shutdown: false,
        }
    }

    fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }

    async fn recv(&mut self) {
        let _ = self.shutdown_signal.recv().await;
        self.is_shutdown = true;
    }
}
