use tokio::io::{AsyncRead, AsyncWrite};

use crate::connection::EslConnection;
use crate::EslError;
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
        stream: impl AsyncRead + AsyncWrite + Send + 'static + Unpin,
        password: impl ToString,
    ) -> Result<EslConnection, EslError> {
        EslConnection::new(stream, password, EslConnectionType::Inbound).await
    }

    /// Creates new server for outbound connection
    pub async fn outbound(
        stream: impl AsyncRead + AsyncWrite + Send + 'static + Unpin,
    ) -> Result<EslConnection, EslError> {
        let connection = EslConnection::new(stream, "None", EslConnectionType::Outbound).await?;
        Ok(connection)
    }
}
