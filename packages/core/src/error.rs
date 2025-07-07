use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{}", .0)]
    Vrlh(&'static str),
    #[error("{}", .0)]
    Btle(#[from] btleplug::Error),
    #[error("Secondary thread panicked!")]
    JoinError,
    #[error("Channel closed early, cannot send event!")]
    ChannelClosed,
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Self::ChannelClosed
    }
}

impl From<JoinError> for Error {
    fn from(_: JoinError) -> Self {
        Self::JoinError
    }
}
