// TODO -> shutdown semantics: stop accepting new jobs, decide whether queued jobs are
// drained or canceled, and wait for active workers to finish up to some policy.

use tokio::sync::{mpsc, oneshot};

use crate::message::SubmitJob;

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

pub(crate) struct ThreadPool {
    //
}

impl ThreadPool {
    pub(crate) async fn run(mut receiver: mpsc::Receiver<QueuedJob>) -> Result<(), WorkerPoolErr> {
        while let Some(job) = receiver.recv().await {
            // dbg!(&job);

            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let _ = job.reply_tx.send(JobResult { foo: "okay".into() });
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum WorkerPoolErr {
    /// Invalid message encoding
    Other(crate::Error),
}
