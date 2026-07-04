use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("mailfold API error {status}: {message}")]
    Api {
        status: u16,
        message: String,
        retry_after: Option<u64>,
    },

    #[error("http transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to decode response body: {0}")]
    Decode(serde_json::Error),
}

impl Error {
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn message(&self) -> Option<&str> {
        match self {
            Error::Api { message, .. } => Some(message),
            _ => None,
        }
    }

    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Error::Api { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
