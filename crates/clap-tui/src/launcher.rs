use std::ffi::OsString;
use std::marker::PhantomData;

use clap::{ArgMatches, Command, CommandFactory, Parser};

use crate::app::TuiApp;
use crate::config::TuiConfig;
use crate::error::TuiError;
use crate::runtime::{CrosstermRuntime, Runtime};

const SYNTHETIC_LAUNCHER_NAME: &str = "tui";
const SYNTHETIC_LAUNCHER_ABOUT: &str = "Launch the interactive terminal UI";

/// Canonical typed launcher for derive-based root `clap` parsers.
///
/// `ParserLauncher` owns the synthetic root `tui` entrypoint flow:
/// - ordinary CLI help, version output, and diagnostics come from the augmented command surface
/// - `tool tui` launches the TUI and parses successful output back into the same root parser type
/// - non-TUI invocations fall through to the bound typed parser
pub struct ParserLauncher<T, R: Runtime = CrosstermRuntime> {
    config: TuiConfig,
    runtime: R,
    _parser: PhantomData<fn() -> T>,
}

impl<T> ParserLauncher<T, CrosstermRuntime>
where
    T: Parser + CommandFactory,
{
    /// Create the canonical typed launcher for a derive-based CLI root parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: TuiConfig::default(),
            runtime: CrosstermRuntime,
            _parser: PhantomData,
        }
    }
}

impl<T> Default for ParserLauncher<T, CrosstermRuntime>
where
    T: Parser + CommandFactory,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, R: Runtime> ParserLauncher<T, R>
where
    T: Parser + CommandFactory,
{
    /// Apply configuration used when the synthetic `tui` launcher starts the TUI.
    #[must_use]
    pub fn with_config(mut self, config: TuiConfig) -> Self {
        self.config = config;
        self
    }

    /// Replace the default runtime used by the synthetic `tui` launcher.
    #[must_use]
    pub fn with_runtime<NR: Runtime>(self, runtime: NR) -> ParserLauncher<T, NR> {
        ParserLauncher {
            config: self.config,
            runtime,
            _parser: PhantomData,
        }
    }

    /// Run the typed launcher against `std::env::args_os()`.
    ///
    /// Clap help, version output, and diagnostics terminate directly from the augmented command
    /// surface. TUI runtime/setup failures are converted through `E: From<TuiError>`.
    ///
    /// # Errors
    ///
    /// Returns the user handler error type `E` for handler failures and for `clap-tui` launcher
    /// or TUI runtime errors converted via `From<TuiError>`.
    pub fn run<F, E>(self, runner: F) -> Result<(), E>
    where
        F: FnOnce(T) -> Result<(), E>,
        E: From<TuiError>,
    {
        self.run_with_args(std::env::args_os(), runner)
    }

    /// Run the typed launcher against a custom argv source.
    ///
    /// Clap help, version output, and diagnostics terminate directly from the augmented command
    /// surface. TUI runtime/setup failures are converted through `E: From<TuiError>`.
    ///
    /// # Errors
    ///
    /// Returns the user handler error type `E` for handler failures and for `clap-tui` launcher
    /// or TUI runtime errors converted via `From<TuiError>`.
    pub fn run_with_args<I, A, F, E>(self, args: I, runner: F) -> Result<(), E>
    where
        I: IntoIterator<Item = A>,
        A: Into<OsString>,
        F: FnOnce(T) -> Result<(), E>,
        E: From<TuiError>,
    {
        match self.dispatch(args) {
            Ok(Some(parsed)) => runner(parsed),
            Ok(None) => Ok(()),
            Err(DispatchError::Clap(error)) => error.exit(),
            Err(DispatchError::Tui(error)) => Err(E::from(error)),
        }
    }

    fn dispatch<I, A>(self, args: I) -> Result<Option<T>, DispatchError>
    where
        I: IntoIterator<Item = A>,
        A: Into<OsString>,
    {
        let os_args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        let prepared = PreparedRootCommand::new(T::command()).map_err(DispatchError::Tui)?;
        let matches = prepared
            .parse_command
            .clone()
            .try_get_matches_from(os_args.clone())
            .map_err(DispatchError::Clap)?;

        if matches_select_synthetic_launcher(&matches) {
            let run_result = TuiApp::from_command(prepared.render_command)
                .with_config(self.config)
                .with_runtime(self.runtime)
                .run()
                .map_err(DispatchError::Tui)?;

            let Some(tui_argv) = run_result else {
                return Ok(None);
            };

            T::try_parse_from(tui_argv)
                .map(Some)
                .map_err(DispatchError::Clap)
        } else {
            T::try_parse_from(os_args)
                .map(Some)
                .map_err(DispatchError::Clap)
        }
    }
}

#[derive(Debug)]
enum DispatchError {
    Clap(clap::Error),
    Tui(TuiError),
}

#[derive(Debug)]
struct PreparedRootCommand {
    parse_command: Command,
    render_command: Command,
}

impl PreparedRootCommand {
    fn new(command: Command) -> Result<Self, TuiError> {
        validate_root_launcher_attachment(&command)?;

        let parse_command = add_synthetic_launcher(command.clone());
        let mut render_command = add_synthetic_launcher(command);
        hide_synthetic_launcher(&mut render_command);

        Ok(Self {
            parse_command,
            render_command,
        })
    }
}

fn validate_root_launcher_attachment(command: &Command) -> Result<(), TuiError> {
    if let Some(conflict_path) = existing_tui_conflict(command) {
        return Err(TuiError::LauncherConflict {
            path: conflict_path,
        });
    }

    if command.is_allow_external_subcommands_set() {
        return Err(TuiError::AmbiguousLauncherAttachment {
            reason: "root command allows external subcommands that can already consume `tool tui`"
                .to_string(),
        });
    }

    if command.is_trailing_var_arg_set() || has_ambiguous_root_positional(command) {
        return Err(TuiError::AmbiguousLauncherAttachment {
            reason:
                "root command captures trailing positional input that makes `tool tui` ambiguous"
                    .to_string(),
        });
    }

    let ordinary_probe = vec![
        OsString::from(command.get_name()),
        OsString::from(SYNTHETIC_LAUNCHER_NAME),
    ];
    if command.clone().try_get_matches_from(ordinary_probe).is_ok() {
        return Err(TuiError::AmbiguousLauncherAttachment {
            reason: "the unmodified clap grammar already accepts `tool tui` as ordinary input"
                .to_string(),
        });
    }

    Ok(())
}

fn existing_tui_conflict(command: &Command) -> Option<String> {
    command.get_subcommands().find_map(|subcommand| {
        let name = subcommand.get_name();
        if name == SYNTHETIC_LAUNCHER_NAME {
            return Some(format!("{} {}", command.get_name(), name));
        }

        subcommand
            .get_visible_aliases()
            .find(|alias| *alias == SYNTHETIC_LAUNCHER_NAME)
            .map(|_| format!("{} {}", command.get_name(), SYNTHETIC_LAUNCHER_NAME))
    })
}

fn has_ambiguous_root_positional(command: &Command) -> bool {
    command
        .get_arguments()
        .any(|arg| arg.is_positional() && (arg.is_last_set() || arg.is_trailing_var_arg_set()))
}

fn add_synthetic_launcher(command: Command) -> Command {
    command.subcommand(synthetic_launcher_subcommand())
}

fn hide_synthetic_launcher(command: &mut Command) {
    for subcommand in command.get_subcommands_mut() {
        if subcommand.get_name() == SYNTHETIC_LAUNCHER_NAME {
            *subcommand = subcommand.clone().hide(true);
            break;
        }
    }
}

fn synthetic_launcher_subcommand() -> Command {
    Command::new(SYNTHETIC_LAUNCHER_NAME).about(SYNTHETIC_LAUNCHER_ABOUT)
}

fn matches_select_synthetic_launcher(matches: &ArgMatches) -> bool {
    matches
        .subcommand()
        .is_some_and(|(name, _)| name == SYNTHETIC_LAUNCHER_NAME)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io;
    use std::time::Duration;

    use clap::{Arg, Command, CommandFactory, Parser, Subcommand};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::{ParserLauncher, PreparedRootCommand};
    use crate::TuiError;
    use crate::runtime::{AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, Runtime};
    use crate::spec::CommandSpec;

    #[derive(Debug)]
    struct TestRuntime {
        events: VecDeque<AppEvent>,
    }

    impl TestRuntime {
        fn with_events(events: impl IntoIterator<Item = AppEvent>) -> Self {
            Self {
                events: events.into_iter().collect(),
            }
        }
    }

    impl Runtime for TestRuntime {
        type Backend = TestBackend;

        fn init_terminal(&mut self) -> Result<Terminal<Self::Backend>, TuiError> {
            Terminal::new(TestBackend::new(80, 24)).map_err(TuiError::from)
        }

        fn restore_terminal(&mut self, _terminal: &mut Terminal<Self::Backend>) {}

        fn poll_event(&mut self, _timeout: Duration) -> Result<bool, TuiError> {
            Ok(!self.events.is_empty())
        }

        fn read_event(&mut self) -> Result<AppEvent, TuiError> {
            Ok(self.events.pop_front().expect("queued event"))
        }

        fn copy_to_clipboard(&mut self, _text: &str) -> Result<(), String> {
            Ok(())
        }
    }

    fn dispatch<T, I, A>(
        launcher: ParserLauncher<T, TestRuntime>,
        args: I,
    ) -> Result<Option<T>, String>
    where
        T: Parser + CommandFactory,
        I: IntoIterator<Item = A>,
        A: Into<std::ffi::OsString>,
    {
        launcher.dispatch(args).map_err(|error| match error {
            super::DispatchError::Clap(clap) => clap.to_string(),
            super::DispatchError::Tui(tui) => tui.to_string(),
        })
    }

    #[derive(Debug, Parser, PartialEq, Eq)]
    #[command(name = "tool", version = "1.2.3")]
    struct SimpleCli {
        #[arg(long, default_value = "world")]
        name: String,
    }

    #[derive(Debug, Parser, PartialEq, Eq)]
    #[command(name = "tool", subcommand_required = true)]
    struct CommandCli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(Debug, Subcommand, PartialEq, Eq)]
    enum Commands {
        Build,
    }

    #[test]
    fn launcher_rejects_existing_tui_subcommand() {
        let error = PreparedRootCommand::new(Command::new("tool").subcommand(Command::new("tui")))
            .expect_err("launcher should reject conflicting subcommand");

        assert!(
            error
                .to_string()
                .contains("conflicts with existing root command path")
        );
    }

    #[test]
    fn launcher_rejects_existing_tui_alias() {
        let error = PreparedRootCommand::new(
            Command::new("tool").subcommand(Command::new("shell").visible_alias("tui")),
        )
        .expect_err("launcher should reject conflicting alias");

        assert!(error.to_string().contains("tool tui"));
    }

    #[test]
    fn launcher_rejects_external_subcommand_hosts() {
        let error = PreparedRootCommand::new(Command::new("tool").allow_external_subcommands(true))
            .expect_err("launcher should reject external subcommands");

        assert!(error.to_string().contains("external subcommands"));
    }

    #[test]
    fn launcher_rejects_trailing_capture_hosts() {
        let error = PreparedRootCommand::new(
            Command::new("tool")
                .trailing_var_arg(true)
                .arg(Arg::new("raw").raw(true).required(true)),
        )
        .expect_err("launcher should reject trailing capture");

        assert!(error.to_string().contains("trailing positional input"));
    }

    #[test]
    fn augmented_help_includes_synthetic_tui_launcher() {
        let launcher =
            ParserLauncher::<CommandCli, _>::new().with_runtime(TestRuntime::with_events([]));
        let error = dispatch(launcher, ["tool", "--help"]).expect_err("help should short-circuit");

        assert!(error.contains("tui"));
        assert!(error.contains("Launch the interactive terminal UI"));
    }

    #[test]
    fn parse_failures_use_augmented_command_surface() {
        let launcher =
            ParserLauncher::<SimpleCli, _>::new().with_runtime(TestRuntime::with_events([]));
        let error = dispatch(launcher, ["tool", "tui", "unexpected"])
            .expect_err("parse failure should short-circuit");

        assert!(error.contains("unexpected"));
        assert!(error.contains("Usage: tool tui"));
    }

    #[test]
    fn ordinary_invocation_uses_typed_parser_path() {
        let launcher =
            ParserLauncher::<SimpleCli, _>::new().with_runtime(TestRuntime::with_events([]));
        let parsed =
            dispatch(launcher, ["tool", "--name", "friend"]).expect("non-tui parse should work");

        assert_eq!(
            parsed,
            Some(SimpleCli {
                name: "friend".to_string(),
            })
        );
    }

    #[test]
    fn synthetic_tui_launch_parses_back_into_root_type() {
        let runtime = TestRuntime::with_events([AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::Char('r'),
            AppKeyModifiers {
                control: true,
                ..AppKeyModifiers::default()
            },
        ))]);
        let launcher = ParserLauncher::<SimpleCli, _>::new().with_runtime(runtime);

        let parsed = dispatch(launcher, ["tool", "tui"]).expect("synthetic tui launch should work");

        assert_eq!(
            parsed,
            Some(SimpleCli {
                name: "world".to_string(),
            })
        );
    }

    #[test]
    fn cancelled_tui_launch_returns_success_without_parsed_value() {
        let runtime = TestRuntime::with_events([AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::Char('c'),
            AppKeyModifiers {
                control: true,
                ..AppKeyModifiers::default()
            },
        ))]);
        let launcher = ParserLauncher::<SimpleCli, _>::new().with_runtime(runtime);

        let parsed =
            dispatch(launcher, ["tool", "tui"]).expect("cancelled tui launch should succeed");

        assert_eq!(parsed, None);
    }

    #[test]
    fn hidden_synthetic_launcher_is_omitted_from_tui_command_tree() {
        let prepared =
            PreparedRootCommand::new(SimpleCli::command()).expect("launcher should prepare");
        let spec = CommandSpec::from_command(&prepared.render_command);

        assert!(
            spec.subcommands
                .iter()
                .all(|subcommand| subcommand.name != "tui")
        );
    }

    #[test]
    fn launcher_run_preserves_user_handler_errors() {
        let launcher =
            ParserLauncher::<SimpleCli, _>::new().with_runtime(TestRuntime::with_events([]));

        let error = launcher
            .run_with_args(["tool"], |_cli| {
                Err::<(), Box<dyn std::error::Error + Send + Sync>>(Box::new(io::Error::other(
                    "boom",
                )))
            })
            .expect_err("user error should pass through");

        assert_eq!(error.to_string(), "boom");
    }
}
