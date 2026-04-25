use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;
use std::time::Duration;

use clap::Command;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

use super::{DrawObserver, TerminalSession, event_loop_with_observer};
use crate::TuiConfig;
use crate::error::TuiError;
use crate::frame_snapshot::FrameSnapshot;
use crate::input::HoverTarget;
use crate::runtime::{
    AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, AppMouseButton, AppMouseEvent,
    AppMouseEventKind, Runtime,
};

#[derive(Debug, Clone)]
pub(crate) struct ObservedFrame {
    pub(crate) text: String,
    pub(crate) snapshot: FrameSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScriptedOutcome {
    Run(Vec<String>),
    Cancelled,
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptedRun {
    pub(crate) outcome: ScriptedOutcome,
    pub(crate) frames: Vec<ObservedFrame>,
}

#[derive(Debug, Clone)]
pub(crate) enum ScriptedStep {
    ExpectText(String),
    Event(AppEvent),
    TypeText(String),
    Paste(String),
    ClickFooter(HoverTarget),
    ClickInput(String),
    SelectDropdownOption(String),
    Resize { width: u16, height: u16 },
}

impl ScriptedStep {
    pub(crate) fn expect_text(text: impl Into<String>) -> Self {
        Self::ExpectText(text.into())
    }

    pub(crate) fn raw(event: AppEvent) -> Self {
        Self::Event(event)
    }

    pub(crate) fn key(code: AppKeyCode) -> Self {
        Self::Event(key_event(code))
    }

    pub(crate) fn ctrl(code: AppKeyCode) -> Self {
        Self::Event(ctrl_event(code))
    }

    pub(crate) fn type_text(text: impl Into<String>) -> Self {
        Self::TypeText(text.into())
    }

    pub(crate) fn paste(text: impl Into<String>) -> Self {
        Self::Paste(text.into())
    }

    pub(crate) fn click_footer(target: HoverTarget) -> Self {
        Self::ClickFooter(target)
    }

    pub(crate) fn click_input(arg_id: impl Into<String>) -> Self {
        Self::ClickInput(arg_id.into())
    }

    pub(crate) fn open_dropdown(arg_id: impl Into<String>) -> Self {
        Self::ClickInput(arg_id.into())
    }

    pub(crate) fn select_dropdown_option(label: impl Into<String>) -> Self {
        Self::SelectDropdownOption(label.into())
    }

    pub(crate) fn resize(width: u16, height: u16) -> Self {
        Self::Resize { width, height }
    }

    fn description(&self) -> String {
        match self {
            Self::ExpectText(text) => format!("text `{text}`"),
            Self::Event(event) => format!("raw event {event:?}"),
            Self::TypeText(text) => format!("typed text `{text}`"),
            Self::Paste(text) => format!("paste `{text}`"),
            Self::ClickFooter(target) => format!("footer target {target:?}"),
            Self::ClickInput(arg_id) => format!("input `{arg_id}`"),
            Self::SelectDropdownOption(label) => format!("dropdown option `{label}`"),
            Self::Resize { width, height } => format!("resize to {width}x{height}"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptedHarness {
    command: Command,
    config: TuiConfig,
    width: u16,
    height: u16,
    steps: Vec<ScriptedStep>,
}

impl ScriptedHarness {
    pub(crate) fn new(command: Command) -> Self {
        Self {
            command,
            config: TuiConfig::default(),
            width: 80,
            height: 24,
            steps: Vec::new(),
        }
    }

    pub(crate) fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub(crate) fn with_config(mut self, config: TuiConfig) -> Self {
        self.config = config;
        self
    }

    pub(crate) fn script(mut self, steps: impl IntoIterator<Item = ScriptedStep>) -> Self {
        self.steps.extend(steps);
        self
    }

    pub(crate) fn run(self) -> Result<ScriptedRun, TuiError> {
        let shared = Rc::new(RefCell::new(SharedScriptState::new(self.steps)));
        let mut runtime = ScriptedRuntime::new(self.width, self.height, Rc::clone(&shared));
        let terminal = runtime.init_terminal()?;
        let mut session = TerminalSession::new(&mut runtime, terminal);
        let mut observer = ScriptObserver::new(Rc::clone(&shared));
        let outcome = match event_loop_with_observer(
            &self.command,
            &self.config,
            &mut session,
            &mut observer,
        ) {
            Ok(argv) => ScriptedOutcome::Run(
                argv.into_iter()
                    .map(|token| token.to_string_lossy().to_string())
                    .collect(),
            ),
            Err(TuiError::Cancelled) => ScriptedOutcome::Cancelled,
            Err(err) => return Err(err),
        };
        let frames = shared.borrow().frames.clone();

        Ok(ScriptedRun { outcome, frames })
    }
}

#[derive(Debug)]
struct SharedScriptState {
    steps: VecDeque<ScriptedStep>,
    queued_events: VecDeque<AppEvent>,
    frames: Vec<ObservedFrame>,
}

impl SharedScriptState {
    fn new(steps: Vec<ScriptedStep>) -> Self {
        Self {
            steps: steps.into(),
            queued_events: VecDeque::new(),
            frames: Vec::new(),
        }
    }

    fn next_step_description(&self) -> String {
        self.steps
            .front()
            .map_or_else(|| "application exit".to_string(), ScriptedStep::description)
    }

    fn record_frame(&mut self, backend: &TestBackend, snapshot: &FrameSnapshot) {
        self.frames.push(ObservedFrame {
            text: buffer_text(backend),
            snapshot: snapshot.clone(),
        });
    }

    fn resolve_steps(&mut self, backend: &TestBackend) {
        let Some(frame) = self.frames.last() else {
            return;
        };

        while let Some(step) = self.steps.front() {
            match resolve_step(step, backend, frame) {
                StepResolution::Satisfied => {
                    self.steps.pop_front();
                }
                StepResolution::Queue(events) => {
                    self.queued_events.extend(events);
                    self.steps.pop_front();
                    break;
                }
                StepResolution::Pending => break,
            }
        }
    }
}

#[derive(Debug)]
struct ScriptedRuntime {
    width: u16,
    height: u16,
    shared: Rc<RefCell<SharedScriptState>>,
}

impl ScriptedRuntime {
    fn new(width: u16, height: u16, shared: Rc<RefCell<SharedScriptState>>) -> Self {
        Self {
            width,
            height,
            shared,
        }
    }
}

impl Runtime for ScriptedRuntime {
    type Backend = TestBackend;

    fn init_terminal(&mut self) -> Result<Terminal<Self::Backend>, TuiError> {
        Terminal::new(TestBackend::new(self.width, self.height)).map_err(TuiError::from)
    }

    fn restore_terminal(&mut self, _terminal: &mut Terminal<Self::Backend>) {}

    fn poll_event(&mut self, _timeout: Duration) -> Result<bool, TuiError> {
        let shared = self.shared.borrow();
        if !shared.queued_events.is_empty() {
            return Ok(true);
        }
        Err(TuiError::from(io::Error::other(format!(
            "script stalled waiting for {}",
            shared.next_step_description()
        ))))
    }

    fn read_event(&mut self) -> Result<AppEvent, TuiError> {
        self.shared
            .borrow_mut()
            .queued_events
            .pop_front()
            .ok_or_else(|| {
                TuiError::from(io::Error::other(
                    "script runtime tried to read an event from an empty queue",
                ))
            })
    }

    fn copy_to_clipboard(&mut self, _text: &str) -> Result<(), String> {
        Ok(())
    }
}

struct ScriptObserver {
    shared: Rc<RefCell<SharedScriptState>>,
}

impl ScriptObserver {
    fn new(shared: Rc<RefCell<SharedScriptState>>) -> Self {
        Self { shared }
    }
}

impl DrawObserver<TestBackend> for ScriptObserver {
    fn observe(
        &mut self,
        backend: &TestBackend,
        frame_snapshot: &FrameSnapshot,
    ) -> Result<(), TuiError> {
        let mut shared = self.shared.borrow_mut();
        shared.record_frame(backend, frame_snapshot);
        shared.resolve_steps(backend);
        Ok(())
    }
}

enum StepResolution {
    Satisfied,
    Queue(Vec<AppEvent>),
    Pending,
}

fn resolve_step(
    step: &ScriptedStep,
    backend: &TestBackend,
    frame: &ObservedFrame,
) -> StepResolution {
    match step {
        ScriptedStep::ExpectText(text) => {
            if frame.text.contains(text) {
                StepResolution::Satisfied
            } else {
                StepResolution::Pending
            }
        }
        ScriptedStep::Event(event) => StepResolution::Queue(vec![event.clone()]),
        ScriptedStep::TypeText(text) => StepResolution::Queue(
            text.chars()
                .map(|ch| key_event(AppKeyCode::Char(ch)))
                .collect(),
        ),
        ScriptedStep::Paste(text) => StepResolution::Queue(vec![AppEvent::Paste(text.clone())]),
        ScriptedStep::ClickFooter(target) => frame
            .snapshot
            .layout
            .footer_buttons
            .iter()
            .find(|button| button.target == *target)
            .map_or(StepResolution::Pending, |button| {
                StepResolution::Queue(vec![mouse_down(center_of(button.rect))])
            }),
        ScriptedStep::ClickInput(arg_id) => frame
            .snapshot
            .form_input_rect(arg_id)
            .map_or(StepResolution::Pending, |rect| {
                StepResolution::Queue(vec![mouse_down(center_of(rect))])
            }),
        ScriptedStep::SelectDropdownOption(label) => {
            dropdown_row_for_label(backend, frame.snapshot.layout.dropdown, label)
                .map_or(StepResolution::Pending, |row| {
                    StepResolution::Queue(vec![mouse_down((row.0, row.1))])
                })
        }
        ScriptedStep::Resize { width, height } => StepResolution::Queue(vec![AppEvent::Resize {
            width: *width,
            height: *height,
        }]),
    }
}

fn key_event(code: AppKeyCode) -> AppEvent {
    AppEvent::Key(AppKeyEvent::new(code, AppKeyModifiers::default()))
}

fn ctrl_event(code: AppKeyCode) -> AppEvent {
    AppEvent::Key(AppKeyEvent::new(
        code,
        AppKeyModifiers {
            control: true,
            ..AppKeyModifiers::default()
        },
    ))
}

fn mouse_down((column, row): (u16, u16)) -> AppEvent {
    AppEvent::Mouse(AppMouseEvent {
        kind: AppMouseEventKind::Down(AppMouseButton::Left),
        column,
        row,
        modifiers: AppKeyModifiers::default(),
    })
}

fn center_of(rect: Rect) -> (u16, u16) {
    let x = rect.x.saturating_add(rect.width.saturating_sub(1) / 2);
    let y = rect.y.saturating_add(rect.height.saturating_sub(1) / 2);
    (x, y)
}

fn dropdown_row_for_label(
    backend: &TestBackend,
    area: Option<Rect>,
    label: &str,
) -> Option<(u16, u16)> {
    let area = area?;
    for row in area.y.saturating_add(1)..area.y.saturating_add(area.height.saturating_sub(1)) {
        let line = buffer_row_text(backend, area, row);
        if line.contains(label) {
            return Some((area.x.saturating_add(1), row));
        }
    }
    None
}

fn buffer_row_text(backend: &TestBackend, area: Rect, row: u16) -> String {
    let mut line = String::new();
    for column in area.x..area.x.saturating_add(area.width) {
        line.push_str(backend.buffer()[(column, row)].symbol());
    }
    line
}

fn buffer_text(backend: &TestBackend) -> String {
    backend
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>()
}
