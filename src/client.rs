use tokio::net::{TcpStream, ToSocketAddrs};

use crate::connection::Connection;

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
        let tcp_stream = TcpStream::connect(addr).await?;
        let connection = Connection::new(tcp_stream);

        Ok(Client { connection })
    }
}
