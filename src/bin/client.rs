use asna::Result;
use asna::client::Client;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:8888").await?;

    //

    Ok(())
}
