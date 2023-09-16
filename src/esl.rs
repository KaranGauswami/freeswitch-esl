use tokio::net::TcpStream;

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
        stream: TcpStream,
        password: impl ToString,
    ) -> Result<EslConnection, EslError> {
        EslConnection::new(stream, password, EslConnectionType::Inbound).await
    }

    /// Creates new server for outbound connection
    pub async fn outbound(stream: TcpStream) -> Result<EslConnection, EslError> {
        let connection = EslConnection::new(stream, "None", EslConnectionType::Outbound).await?;
        Ok(connection)
    }
}
