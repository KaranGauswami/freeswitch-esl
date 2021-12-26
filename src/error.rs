use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Ord, PartialOrd, Eq, Hash, Error)]
pub enum InboundError {
    #[error("unknown error")]
    InternalError(String),

    #[error("Wrong password.")]
    AuthFailed,

    #[error("Unable to connect to destination server.")]
    ConnectionError(String),

    #[error("{0:?}")]
    ApiError(String),

    #[error("")]
    CodeParseError(),
}

impl From<std::io::Error> for InboundError {
    fn from(error: std::io::Error) -> Self {
        Self::InternalError(error.to_string())
    }
}
