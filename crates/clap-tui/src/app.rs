use std::io::{self, Stdout};
use std::time::Duration;

use clap::{Command, CommandFactory, Parser};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::config::TuiConfig;
use crate::controller::{self, Action};
use crate::error::TuiError;
use crate::input::AppState;
use crate::spec::CommandSpec;
use crate::ui;

/// Primary entry point for building and running the TUI.
pub struct TuiApp {
    command: Command,
    config: TuiConfig,
}

impl TuiApp {
    /// Create from a `clap::Command`.
    pub fn from_command(command: Command) -> Self {
        Self {
            command,
            config: TuiConfig::default(),
        }
    }

    /// Create from a `clap::CommandFactory` (derive-based CLI).
    pub fn from_factory<T: CommandFactory>() -> Self {
        Self::from_command(T::command())
    }

    /// Apply configuration.
    pub fn with_config(mut self, config: TuiConfig) -> Self {
        self.config = config;
        self
    }

    /// Run the TUI and return the argv for the selected command.
    pub fn run(self) -> Result<Vec<String>, TuiError> {
        match self.run_inner() {
            Ok(argv) => Ok(argv),
            Err(TuiError::Cancelled) => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }

    /// Run the TUI and execute a custom handler with `ArgMatches`.
    pub fn run_with_matches<F>(self, runner: F) -> Result<(), TuiError>
    where
        F: FnOnce(clap::ArgMatches) -> anyhow::Result<()>,
    {
        let command = self.command.clone();
        let argv = match self.run_inner() {
            Ok(argv) => argv,
            Err(TuiError::Cancelled) => return Ok(()),
            Err(err) => return Err(err),
        };
        let matches = command.try_get_matches_from(argv)?;
        runner(matches).map_err(|err| TuiError::Terminal(io::Error::new(io::ErrorKind::Other, err)))
    }

    /// Run the TUI and parse into a `clap::Parser` struct.
    pub fn run_with_parser<T, F>(self, runner: F) -> Result<(), TuiError>
    where
        T: Parser,
        F: FnOnce(T) -> anyhow::Result<()>,
    {
        let argv = match self.run_inner() {
            Ok(argv) => argv,
            Err(TuiError::Cancelled) => return Ok(()),
            Err(err) => return Err(err),
        };
        let parsed = T::try_parse_from(argv)?;
        runner(parsed).map_err(|err| TuiError::Terminal(io::Error::new(io::ErrorKind::Other, err)))
    }

    fn run_inner(mut self) -> Result<Vec<String>, TuiError> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        #[cfg(feature = "mouse")]
        {
            execute!(stdout, crossterm::event::EnableMouseCapture)?;
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        #[cfg(feature = "mouse")]
        {
            execute!(
                terminal.backend_mut(),
                crossterm::event::DisableMouseCapture
            )?;
        }
        execute!(
            terminal.backend_mut(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<Vec<String>, TuiError> {
        let spec = CommandSpec::from_command(&self.command);
        let mut state = AppState::new(spec);
        if let Some(start) = self.config.start_command.clone() {
            controller::navigation::apply_start_command(&mut state, &start);
        }

        loop {
            terminal.draw(|frame| ui::render(frame, &mut state, &self.config))?;

            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Some(action) =
                            controller::handle_key_event(key, &mut state, &self.config)
                        {
                            match action {
                                Action::Run(argv) => return Ok(argv),
                                Action::Exit => return Err(TuiError::Cancelled),
                            }
                        }
                    }
                    Event::Mouse(mouse) => {
                        if let Some(action) =
                            controller::handle_mouse_event(mouse, &mut state, &self.config)
                        {
                            match action {
                                Action::Run(argv) => return Ok(argv),
                                Action::Exit => return Err(TuiError::Cancelled),
                            }
                        }
                    }
                    Event::Resize(_, _) => {
                        // Layout recalculates on next draw.
                    }
                    _ => {}
                }
            }
        }
    }
}
