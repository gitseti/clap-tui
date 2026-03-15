use std::io::{self, Stdout};
use std::time::Duration;

use arboard::Clipboard;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::error::TuiError;

/// Runtime services required by `TuiApp`.
///
/// This trait allows advanced users to plug in custom terminal/event/clipboard
/// implementations while the default crate experience still uses crossterm.
pub trait Runtime {
    /// Terminal backend used by the runtime.
    type Backend: ratatui::backend::Backend;

    /// Enter interactive terminal mode and create a terminal instance.
    ///
    /// # Errors
    ///
    /// Returns an error when the runtime cannot switch the terminal into
    /// interactive mode.
    fn init_terminal(&mut self) -> Result<Terminal<Self::Backend>, TuiError>;

    /// Restore the terminal to its original state.
    ///
    /// Implementations should make a best effort to clean up even when prior
    /// runtime operations failed.
    fn restore_terminal(&mut self, terminal: &mut Terminal<Self::Backend>);

    /// Poll for an input event.
    ///
    /// # Errors
    ///
    /// Returns an error when event polling fails.
    fn poll_event(&mut self, timeout: Duration) -> Result<bool, TuiError>;

    /// Read the next input event.
    ///
    /// # Errors
    ///
    /// Returns an error when event reading fails.
    fn read_event(&mut self) -> Result<Event, TuiError>;

    /// Copy text to the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns an error string when the clipboard is unavailable.
    fn copy_to_clipboard(&mut self, text: &str) -> Result<(), String>;
}

/// Default runtime backed by crossterm and arboard.
#[derive(Debug, Default, Clone, Copy)]
pub struct CrosstermRuntime;

impl Runtime for CrosstermRuntime {
    type Backend = CrosstermBackend<Stdout>;

    fn init_terminal(&mut self) -> Result<Terminal<Self::Backend>, TuiError> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;

        if let Err(err) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err.into());
        }
        if let Err(err) = execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        ) {
            let _ = execute!(stdout, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            return Err(err.into());
        }
        #[cfg(feature = "mouse")]
        if let Err(err) = execute!(stdout, crossterm::event::EnableMouseCapture) {
            let _ = execute!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
            );
            let _ = execute!(stdout, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            return Err(err.into());
        }

        Terminal::new(CrosstermBackend::new(stdout)).map_err(TuiError::from)
    }

    fn restore_terminal(&mut self, terminal: &mut Terminal<Self::Backend>) {
        let _ = disable_raw_mode();
        #[cfg(feature = "mouse")]
        {
            let _ = execute!(
                terminal.backend_mut(),
                crossterm::event::DisableMouseCapture
            );
        }
        let _ = execute!(
            terminal.backend_mut(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        );
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal.show_cursor();
    }

    fn poll_event(&mut self, timeout: Duration) -> Result<bool, TuiError> {
        event::poll(timeout).map_err(TuiError::from)
    }

    fn read_event(&mut self) -> Result<Event, TuiError> {
        event::read().map_err(TuiError::from)
    }

    fn copy_to_clipboard(&mut self, text: &str) -> Result<(), String> {
        Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(text.to_string()))
            .map_err(|err| err.to_string())
    }
}
