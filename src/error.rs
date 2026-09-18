/// Errors returned by the TypeSafe SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No API key was provided via builder or environment variable.
    #[error("typesafe: missing API key (set TYPESAFE_API_KEY or use .api_key())")]
    MissingApiKey,

    /// The API returned a non-2xx HTTP response.
    #[error("{}", ApiError::fmt_error(.status, .detail, .body))]
    Api {
        status: u16,
        detail: Option<serde_json::Value>,
        body: String,
    },

    /// An HTTP transport error from reqwest.
    #[error("typesafe: {0}")]
    Http(#[from] reqwest::Error),
}

/// Helper for formatting API errors.
struct ApiError;

impl ApiError {
    fn fmt_error(status: &u16, detail: &Option<serde_json::Value>, body: &str) -> String {
        if let Some(d) = detail {
            format!("typesafe: HTTP {}: {}", status, d)
        } else {
            format!("typesafe: HTTP {}: {}", status, body)
        }
    }
}

/// A specialized `Result` type for TypeSafe operations.
pub type Result<T> = std::result::Result<T, Error>;
