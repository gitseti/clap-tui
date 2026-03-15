use std::error::Error as StdError;
use std::time::Duration;

use clap::{Command, CommandFactory, Parser};
use crossterm::event::Event;
use ratatui::Frame;

use crate::config::TuiConfig;
use crate::controller;
use crate::error::TuiError;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::AppState;
use crate::runtime::{CrosstermRuntime, Runtime};
use crate::spec::CommandSpec;
use crate::ui;
use crate::update::{self, Effect};

/// Primary entry point for building and running the TUI.
pub struct TuiApp<R: Runtime = CrosstermRuntime> {
    command: Command,
    config: TuiConfig,
    runtime: R,
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

    /// Create from a `clap::CommandFactory` (derive-based CLI).
    #[must_use]
    pub fn from_factory<T: CommandFactory>() -> Self {
        Self::from_command(T::command())
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
    /// # Errors
    ///
    /// Returns an error when terminal setup, event handling, or clap validation fails.
    pub fn run(self) -> Result<Option<Vec<String>>, TuiError> {
        match self.run_inner() {
            Ok(argv) => Ok(Some(argv)),
            Err(TuiError::Cancelled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Run the TUI and execute a custom handler with `ArgMatches`.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal setup, event handling, clap validation, or the runner
    /// callback fails.
    pub fn run_with_matches<F, E>(self, runner: F) -> Result<(), TuiError>
    where
        F: FnOnce(clap::ArgMatches) -> Result<(), E>,
        E: StdError + Send + Sync + 'static,
    {
        let command = self.command.clone();
        let Some(argv) = self.run()? else {
            return Ok(());
        };
        let matches = command.try_get_matches_from(argv)?;
        runner(matches).map_err(|err| TuiError::Runner(Box::new(err)))
    }

    /// Run the TUI and parse into a `clap::Parser` struct.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal setup, event handling, clap validation, or the runner
    /// callback fails.
    pub fn run_with_parser<T, F, E>(self, runner: F) -> Result<(), TuiError>
    where
        T: Parser,
        F: FnOnce(T) -> Result<(), E>,
        E: StdError + Send + Sync + 'static,
    {
        let Some(argv) = self.run()? else {
            return Ok(());
        };
        let parsed = T::try_parse_from(argv)?;
        runner(parsed).map_err(|err| TuiError::Runner(Box::new(err)))
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

fn event_loop<R: Runtime>(
    command: &Command,
    config: &TuiConfig,
    session: &mut TerminalSession<'_, R>,
) -> Result<Vec<String>, TuiError> {
    let spec = CommandSpec::from_command(command);
    let mut state = AppState::new(spec);
    if let Some(start) = config.start_command.clone() {
        controller::navigation::apply_start_command(&mut state, &start);
    }
    update::normalize_state(&mut state);
    let mut frame_snapshot = FrameSnapshot::default();
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            session.draw(|frame| {
                frame_snapshot = render_frame(frame, &mut state, config);
            })?;
            needs_redraw = false;
        }

        if !session.poll_event(redraw_timeout(&state))? {
            let had_toast = state.notifications.toast.is_some();
            state.notifications.clear_expired_toast();
            if had_toast && state.notifications.toast.is_none() {
                needs_redraw = true;
            }
            continue;
        }

        match session.read_event()? {
            Event::Key(key) => {
                if let Some(action) = controller::handle_key_event(key, &state, &frame_snapshot, config)
                {
                    let effect = update::apply_action(action, &mut state, &frame_snapshot);
                    match handle_effect(effect, &mut state, session) {
                        ActionOutcome::Continue => {}
                        ActionOutcome::Exit => return Err(TuiError::Cancelled),
                        ActionOutcome::Run(argv) => return Ok(argv),
                    }
                    needs_redraw = true;
                }
            }
            Event::Mouse(mouse) => {
                if let Some(action) = controller::handle_mouse_event(
                    mouse,
                    &state,
                    &frame_snapshot,
                    config,
                ) {
                    let effect = update::apply_action(action, &mut state, &frame_snapshot);
                    match handle_effect(effect, &mut state, session) {
                        ActionOutcome::Continue => {}
                        ActionOutcome::Exit => return Err(TuiError::Cancelled),
                        ActionOutcome::Run(argv) => return Ok(argv),
                    }
                    needs_redraw = true;
                }
            }
            _ => {}
        }
    }
}

fn redraw_timeout(state: &AppState) -> Duration {
    state
        .notifications
        .toast
        .as_ref()
        .map_or(Duration::from_secs(60 * 60), |toast| {
            toast.expires_at
                .saturating_duration_since(std::time::Instant::now())
        })
}

fn handle_effect<R: Runtime>(
    effect: Effect,
    state: &mut AppState,
    session: &mut TerminalSession<'_, R>,
) -> ActionOutcome {
    match effect {
        Effect::None => ActionOutcome::Continue,
        Effect::Run(argv) => ActionOutcome::Run(argv),
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

fn render_frame(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig) -> FrameSnapshot {
    ui::render(frame, state, config)
}

enum ActionOutcome {
    Continue,
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

    fn poll_event(&mut self, timeout: Duration) -> Result<bool, TuiError> {
        self.runtime.poll_event(timeout)
    }

    fn read_event(&mut self) -> Result<Event, TuiError> {
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
mod tests {
    use std::collections::VecDeque;
    use std::time::Duration;

    use clap::{Arg, ArgAction, Command};
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::{ActionOutcome, TerminalSession, event_loop, handle_effect};
    use crate::input::AppState;
    use crate::runtime::Runtime;
    use crate::spec::CommandSpec;
    use crate::update::Effect;
    use crate::{TuiConfig, TuiError};

    #[derive(Debug)]
    struct TestRuntime {
        events: VecDeque<Event>,
        clipboard_result: Result<(), String>,
        copied_text: Option<String>,
    }

    impl TestRuntime {
        fn with_events(events: impl IntoIterator<Item = Event>) -> Self {
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

        fn read_event(&mut self) -> Result<Event, TuiError> {
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

    #[test]
    fn event_loop_returns_cancelled_on_ctrl_c() {
        let mut runtime = TestRuntime::with_events([Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        ))]);
        let terminal = runtime.init_terminal().expect("terminal");
        let mut session = TerminalSession::new(&mut runtime, terminal);

        let result = event_loop(&Command::new("tool"), &TuiConfig::default(), &mut session);

        assert!(matches!(result, Err(TuiError::Cancelled)));
    }

    #[test]
    fn event_loop_returns_built_argv_on_ctrl_enter() {
        let mut runtime = TestRuntime::with_events([Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::CONTROL,
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
}
