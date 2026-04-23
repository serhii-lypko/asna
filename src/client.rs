use std::io::{self, Cursor};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::SubmitJob;
use crate::connection::Connection;
use crate::message::Message;

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
        let tcp_stream = TcpStream::connect(addr).await?;
        let connection = Connection::new(tcp_stream);

        Ok(Client { connection })
    }

    pub async fn submit_job(&mut self, job: SubmitJob) -> io::Result<()> {
        let job_message = Message::SubmitJob(job);
        let _ = self.connection.write_message(&job_message).await;

        Ok(())
    }

    pub async fn ping(&mut self) -> io::Result<()> {
        let ping_message = Message::Ping;

        Ok(())
    }
}
