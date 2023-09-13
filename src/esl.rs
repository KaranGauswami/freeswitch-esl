use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{connection::EslConnection, EslError};
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EslConnectionType {
    Inbound,
    Outbound,
}
/// Esl struct with inbound and outbound method.
pub struct Esl;
impl Esl {
    /// Creates new inbound connection to freeswitch
    pub async fn inbound(
        addr: impl ToSocketAddrs,
        password: impl ToString,
    ) -> Result<EslConnection, EslError> {
        EslConnection::new(addr, password, EslConnectionType::Inbound).await
    }

    /// Creates new server for outbound connection
    pub async fn outbound(stream: TcpStream) -> Result<EslConnection, EslError> {
        let connection =
            EslConnection::with_tcpstream(stream, "None", EslConnectionType::Outbound).await?;
        Ok(connection)
    }
}
