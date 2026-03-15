use std::error::Error as StdError;
use std::time::Duration;

use clap::{Command, CommandFactory, Parser};
use crossterm::event::Event;
use ratatui::Frame;

use crate::config::TuiConfig;
use crate::controller::{self, Action};
use crate::error::TuiError;
use crate::input::AppState;
use crate::runtime::{CrosstermRuntime, Runtime};
use crate::spec::CommandSpec;
use crate::ui;

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

    loop {
        state.clear_expired_toast();
        ui::prepare(&mut state);
        session.draw(|frame| render_frame(frame, &mut state, config))?;

        if !session.poll_event(Duration::from_millis(200))? {
            continue;
        }

        match session.read_event()? {
            Event::Key(key) => {
                if let Some(action) = controller::handle_key_event(key, &mut state, config) {
                    match handle_action(action, &mut state, session) {
                        ActionOutcome::Continue => {}
                        ActionOutcome::Exit => return Err(TuiError::Cancelled),
                        ActionOutcome::Run(argv) => return Ok(argv),
                    }
                }
            }
            Event::Mouse(mouse) => {
                if let Some(action) = controller::handle_mouse_event(mouse, &mut state, config) {
                    match handle_action(action, &mut state, session) {
                        ActionOutcome::Continue => {}
                        ActionOutcome::Exit => return Err(TuiError::Cancelled),
                        ActionOutcome::Run(argv) => return Ok(argv),
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_action<R: Runtime>(
    action: Action,
    state: &mut AppState,
    session: &mut TerminalSession<'_, R>,
) -> ActionOutcome {
    match action {
        Action::Run(argv) => ActionOutcome::Run(argv),
        Action::CopyCommand(command) => {
            let result = session.copy_to_clipboard(&command);
            match result {
                Ok(()) => {
                    state.show_toast("Copied command to clipboard", Duration::from_secs(2), false);
                }
                Err(_) => state.show_toast("Clipboard unavailable", Duration::from_secs(2), true),
            }
            ActionOutcome::Continue
        }
        Action::Exit => ActionOutcome::Exit,
    }
}

fn render_frame(frame: &mut Frame<'_>, state: &mut AppState, config: &TuiConfig) {
    ui::render(frame, state, config);
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
