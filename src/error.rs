use std::fmt;

#[derive(Debug)]
pub enum TradierError {
    HttpError(String),
    JsonError(String)
}

impl fmt::Display for TradierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradierError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            TradierError::JsonError(msg) => write!(f, "JSON parsing error: {}", msg)
        }
    }
}

impl std::error::Error for TradierError {}

impl From<serde_json::Error> for TradierError {
    fn from(err: serde_json::Error) -> TradierError {
        TradierError::JsonError(err.to_string())
    }
}

impl From<reqwest::Error> for TradierError {
    fn from(err: reqwest::Error) -> TradierError {
        TradierError::HttpError(err.to_string())
    }
}

impl From<anyhow::Error> for TradierError {
    fn from(err: anyhow::Error) -> TradierError {
        TradierError::HttpError(err.to_string())
    }
}
