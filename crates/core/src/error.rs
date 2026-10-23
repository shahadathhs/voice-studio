//! Error type shared by all front ends, with a stable process exit code per kind.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    /// A file is missing, unreadable, unsupported or malformed.
    #[error("{0}")]
    Input(String),
    /// The requested model is not installed.
    #[error("model not installed: {0}")]
    ModelMissing(String),
    /// A download failed: no connection, bad response, incomplete data or a checksum mismatch.
    #[error("{0}")]
    Network(String),
    /// Decoding, inference or device failure.
    #[error("{0}")]
    Engine(String),
    #[error("cancelled")]
    Cancelled,
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The library database could not be opened, read or written.
    #[error("library: {0}")]
    Storage(String),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::Storage(error.to_string())
    }
}

impl Error {
    /// Process exit code; documented in docs/product/cli.md.
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::Input(_) | Error::Json(_) => 3,
            Error::ModelMissing(_) => 4,
            Error::Engine(_) => 6,
            Error::Network(_) => 7,
            Error::NotImplemented(_) => 70,
            Error::Cancelled => 130,
            Error::Io(_) | Error::Storage(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_match_the_documented_table() {
        assert_eq!(Error::Input("x".into()).exit_code(), 3);
        assert_eq!(Error::ModelMissing("m".into()).exit_code(), 4);
        assert_eq!(Error::Engine("x".into()).exit_code(), 6);
        assert_eq!(Error::Network("x".into()).exit_code(), 7);
        assert_eq!(Error::NotImplemented("x").exit_code(), 70);
        assert_eq!(Error::Cancelled.exit_code(), 130);
    }
}
