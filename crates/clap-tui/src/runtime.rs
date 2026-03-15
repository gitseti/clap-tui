use std::io::{self, Stdout};
use std::time::Duration;

use arboard::Clipboard;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::error::TuiError;

/// Crate-local input event emitted by the runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    /// Keyboard input.
    Key(AppKeyEvent),
    /// Mouse input.
    Mouse(AppMouseEvent),
    /// Terminal resize event.
    Resize {
        /// New terminal width.
        width: u16,
        /// New terminal height.
        height: u16,
    },
    /// Focus gained event.
    FocusGained,
    /// Focus lost event.
    FocusLost,
    /// Paste payload.
    Paste(String),
    /// Event variants the app does not currently model.
    Unsupported,
}

/// Crate-local keyboard event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppKeyEvent {
    /// Pressed key.
    pub code: AppKeyCode,
    /// Active modifiers.
    pub modifiers: AppKeyModifiers,
}

/// Crate-local key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppKeyCode {
    /// Character input.
    Char(char),
    /// Function key.
    F(u8),
    /// Backspace key.
    Backspace,
    /// Enter key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Tab key.
    Tab,
    /// Reverse tab key.
    BackTab,
    /// Delete key.
    Delete,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page down key.
    PageDown,
    /// Escape key.
    Esc,
    /// Unsupported key code.
    Null,
}

/// Crate-local key modifiers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppKeyModifiers {
    /// Ctrl was pressed.
    pub control: bool,
    /// Alt was pressed.
    pub alt: bool,
    /// Shift was pressed.
    pub shift: bool,
}

/// Crate-local mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppMouseEvent {
    /// Mouse action.
    pub kind: AppMouseEventKind,
    /// Column in terminal coordinates.
    pub column: u16,
    /// Row in terminal coordinates.
    pub row: u16,
    /// Active modifiers.
    pub modifiers: AppKeyModifiers,
}

/// Crate-local mouse event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppMouseEventKind {
    /// Button press.
    Down(AppMouseButton),
    /// Button release.
    Up(AppMouseButton),
    /// Button drag.
    Drag(AppMouseButton),
    /// Mouse move.
    Moved,
    /// Scroll down.
    ScrollDown,
    /// Scroll up.
    ScrollUp,
    /// Scroll left.
    ScrollLeft,
    /// Scroll right.
    ScrollRight,
}

/// Crate-local mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppMouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
}

impl AppKeyEvent {
    /// Construct a new key event.
    #[must_use]
    pub fn new(code: AppKeyCode, modifiers: AppKeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

impl From<KeyModifiers> for AppKeyModifiers {
    fn from(value: KeyModifiers) -> Self {
        Self {
            control: value.contains(KeyModifiers::CONTROL),
            alt: value.contains(KeyModifiers::ALT),
            shift: value.contains(KeyModifiers::SHIFT),
        }
    }
}

impl From<KeyCode> for AppKeyCode {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Char(c) => Self::Char(c),
            KeyCode::F(value) => Self::F(value),
            KeyCode::Backspace => Self::Backspace,
            KeyCode::Enter => Self::Enter,
            KeyCode::Left => Self::Left,
            KeyCode::Right => Self::Right,
            KeyCode::Up => Self::Up,
            KeyCode::Down => Self::Down,
            KeyCode::Tab => Self::Tab,
            KeyCode::BackTab => Self::BackTab,
            KeyCode::Delete => Self::Delete,
            KeyCode::Home => Self::Home,
            KeyCode::End => Self::End,
            KeyCode::PageUp => Self::PageUp,
            KeyCode::PageDown => Self::PageDown,
            KeyCode::Esc => Self::Esc,
            _ => Self::Null,
        }
    }
}

impl From<KeyEvent> for AppEvent {
    fn from(value: KeyEvent) -> Self {
        if value.kind == KeyEventKind::Release {
            return Self::Unsupported;
        }
        Self::Key(AppKeyEvent::new(
            AppKeyCode::from(value.code),
            AppKeyModifiers::from(value.modifiers),
        ))
    }
}

impl From<MouseButton> for AppMouseButton {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
        }
    }
}

impl From<MouseEvent> for AppEvent {
    fn from(value: MouseEvent) -> Self {
        let kind = match value.kind {
            MouseEventKind::Down(button) => AppMouseEventKind::Down(button.into()),
            MouseEventKind::Up(button) => AppMouseEventKind::Up(button.into()),
            MouseEventKind::Drag(button) => AppMouseEventKind::Drag(button.into()),
            MouseEventKind::Moved => AppMouseEventKind::Moved,
            MouseEventKind::ScrollDown => AppMouseEventKind::ScrollDown,
            MouseEventKind::ScrollUp => AppMouseEventKind::ScrollUp,
            MouseEventKind::ScrollLeft => AppMouseEventKind::ScrollLeft,
            MouseEventKind::ScrollRight => AppMouseEventKind::ScrollRight,
        };
        Self::Mouse(AppMouseEvent {
            kind,
            column: value.column,
            row: value.row,
            modifiers: AppKeyModifiers::from(value.modifiers),
        })
    }
}

impl From<Event> for AppEvent {
    fn from(value: Event) -> Self {
        match value {
            Event::Key(event) => Self::from(event),
            Event::Mouse(event) => Self::from(event),
            Event::Resize(width, height) => Self::Resize { width, height },
            Event::FocusGained => Self::FocusGained,
            Event::FocusLost => Self::FocusLost,
            Event::Paste(text) => Self::Paste(text),
        }
    }
}

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
    fn read_event(&mut self) -> Result<AppEvent, TuiError>;

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

    fn read_event(&mut self) -> Result<AppEvent, TuiError> {
        event::read().map(AppEvent::from).map_err(TuiError::from)
    }

    fn copy_to_clipboard(&mut self, text: &str) -> Result<(), String> {
        Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(text.to_string()))
            .map_err(|err| err.to_string())
    }
}
