mod client;
mod connection;
mod dispatcher;
mod ingress;
mod thread_pool;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    println!("Hello, world!");
}
