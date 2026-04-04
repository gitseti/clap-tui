use std::error::Error as StdError;
use std::marker::PhantomData;
use std::time::Duration;

use clap::error::ErrorKind;
use clap::{Command, CommandFactory, Parser};
use ratatui::Frame;

use crate::config::TuiConfig;
use crate::controller;
use crate::error::TuiError;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::runtime::{AppEvent, CrosstermRuntime, Runtime};
use crate::ui;
use crate::update::{self, Effect};

/// Primary entry point for building and running the TUI.
///
/// Supported extension points for library users are:
/// - custom runtimes via [`TuiApp::with_runtime`]
/// - theming and layout via [`TuiApp::with_config`]
/// - startup command selection via [`crate::TuiConfig::start_command`]
///
/// Other public items are exported to support those seams, but internal controller,
/// pipeline, and rendering details are not stable extension points.
pub struct TuiApp<R: Runtime = CrosstermRuntime> {
    command: Command,
    config: TuiConfig,
    runtime: R,
}

/// Schema-bound TUI application for derive-based `clap` parsers.
///
/// This wrapper ties parser execution to the same `Parser + CommandFactory` type that produced
/// the rendered clap schema, avoiding post-hoc parser selection against an unrelated command.
pub struct ParserTuiApp<T, R: Runtime = CrosstermRuntime> {
    inner: TuiApp<R>,
    _parser: PhantomData<fn() -> T>,
}

impl TuiApp<CrosstermRuntime> {
    /// Create from a `clap::Command`.
    #[must_use]
    pub fn from_command(command: Command) -> Self {
        Self {
            command,
            config: TuiConfig::default(),
            runtime: CrosstermRuntime,
        }
    }

    /// Create a schema-bound app from a derive-based CLI.
    #[must_use]
    pub fn from_factory<T: Parser + CommandFactory>() -> ParserTuiApp<T, CrosstermRuntime> {
        ParserTuiApp::new()
    }
}

impl<R: Runtime> TuiApp<R> {
    /// Apply configuration.
    #[must_use]
    pub fn with_config(mut self, config: TuiConfig) -> Self {
        self.config = config;
        self
    }

    /// Replace the default runtime.
    #[must_use]
    pub fn with_runtime<NR: Runtime>(self, runtime: NR) -> TuiApp<NR> {
        TuiApp {
            command: self.command,
            config: self.config,
            runtime,
        }
    }

    /// Run the TUI and return the selected argv.
    ///
    /// Returns `Ok(Some(argv))` when the user runs a valid command and `Ok(None)` when the
    /// user exits without running. Validation stays inside the TUI flow, so invalid form
    /// state is surfaced in-app rather than returned as a clap error from this method.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal setup or event handling fails.
    pub fn run(self) -> Result<Option<Vec<String>>, TuiError> {
        match self.run_inner() {
            Ok(argv) => Ok(Some(argv)),
            Err(TuiError::Cancelled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Run the TUI and execute a custom handler with `ArgMatches`.
    ///
    /// Returns `Ok(())` when the user exits without running. When the user does run, this
    /// method reparses the selected argv with the original [`clap::Command`] before calling
    /// the handler.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal setup or event handling fails, when reparsing the
    /// selected argv with clap fails, or when the runner callback fails.
    pub fn run_with_matches<F, E>(self, runner: F) -> Result<(), TuiError>
    where
        F: FnOnce(clap::ArgMatches) -> Result<(), E>,
        E: StdError + Send + Sync + 'static,
    {
        let command = self.command.clone();
        let Some(argv) = self.run()? else {
            return Ok(());
        };
        run_matches_handler(command, argv, runner)
    }

    fn run_inner(self) -> Result<Vec<String>, TuiError> {
        let Self {
            command,
            config,
            mut runtime,
        } = self;
        let terminal = runtime.init_terminal()?;
        let mut session = TerminalSession::new(&mut runtime, terminal);
        event_loop(&command, &config, &mut session)
    }
}

impl<T> ParserTuiApp<T, CrosstermRuntime>
where
    T: Parser + CommandFactory,
{
    /// Create a schema-bound app from a derive-based CLI.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: TuiApp::from_command(T::command()),
            _parser: PhantomData,
        }
    }
}

impl<T> Default for ParserTuiApp<T, CrosstermRuntime>
where
    T: Parser + CommandFactory,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, R: Runtime> ParserTuiApp<T, R>
where
    T: Parser + CommandFactory,
{
    /// Apply configuration.
    #[must_use]
    pub fn with_config(self, config: TuiConfig) -> Self {
        Self {
            inner: self.inner.with_config(config),
            _parser: PhantomData,
        }
    }

    /// Replace the default runtime.
    #[must_use]
    pub fn with_runtime<NR: Runtime>(self, runtime: NR) -> ParserTuiApp<T, NR> {
        ParserTuiApp {
            inner: self.inner.with_runtime(runtime),
            _parser: PhantomData,
        }
    }

    /// Run the TUI and return the selected argv.
    ///
    /// Returns `Ok(Some(argv))` when the user runs a valid command and `Ok(None)` when the
    /// user exits without running. Validation stays inside the TUI flow, so invalid form
    /// state is surfaced in-app rather than returned as a clap error from this method.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal setup or event handling fails.
    pub fn run(self) -> Result<Option<Vec<String>>, TuiError> {
        self.inner.run()
    }

    /// Run the TUI and execute a custom handler with `ArgMatches`.
    ///
    /// Returns `Ok(())` when the user exits without running. When the user does run, this
    /// method reparses the selected argv with the bound command schema before calling the
    /// handler.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal setup or event handling fails, when reparsing the
    /// selected argv with clap fails, or when the runner callback fails.
    pub fn run_with_matches<F, E>(self, runner: F) -> Result<(), TuiError>
    where
        F: FnOnce(clap::ArgMatches) -> Result<(), E>,
        E: StdError + Send + Sync + 'static,
    {
        self.inner.run_with_matches(runner)
    }

    /// Run the TUI and parse into the bound `clap::Parser` type.
    ///
    /// Returns `Ok(())` when the user exits without running. When the user does run, this
    /// method reparses the selected argv with the bound parser type before calling the
    /// handler.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal setup or event handling fails, when reparsing the
    /// selected argv with the bound parser fails, or when the runner callback fails.
    pub fn run_with_parser<F, E>(self, runner: F) -> Result<(), TuiError>
    where
        F: FnOnce(T) -> Result<(), E>,
        E: StdError + Send + Sync + 'static,
    {
        let Some(argv) = self.run()? else {
            return Ok(());
        };
        run_parser_handler::<T, _, _>(argv, runner)
    }

    /// Drop down to the untyped app surface when only argv or `ArgMatches` execution is needed.
    #[must_use]
    pub fn into_untyped(self) -> TuiApp<R> {
        self.inner
    }
}

fn parse_or_display<T>(result: Result<T, clap::Error>) -> Result<Option<T>, TuiError> {
    match result {
        Ok(parsed) => Ok(Some(parsed)),
        Err(error) if error.kind() == ErrorKind::DisplayVersion => {
            error.print()?;
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

fn run_matches_handler<F, E>(command: Command, argv: Vec<String>, runner: F) -> Result<(), TuiError>
where
    F: FnOnce(clap::ArgMatches) -> Result<(), E>,
    E: StdError + Send + Sync + 'static,
{
    let Some(matches) = parse_or_display(command.try_get_matches_from(argv))? else {
        return Ok(());
    };
    runner(matches).map_err(|err| TuiError::Runner(Box::new(err)))
}

fn run_parser_handler<T, F, E>(argv: Vec<String>, runner: F) -> Result<(), TuiError>
where
    T: Parser,
    F: FnOnce(T) -> Result<(), E>,
    E: StdError + Send + Sync + 'static,
{
    let Some(parsed) = parse_or_display(T::try_parse_from(argv))? else {
        return Ok(());
    };
    runner(parsed).map_err(|err| TuiError::Runner(Box::new(err)))
}

fn event_loop<R: Runtime>(
    command: &Command,
    config: &TuiConfig,
    session: &mut TerminalSession<'_, R>,
) -> Result<Vec<String>, TuiError> {
    let mut observer = NoopDrawObserver;
    event_loop_with_observer(command, config, session, &mut observer)
}

fn event_loop_with_observer<R, O>(
    command: &Command,
    config: &TuiConfig,
    session: &mut TerminalSession<'_, R>,
    observer: &mut O,
) -> Result<Vec<String>, TuiError>
where
    R: Runtime,
    O: DrawObserver<R::Backend>,
{
    let mut state = AppState::from_command(command);
    if let Some(start) = config.start_command.clone() {
        controller::navigation::apply_start_command(&mut state, &start);
    }
    let mut frame_snapshot = FrameSnapshot::default();
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            session.draw(|frame| {
                frame_snapshot = render_frame(frame, &mut state, config);
            })?;
            observer.observe(session.backend(), &frame_snapshot)?;
            needs_redraw = false;
        }

        if !session.poll_event(redraw_timeout(&state))? {
            needs_redraw |= clear_expired_toast_and_request_redraw(&mut state);
            continue;
        }

        match handle_app_event(
            &session.read_event()?,
            &mut state,
            &frame_snapshot,
            config,
            session,
        ) {
            EventOutcome::Continue {
                needs_redraw: redraw,
            } => {
                needs_redraw |= redraw;
            }
            EventOutcome::Exit => return Err(TuiError::Cancelled),
            EventOutcome::Run(argv) => return Ok(argv),
        }
    }
}

fn redraw_timeout(state: &AppState) -> Duration {
    state
        .notifications
        .toast
        .as_ref()
        .map_or(Duration::from_secs(60 * 60), |toast| {
            toast
                .expires_at
                .saturating_duration_since(std::time::Instant::now())
        })
}

fn clear_expired_toast_and_request_redraw(state: &mut AppState) -> bool {
    let had_toast = state.notifications.toast.is_some();
    state.notifications.clear_expired_toast();
    had_toast && state.notifications.toast.is_none()
}

fn handle_effect<R: Runtime>(
    effect: Effect,
    state: &mut AppState,
    session: &mut TerminalSession<'_, R>,
) -> ActionOutcome {
    match effect {
        Effect::None => ActionOutcome::Continue,
        Effect::Run(argv) => {
            let validation = state.derived_validation();
            if validation.is_valid {
                ActionOutcome::Run(argv)
            } else {
                state.notifications.show_toast(
                    validation
                        .summary
                        .unwrap_or_else(|| "Command is invalid".to_string()),
                    Duration::from_secs(3),
                    true,
                );
                ActionOutcome::Continue
            }
        }
        Effect::CopyToClipboard(command) => {
            let result = session.copy_to_clipboard(&command);
            match result {
                Ok(()) => {
                    state.notifications.show_toast(
                        "Copied command to clipboard",
                        Duration::from_secs(2),
                        false,
                    );
                }
                Err(_) => state.notifications.show_toast(
                    "Clipboard unavailable",
                    Duration::from_secs(2),
                    true,
                ),
            }
            ActionOutcome::Continue
        }
        Effect::Exit => ActionOutcome::Exit,
    }
}

fn handle_app_event<R: Runtime>(
    event: &AppEvent,
    state: &mut AppState,
    frame_snapshot: &FrameSnapshot,
    config: &TuiConfig,
    session: &mut TerminalSession<'_, R>,
) -> EventOutcome {
    let mut needs_redraw = clear_expired_toast_and_request_redraw(state);

    match event {
        AppEvent::Key(key) => {
            if let Some(action) = controller::handle_key_event(*key, state, frame_snapshot, config)
            {
                let effect = update::apply_action(&action, state, frame_snapshot);
                match handle_effect(effect, state, session) {
                    ActionOutcome::Continue => {
                        needs_redraw |= true;
                        needs_redraw |= clear_expired_toast_and_request_redraw(state);
                        EventOutcome::Continue { needs_redraw }
                    }
                    ActionOutcome::Exit => EventOutcome::Exit,
                    ActionOutcome::Run(argv) => EventOutcome::Run(argv),
                }
            } else {
                needs_redraw |= clear_expired_toast_and_request_redraw(state);
                EventOutcome::Continue { needs_redraw }
            }
        }
        AppEvent::Mouse(mouse) => {
            if let Some(action) =
                controller::handle_mouse_event(*mouse, state, frame_snapshot, config)
            {
                let effect = update::apply_action(&action, state, frame_snapshot);
                match handle_effect(effect, state, session) {
                    ActionOutcome::Continue => {
                        needs_redraw |= true;
                        needs_redraw |= clear_expired_toast_and_request_redraw(state);
                        EventOutcome::Continue { needs_redraw }
                    }
                    ActionOutcome::Exit => EventOutcome::Exit,
                    ActionOutcome::Run(argv) => EventOutcome::Run(argv),
                }
            } else {
                needs_redraw |= clear_expired_toast_and_request_redraw(state);
                EventOutcome::Continue { needs_redraw }
            }
        }
        AppEvent::Resize { .. } => {
            needs_redraw = true;
            needs_redraw |= clear_expired_toast_and_request_redraw(state);
            EventOutcome::Continue { needs_redraw }
        }
        AppEvent::Paste(text) => {
            let effect =
                update::apply_action(&update::Action::Paste(text.clone()), state, frame_snapshot);
            match handle_effect(effect, state, session) {
                ActionOutcome::Continue => {
                    needs_redraw |= true;
                    needs_redraw |= clear_expired_toast_and_request_redraw(state);
                    EventOutcome::Continue { needs_redraw }
                }
                ActionOutcome::Exit => EventOutcome::Exit,
                ActionOutcome::Run(argv) => EventOutcome::Run(argv),
            }
        }
        AppEvent::FocusGained | AppEvent::FocusLost | AppEvent::Unsupported => {
            needs_redraw |= clear_expired_toast_and_request_redraw(state);
            EventOutcome::Continue { needs_redraw }
        }
    }
}

fn render_frame(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig) -> FrameSnapshot {
    ui::render(frame, state, config)
}

trait DrawObserver<B: ratatui::backend::Backend> {
    fn observe(&mut self, _backend: &B, _frame_snapshot: &FrameSnapshot) -> Result<(), TuiError> {
        Ok(())
    }
}

struct NoopDrawObserver;

impl<B: ratatui::backend::Backend> DrawObserver<B> for NoopDrawObserver {}

enum ActionOutcome {
    Continue,
    Exit,
    Run(Vec<String>),
}

enum EventOutcome {
    Continue { needs_redraw: bool },
    Exit,
    Run(Vec<String>),
}

struct TerminalSession<'a, R: Runtime> {
    runtime: &'a mut R,
    terminal: Option<ratatui::Terminal<R::Backend>>,
}

impl<'a, R: Runtime> TerminalSession<'a, R> {
    fn new(runtime: &'a mut R, terminal: ratatui::Terminal<R::Backend>) -> Self {
        Self {
            runtime,
            terminal: Some(terminal),
        }
    }

    fn draw<F>(&mut self, draw_fn: F) -> Result<(), TuiError>
    where
        F: FnOnce(&mut Frame<'_>),
    {
        self.terminal
            .as_mut()
            .expect("terminal session is active")
            .draw(draw_fn)
            .map(|_| ())
            .map_err(TuiError::from)
    }

    fn backend(&self) -> &R::Backend {
        self.terminal
            .as_ref()
            .expect("terminal session is active")
            .backend()
    }

    fn poll_event(&mut self, timeout: Duration) -> Result<bool, TuiError> {
        self.runtime.poll_event(timeout)
    }

    fn read_event(&mut self) -> Result<AppEvent, TuiError> {
        self.runtime.read_event()
    }

    fn copy_to_clipboard(&mut self, text: &str) -> Result<(), String> {
        self.runtime.copy_to_clipboard(text)
    }
}

impl<R: Runtime> Drop for TerminalSession<'_, R> {
    fn drop(&mut self) {
        if let Some(mut terminal) = self.terminal.take() {
            self.runtime.restore_terminal(&mut terminal);
        }
    }
}

#[cfg(test)]
mod scripted;
#[cfg(test)]
mod scripted_tests;

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::time::{Duration, Instant};

    use clap::{Arg, ArgAction, Command};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::{
        ActionOutcome, EventOutcome, TerminalSession, event_loop, handle_app_event, handle_effect,
        redraw_timeout,
    };
    use crate::frame_snapshot::FrameSnapshot;
    use crate::input::{AppState, Toast};
    use crate::pipeline;
    use crate::runtime::{AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, Runtime};
    use crate::spec::CommandSpec;
    use crate::update::Effect;
    use crate::{TuiConfig, TuiError};

    #[derive(Debug)]
    struct TestRuntime {
        events: VecDeque<AppEvent>,
        clipboard_result: Result<(), String>,
        copied_text: Option<String>,
    }

    impl TestRuntime {
        fn with_events(events: impl IntoIterator<Item = AppEvent>) -> Self {
            Self {
                events: events.into_iter().collect(),
                clipboard_result: Ok(()),
                copied_text: None,
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

        fn copy_to_clipboard(&mut self, text: &str) -> Result<(), String> {
            self.copied_text = Some(text.to_string());
            self.clipboard_result.clone()
        }
    }

    fn terminal_session(runtime: &mut TestRuntime) -> TerminalSession<'_, TestRuntime> {
        let terminal = runtime.init_terminal().expect("terminal");
        TerminalSession::new(runtime, terminal)
    }

    fn app_state() -> AppState {
        AppState::new(CommandSpec::from_command(&Command::new("tool")))
    }

    fn app_state_from_command(command: &Command) -> AppState {
        AppState::from_command(command)
    }

    #[test]
    fn event_loop_returns_cancelled_on_ctrl_c() {
        let mut runtime = TestRuntime::with_events([AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::Char('c'),
            AppKeyModifiers {
                control: true,
                alt: false,
                shift: false,
            },
        ))]);
        let terminal = runtime.init_terminal().expect("terminal");
        let mut session = TerminalSession::new(&mut runtime, terminal);

        let result = event_loop(&Command::new("tool"), &TuiConfig::default(), &mut session);

        assert!(matches!(result, Err(TuiError::Cancelled)));
    }

    #[test]
    fn event_loop_returns_built_argv_on_ctrl_enter() {
        let mut runtime = TestRuntime::with_events([AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::Enter,
            AppKeyModifiers {
                control: true,
                alt: false,
                shift: false,
            },
        ))]);
        let terminal = runtime.init_terminal().expect("terminal");
        let mut session = TerminalSession::new(&mut runtime, terminal);
        let command = Command::new("tool").arg(
            Arg::new("verbose")
                .long("verbose")
                .action(ArgAction::SetTrue),
        );

        let result = event_loop(&command, &TuiConfig::default(), &mut session);

        assert_eq!(result.expect("run result"), vec!["tool".to_string()]);
    }

    #[test]
    fn event_loop_returns_built_argv_on_ctrl_r() {
        let mut runtime = TestRuntime::with_events([AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::Char('r'),
            AppKeyModifiers {
                control: true,
                alt: false,
                shift: false,
            },
        ))]);
        let terminal = runtime.init_terminal().expect("terminal");
        let mut session = TerminalSession::new(&mut runtime, terminal);
        let command = Command::new("tool").arg(
            Arg::new("verbose")
                .long("verbose")
                .action(ArgAction::SetTrue),
        );

        let result = event_loop(&command, &TuiConfig::default(), &mut session);

        assert_eq!(result.expect("run result"), vec!["tool".to_string()]);
    }

    #[test]
    fn copy_effect_success_shows_success_toast() {
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state();

        let outcome = handle_effect(
            Effect::CopyToClipboard("tool --verbose".to_string()),
            &mut state,
            &mut session,
        );
        drop(session);

        assert!(matches!(outcome, ActionOutcome::Continue));
        assert_eq!(runtime.copied_text.as_deref(), Some("tool --verbose"));
        let toast = state.notifications.toast.as_ref().expect("toast");
        assert_eq!(toast.message, "Copied command to clipboard");
        assert!(!toast.is_error);
    }

    #[test]
    fn copy_effect_failure_shows_error_toast() {
        let mut runtime = TestRuntime::with_events([]);
        runtime.clipboard_result = Err("clipboard unavailable".to_string());
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state();

        let outcome = handle_effect(
            Effect::CopyToClipboard("tool --verbose".to_string()),
            &mut state,
            &mut session,
        );
        drop(session);

        assert!(matches!(outcome, ActionOutcome::Continue));
        assert_eq!(runtime.copied_text.as_deref(), Some("tool --verbose"));
        let toast = state.notifications.toast.as_ref().expect("toast");
        assert_eq!(toast.message, "Clipboard unavailable");
        assert!(toast.is_error);
    }

    #[test]
    fn invalid_run_effect_is_blocked_and_surfaces_validation_summary() {
        let command = Command::new("tool").arg(
            Arg::new("name")
                .long("name")
                .required(true)
                .action(ArgAction::Set),
        );
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state_from_command(&command);

        let outcome = handle_effect(
            Effect::Run(vec!["tool".to_string()]),
            &mut state,
            &mut session,
        );

        assert!(matches!(outcome, ActionOutcome::Continue));
        let toast = state.notifications.toast.as_ref().expect("toast");
        assert!(toast.is_error);
        assert_eq!(toast.message, "Missing required argument: --name");
    }

    #[test]
    fn run_uses_cached_validation_state_without_revalidating() {
        pipeline::reset_validation_call_count();

        let command = Command::new("tool").arg(
            Arg::new("name")
                .long("name")
                .required(true)
                .action(ArgAction::Set),
        );
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state_from_command(&command);
        let argv = state.preview_argv();

        assert_eq!(pipeline::validation_call_count(), 1);

        let outcome = handle_effect(Effect::Run(argv), &mut state, &mut session);

        assert!(matches!(outcome, ActionOutcome::Continue));
        assert_eq!(pipeline::validation_call_count(), 1);
        let toast = state.notifications.toast.as_ref().expect("toast");
        assert!(toast.is_error);
        assert_eq!(toast.message, "Missing required argument: --name");
    }

    #[test]
    fn run_matches_handler_treats_version_display_as_success_and_skips_runner() {
        let mut called = false;

        let result = super::run_matches_handler(
            Command::new("tool").version("1.2.3"),
            vec!["tool".to_string(), "--version".to_string()],
            |_matches| {
                called = true;
                Ok::<_, std::io::Error>(())
            },
        );

        assert!(result.is_ok());
        assert!(!called);
    }

    #[test]
    fn run_parser_handler_treats_version_display_as_success_and_skips_runner() {
        #[derive(clap::Parser)]
        #[command(name = "tool", version = "1.2.3")]
        struct Cli;

        let mut called = false;

        let result = super::run_parser_handler::<Cli, _, _>(
            vec!["tool".to_string(), "--version".to_string()],
            |_cli| {
                called = true;
                Ok::<_, std::io::Error>(())
            },
        );

        assert!(result.is_ok());
        assert!(!called);
    }

    #[test]
    fn parser_bound_app_runs_with_its_bound_parser_type() {
        #[derive(Debug, clap::Parser, PartialEq, Eq)]
        #[command(name = "tool")]
        struct Cli {
            #[arg(long, default_value = "world")]
            name: String,
        }

        let runtime = TestRuntime::with_events([AppEvent::Key(AppKeyEvent::new(
            AppKeyCode::Char('r'),
            AppKeyModifiers {
                control: true,
                ..AppKeyModifiers::default()
            },
        ))]);

        let mut parsed = None;
        let result = super::TuiApp::from_factory::<Cli>()
            .with_runtime(runtime)
            .run_with_parser(|cli| {
                parsed = Some(cli);
                Ok::<_, std::io::Error>(())
            });

        assert!(result.is_ok());
        assert_eq!(
            parsed,
            Some(Cli {
                name: "world".to_string()
            })
        );
    }

    #[test]
    fn help_style_invalid_run_toast_does_not_show_about_text() {
        let command = Command::new("tool")
            .about("Run the selected tool")
            .arg_required_else_help(true)
            .arg(Arg::new("path").required(true));
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state_from_command(&command);

        let outcome = handle_effect(
            Effect::Run(vec!["tool".to_string()]),
            &mut state,
            &mut session,
        );

        assert!(matches!(outcome, ActionOutcome::Continue));
        let toast = state.notifications.toast.as_ref().expect("toast");
        assert!(toast.is_error);
        assert_eq!(toast.message, "Missing required argument: path");
        assert!(!toast.message.contains("Run the selected tool"));
    }

    #[test]
    fn resize_event_requests_redraw() {
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state();

        let outcome = handle_app_event(
            &AppEvent::Resize {
                width: 120,
                height: 40,
            },
            &mut state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
            &mut session,
        );

        assert!(matches!(
            outcome,
            EventOutcome::Continue { needs_redraw: true }
        ));
    }

    #[test]
    fn paste_event_updates_focused_form_field() {
        let command = Command::new("tool").arg(Arg::new("path").long("path"));
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state_from_command(&command);
        state.ui.focus_form();

        let outcome = handle_app_event(
            &AppEvent::Paste("/tmp/foo".to_string()),
            &mut state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
            &mut session,
        );

        assert!(matches!(
            outcome,
            EventOutcome::Continue { needs_redraw: true }
        ));
        let form = state.domain.current_form().expect("form");
        let arg = state.domain.arg_for_input("path").expect("path arg");
        assert_eq!(
            form.compatibility_value(arg),
            Some(crate::input::ArgValue::Text("/tmp/foo".to_string()))
        );
        let derived = crate::pipeline::derive(&state);
        assert_eq!(
            derived.argv,
            vec![
                "tool".to_string(),
                "--path".to_string(),
                "/tmp/foo".to_string(),
            ]
        );
        assert!(derived.validation.is_valid);
    }

    #[test]
    fn paste_event_updates_search_query_when_search_is_focused() {
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state();
        state.ui.focus_search();

        let outcome = handle_app_event(
            &AppEvent::Paste("build".to_string()),
            &mut state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
            &mut session,
        );

        assert!(matches!(
            outcome,
            EventOutcome::Continue { needs_redraw: true }
        ));
        assert_eq!(state.ui.search_query, "build");
    }

    #[test]
    fn toast_timeout_behavior_is_unchanged() {
        let mut state = app_state();
        state.notifications.show_toast(
            "Copied command to clipboard",
            Duration::from_millis(250),
            false,
        );

        let timeout = redraw_timeout(&state);

        assert!(timeout > Duration::ZERO);
        assert!(timeout <= Duration::from_millis(250));
    }

    #[test]
    fn expired_toast_clears_during_continuous_key_input() {
        let mut runtime = TestRuntime::with_events([]);
        let mut session = terminal_session(&mut runtime);
        let mut state = app_state();
        state.notifications.toast = Some(Toast {
            message: "Copied command to clipboard".to_string(),
            expires_at: Instant::now()
                .checked_sub(Duration::from_millis(1))
                .expect("duration should be representable"),
            is_error: false,
        });

        let outcome = handle_app_event(
            &AppEvent::Key(AppKeyEvent::new(
                AppKeyCode::Char('x'),
                AppKeyModifiers::default(),
            )),
            &mut state,
            &FrameSnapshot::default(),
            &TuiConfig::default(),
            &mut session,
        );

        assert!(matches!(
            outcome,
            EventOutcome::Continue { needs_redraw: true }
        ));
        assert!(state.notifications.toast.is_none());
    }
}
