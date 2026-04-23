use asna::client::Client;
use asna::{Result, SubmitJob};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:8888").await?;

    let job1 = SubmitJob::new("awesome hello world".as_bytes().into());
    let _ = client.submit_job(job1).await;

    // let _ = client.ping().await;
    // let _ = client.ping().await;
    // tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    // let _ = client.ping().await;

    Ok(())
}
