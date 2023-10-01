use thiserror::Error;
use tokio::sync::oneshot::error::RecvError; // Import std::io::Error as IoError

#[derive(Error, Debug, PartialEq)]
#[allow(missing_docs)]
/// Error type for Esl
pub enum EslError {
    #[error("unknown error")]
    InternalError(String),

    #[error("Wrong password.")]
    AuthFailed,

    #[error("Unable to connect to destination server: {0}")]
    ConnectionError(String),

    #[error("{0:?}")]
    ApiError(String),
    #[error("Didnt get any digits")]
    NoInput,

    #[error(transparent)]
    ChannelError(#[from] RecvError),
}
impl From<std::io::Error> for EslError {
    fn from(error: std::io::Error) -> Self {
        Self::InternalError(error.to_string())
    }
}
