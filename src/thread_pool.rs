// TODO -> shutdown semantics: stop accepting new jobs, decide whether queued jobs are
// drained or canceled, and wait for active workers to finish up to some policy.

use tokio::sync::mpsc;

use crate::message::SubmitJob;

#[derive(Debug)]
pub(crate) struct QueuedJob {
    job: SubmitJob,
    // TODO -> send back channel
}

impl QueuedJob {
    pub(crate) fn from_submit_job(job: SubmitJob) -> Self {
        QueuedJob {
            // TODO -> initialize channel
            job,
        }
    }
}

pub(crate) struct ThreadPool {
    //
}

impl ThreadPool {
    pub(crate) async fn run(mut receiver: mpsc::Receiver<QueuedJob>) -> Result<(), WorkerPoolErr> {
        while let Some(job) = receiver.recv().await {
            dbg!(&job);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum WorkerPoolErr {
    /// Invalid message encoding
    Other(crate::Error),
}
