use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::{connection::EslConnection, EslConnectionType, EslError};

pub struct Outbound {
    listener: TcpListener,
}
impl Outbound {
    pub(crate) async fn new(listener: TcpListener) -> Result<Self, EslError> {
        Ok(Self { listener })
    }
    pub async fn accept(&self) -> Result<(EslConnection, SocketAddr), EslError> {
        let (stream, addr) = self.listener.accept().await?;
        let connection =
            EslConnection::with_tcpstream(stream, "None", EslConnectionType::Outbound).await?;
        Ok((connection, addr))
    }
}
