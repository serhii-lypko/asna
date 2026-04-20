pub mod client;
pub mod connection;
pub mod dispatcher;
pub mod ingress;
pub mod thread_pool;

pub const DEFAULT_PORT: u16 = 8888;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
