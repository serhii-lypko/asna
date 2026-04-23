use asna::client::Client;
use asna::{Result, SubmitJob, SubmitJobKind};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:8888").await?;

    let job1 = SubmitJob::new(SubmitJobKind::Hashing, "awesome".as_bytes().into());
    let _ = client.submit_job(job1).await;

    // tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    // TODO -> send two (or more) messages one after another

    // TODO -> try to send messages in interval and observe

    Ok(())
}
