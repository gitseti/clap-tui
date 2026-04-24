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
    /// Application callback error.
    #[error("runner error: {0}")]
    Runner(Box<dyn std::error::Error + Send + Sync>),
    /// Lower-level TUI flow exited without running.
    ///
    /// Higher-level entry points such as `TuiApp::run`, `Tui::run`, and `run_with_matches`
    /// normalize this into `Ok(None)` or let callers decide what cancellation means.
    #[error("cancelled")]
    Cancelled,
    /// Requested entrypoint subcommand does not exist on the top-level render command.
    #[error("{}", unknown_entrypoint_message(name, candidates))]
    UnknownEntrypoint {
        /// Requested canonical top-level subcommand name.
        name: String,
        /// Available canonical top-level subcommand names in declaration order.
        candidates: Vec<String>,
    },
}

fn unknown_entrypoint_message(name: &str, candidates: &[String]) -> String {
    if candidates.is_empty() {
        format!("unknown TUI entrypoint `{name}`; no top-level subcommands are available")
    } else {
        format!(
            "unknown TUI entrypoint `{name}`; expected one of: {}",
            candidates.join(", ")
        )
    }
}
