use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, broadcast, mpsc};
use tokio::time::Duration;

use crate::SubmitJob;
use crate::connection::Connection;

/// Orchestration boundary for the server lifecycle.
pub async fn run(tcp_listener: TcpListener, shutdown: impl Future) {
    // Both channels are used for lifecycle signals, not data queue.
    // It's a lifecycle orchestration across a graph of long-lived concurrent components.
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    // TODO -> connection permit

    let mut listener = Listener {
        tcp_listener,
        notify_shutdown,
        shutdown_complete_tx,
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
        res = listener.run() => {
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
    //
    // TODO -> connection permit
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        loop {
            let socket = self.accept().await?;

            // Per-connection handler state.
            let mut conn_handler = ConnectionHandler {
                connection: Connection::new(socket),

                // Create shutdown receiver for each connection.
                shutdown_signal: Shutdown::new(self.notify_shutdown.subscribe()),

                //
                _notify_shutdown: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                if let Err(_) = conn_handler.run().await {
                    //
                }

                // TODO -> handle (drop) permit
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
}

impl ConnectionHandler {
    async fn run(&mut self) -> crate::Result<()> {
        println!("run connection handler");

        while !self.shutdown_signal.is_shutdown() {
            let maybe_message = tokio::select! {
                res = self.connection.read_message() => res?,
                _ = self.shutdown_signal.recv() => {
                    dbg!("receive shutdown 🟡");
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

            //
        }

        Ok(())
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
    }
}
