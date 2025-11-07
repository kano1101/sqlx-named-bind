/// Error types for sqlx-named-bind
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error during SQL template parsing
    #[error("Failed to parse SQL template: {0}")]
    Parse(#[from] regex::Error),

    /// Error from SQLx database operations
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Placeholder was referenced but not bound by the binder function
    #[error("Placeholder '{0}' was not bound by the binder function")]
    UnboundPlaceholder(String),
}

/// Result type alias for sqlx-named-bind operations
pub type Result<T> = std::result::Result<T, Error>;
