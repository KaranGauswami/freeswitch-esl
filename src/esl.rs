use tokio::net::ToSocketAddrs;

use crate::{connection::EslConnection, EslError};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EslConnectionType {
    Inbound,
    Outbound,
}
pub struct Esl {}
impl Esl {
    pub async fn inbound(
        addr: impl ToSocketAddrs,
        password: impl ToString,
    ) -> Result<EslConnection, EslError> {
        EslConnection::new(addr, password, EslConnectionType::Inbound).await
    }
    pub fn outbound() {}
}
