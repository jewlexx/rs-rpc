use crossbeam_channel::{RecvTimeoutError, SendError};
use serde_json::Error as JsonError;
use std::{
    io::Error as IoError,
    result::Result as StdResult,
    sync::mpsc::{RecvError as ChannelRecv, RecvTimeoutError as ChannelTimeout},
};

use crate::models::Message;

/// Error types from Discord
#[derive(Debug, thiserror::Error)]
#[allow(clippy::module_name_repetitions)]
pub enum DiscordError {
    #[error("Io Error")]
    /// Io Error
    IoError(#[from] IoError),
    #[error("Could not send message: {0}")]
    /// tx.send returned error
    SendMessage(#[from] SendError<Message>),
    #[error("Could not close event loop: {0}")]
    /// tx.send returned error
    CloseError(#[from] SendError<()>),
    #[error("Error Receiving message")]
    /// Error Receiving message
    ReceiveError(#[from] crossbeam_channel::RecvError),
    #[error("Error Receiving message")]
    /// Error Receiving message
    MPSCReceiveError(#[from] ChannelRecv),
    #[error("Error on Channel Timeout")]
    /// Timeout Error
    MPSCTimeout(#[from] ChannelTimeout),
    #[error("Receiving timed out")]
    /// Receiving timed out
    TimeoutError(#[from] RecvTimeoutError),
    #[error("Error parsing Json")]
    /// Json Error
    JsonError(#[from] JsonError),
    #[error("A thread ran into an error. See logs for more info.")]
    /// A thread ran into an error
    ThreadError,
    #[error("{0}")]
    /// Option unwrapped to None
    NoneError(String),
    #[error("Error converting values")]
    /// Conversion Error
    Conversion,
    #[error("Error decoding response. Incorrect header length")]
    /// Header Length Error
    HeaderLength,
    #[error("Header was received, but no message was received")]
    /// Header was received, but no message was received
    NoMessage,
    #[error("Error subscribing to an event")]
    /// Subscription Joining Error
    SubscriptionFailed,
    #[error("Connection was closed prematurely")]
    /// Connection Closing error
    ConnectionClosed,
    #[error("Connection has not been started")]
    /// Connection has not been started
    NotStarted,
    #[error("Event loop ran into an unknown error")]
    /// The send & receive loop ran into an error
    EventLoopError,
    /// No changes were made to the event handler
    #[error("No changes were made to the event handler. This can usually be ignored")]
    NoChangesMade,
    #[error("Could not safely shut down client. Thread is in use.")]
    /// RPC thread is in use
    ThreadInUse,
}

impl DiscordError {
    #[must_use]
    /// Tell whether an [`IoError`] would block the connection
    pub fn io_would_block(&self) -> bool {
        match self {
            Self::IoError(ref err) => err.kind() == std::io::ErrorKind::WouldBlock,
            _ => false,
        }
    }

    #[must_use]
    /// Checks if the error should break the connection
    pub fn should_break(&self) -> bool {
        match self {
            Self::IoError(ref err) => err.kind() == std::io::ErrorKind::ConnectionRefused,
            _ => false,
        }
    }
}

/// Result type for Discord RPC error types
pub type Result<T> = StdResult<T, DiscordError>;
