use tokio::net::ToSocketAddrs;

use crate::{connection::EslConnection, outbound::Outbound, EslError};
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
    pub async fn outbound(addr: impl ToSocketAddrs) -> Result<Outbound, EslError> {
        Outbound::bind(addr).await
    }
}
