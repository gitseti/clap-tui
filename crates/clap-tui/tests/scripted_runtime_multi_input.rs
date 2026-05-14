use std::collections::VecDeque;
use std::ffi::OsString;
use std::time::Duration;

use clap::{Arg, ArgAction, Command};
use clap_tui::{
    AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers, Runtime, TuiApp, TuiConfig, TuiError,
};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[derive(Debug)]
struct ScriptedRuntime {
    events: VecDeque<AppEvent>,
}

impl ScriptedRuntime {
    fn new(events: impl IntoIterator<Item = AppEvent>) -> Self {
        Self {
            events: events.into_iter().collect(),
        }
    }
}

impl Runtime for ScriptedRuntime {
    type Backend = TestBackend;

    fn init_terminal(&mut self) -> Result<Terminal<Self::Backend>, TuiError> {
        let Ok(terminal) = Terminal::new(TestBackend::new(80, 24));
        Ok(terminal)
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

fn key(code: AppKeyCode) -> AppEvent {
    AppEvent::Key(AppKeyEvent::new(code, AppKeyModifiers::default()))
}

fn ctrl(code: AppKeyCode) -> AppEvent {
    AppEvent::Key(AppKeyEvent::new(
        code,
        AppKeyModifiers {
            control: true,
            ..AppKeyModifiers::default()
        },
    ))
}

fn argv_to_strings(argv: Vec<OsString>) -> Vec<String> {
    argv.into_iter()
        .map(|token| token.to_string_lossy().to_string())
        .collect()
}

fn text(input: &str) -> Vec<AppEvent> {
    input.chars().map(|c| key(AppKeyCode::Char(c))).collect()
}

#[test]
fn repeated_occurrence_input_round_trips_through_the_real_app_loop() {
    let command = Command::new("tool").arg(
        Arg::new("include")
            .long("include")
            .action(ArgAction::Append)
            .num_args(1),
    );
    let mut events = vec![key(AppKeyCode::Tab), key(AppKeyCode::Tab)];
    events.extend(text("alpha"));
    events.push(key(AppKeyCode::Enter));
    events.extend(text("beta"));
    events.push(ctrl(AppKeyCode::Char('r')));

    let argv = TuiApp::from_command(command)
        .with_runtime(ScriptedRuntime::new(events))
        .run()
        .expect("app run should succeed")
        .expect("run should return argv");
    let argv = argv_to_strings(argv);

    assert_eq!(
        argv,
        vec![
            "tool".to_string(),
            "--include".to_string(),
            "alpha".to_string(),
            "--include".to_string(),
            "beta".to_string(),
        ]
    );
}

#[test]
fn grouped_multi_value_input_round_trips_through_the_real_app_loop() {
    let command = Command::new("tool").arg(Arg::new("pair").long("pair").num_args(1..));
    let mut events = vec![key(AppKeyCode::Tab), key(AppKeyCode::Tab)];
    events.extend(text("alpha"));
    events.push(key(AppKeyCode::Enter));
    events.extend(text("beta"));
    events.push(ctrl(AppKeyCode::Char('r')));

    let argv = TuiApp::from_command(command)
        .with_runtime(ScriptedRuntime::new(events))
        .run()
        .expect("app run should succeed")
        .expect("run should return argv");
    let argv = argv_to_strings(argv);

    assert_eq!(
        argv,
        vec![
            "tool".to_string(),
            "--pair".to_string(),
            "alpha".to_string(),
            "beta".to_string(),
        ]
    );
}

#[test]
fn delimited_append_input_round_trips_through_the_real_app_loop() {
    let command = Command::new("tool").arg(
        Arg::new("tag")
            .long("tag")
            .action(ArgAction::Append)
            .num_args(1..)
            .value_delimiter(','),
    );
    let mut events = vec![key(AppKeyCode::Tab), key(AppKeyCode::Tab)];
    events.extend(text("alpha,beta"));
    events.push(key(AppKeyCode::Enter));
    events.extend(text("gamma"));
    events.push(ctrl(AppKeyCode::Char('r')));

    let argv = TuiApp::from_command(command)
        .with_runtime(ScriptedRuntime::new(events))
        .run()
        .expect("app run should succeed")
        .expect("run should return argv");
    let argv = argv_to_strings(argv);

    assert_eq!(
        argv,
        vec![
            "tool".to_string(),
            "--tag=alpha,beta".to_string(),
            "--tag=gamma".to_string(),
        ]
    );
}

#[test]
fn fixed_arity_append_input_preserves_grouped_occurrences() {
    let command = Command::new("tool").arg(
        Arg::new("define")
            .long("define")
            .action(ArgAction::Append)
            .num_args(2),
    );
    let mut events = vec![key(AppKeyCode::Tab), key(AppKeyCode::Tab)];
    events.extend(text("KEY1"));
    events.push(key(AppKeyCode::Enter));
    events.extend(text("VALUE1"));
    events.push(key(AppKeyCode::Enter));
    events.extend(text("KEY2"));
    events.push(key(AppKeyCode::Enter));
    events.extend(text("VALUE2"));
    events.push(ctrl(AppKeyCode::Char('r')));

    let argv = TuiApp::from_command(command)
        .with_runtime(ScriptedRuntime::new(events))
        .run()
        .expect("app run should succeed")
        .expect("run should return argv");
    let argv = argv_to_strings(argv);

    assert_eq!(
        argv,
        vec![
            "tool".to_string(),
            "--define".to_string(),
            "KEY1".to_string(),
            "VALUE1".to_string(),
            "--define".to_string(),
            "KEY2".to_string(),
            "VALUE2".to_string(),
        ]
    );
}

#[test]
fn positional_append_input_keeps_space_separated_values_in_one_occurrence() {
    let command = Command::new("tool").arg(
        Arg::new("path")
            .required(true)
            .index(1)
            .action(ArgAction::Append)
            .num_args(1..),
    );
    let mut events = vec![key(AppKeyCode::Tab), key(AppKeyCode::Tab)];
    events.extend(text("src tests"));
    events.push(ctrl(AppKeyCode::Char('r')));

    let argv = TuiApp::from_command(command)
        .with_runtime(ScriptedRuntime::new(events))
        .run()
        .expect("app run should succeed")
        .expect("run should return argv");
    let argv = argv_to_strings(argv);

    assert_eq!(
        argv,
        vec!["tool".to_string(), "src".to_string(), "tests".to_string(),]
    );
}

#[test]
fn paste_event_round_trips_through_the_real_app_loop() {
    let command = Command::new("tool").arg(Arg::new("path").long("path"));
    let events = vec![
        key(AppKeyCode::Tab),
        key(AppKeyCode::Tab),
        AppEvent::Paste("/tmp/foo".to_string()),
        ctrl(AppKeyCode::Char('r')),
    ];

    let argv = TuiApp::from_command(command)
        .with_runtime(ScriptedRuntime::new(events))
        .run()
        .expect("app run should succeed")
        .expect("run should return argv");
    let argv = argv_to_strings(argv);

    assert_eq!(
        argv,
        vec![
            "tool".to_string(),
            "--path".to_string(),
            "/tmp/foo".to_string(),
        ]
    );
}

#[test]
fn inherited_global_choice_fields_work_from_descendant_forms() {
    let command = Command::new("tool")
        .arg(
            Arg::new("color")
                .long("color")
                .global(true)
                .value_parser(["red", "green", "blue"]),
        )
        .subcommand(Command::new("admin"));
    let mut config = TuiConfig::default();
    config.start_command = Some("admin".to_string());
    let events = vec![
        key(AppKeyCode::Tab),
        key(AppKeyCode::Tab),
        key(AppKeyCode::Enter),
        key(AppKeyCode::Down),
        key(AppKeyCode::Enter),
        ctrl(AppKeyCode::Char('r')),
    ];

    let argv = TuiApp::from_command(command)
        .with_config(config)
        .with_runtime(ScriptedRuntime::new(events))
        .run()
        .expect("app run should succeed")
        .expect("run should return argv");
    let argv = argv_to_strings(argv);

    assert_eq!(
        argv,
        vec![
            "tool".to_string(),
            "--color".to_string(),
            "green".to_string(),
            "admin".to_string(),
        ]
    );
}
