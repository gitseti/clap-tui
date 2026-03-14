use thiserror::Error;

/// Errors returned by `clap-tui`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TuiError {
    /// Terminal IO error.
    #[error("terminal error: {0}")]
    Terminal(#[from] std::io::Error),
    /// Clap validation error.
    #[error("clap error: {0}")]
    Clap(#[from] clap::Error),
    /// User exited without running.
    #[error("cancelled")]
    Cancelled,
}
