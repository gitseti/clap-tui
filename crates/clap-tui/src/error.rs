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
    /// Higher-level entry points such as `TuiApp::run`, `ParserTuiApp::run`,
    /// `run_with_matches`, and `run_with_parser` normalize this into `Ok(None)` or `Ok(())`.
    #[error("cancelled")]
    Cancelled,
    /// Synthetic launcher conflicts with an existing root command path.
    #[error("synthetic `tui` launcher conflicts with existing root command path `{path}`")]
    LauncherConflict {
        /// The conflicting root command path, such as `tool tui`.
        path: String,
    },
    /// Synthetic launcher cannot attach safely to the root command grammar.
    #[error("synthetic `tui` launcher is ambiguous for this root command: {reason}")]
    AmbiguousLauncherAttachment {
        /// Why the root clap grammar makes synthetic launcher attachment ambiguous.
        reason: String,
    },
}
