use clap::{Arg, ArgAction, Command};

use super::scripted::{ObservedFrame, ScriptedHarness, ScriptedOutcome, ScriptedRun, ScriptedStep};
use crate::TuiConfig;
use crate::input::HoverTarget;
use crate::runtime::{AppEvent, AppKeyCode, AppKeyEvent, AppKeyModifiers};

fn last_frame(run: &ScriptedRun) -> &ObservedFrame {
    run.frames.last().expect("observed frame")
}

#[test]
fn scripted_harness_observes_intermediate_frames_before_cancellation() {
    let command = Command::new("tool").arg(Arg::new("path").long("path"));

    let run = ScriptedHarness::new(command)
        .script([
            ScriptedStep::expect_text("Ctrl+R Run"),
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::paste("/tmp/demo"),
            ScriptedStep::expect_text("/tmp/demo"),
            ScriptedStep::ctrl(AppKeyCode::Char('c')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(run.outcome, ScriptedOutcome::Cancelled);
    assert!(run.frames.len() >= 3);
    assert!(
        run.frames
            .iter()
            .any(|frame| frame.text.contains("/tmp/demo"))
    );
}

#[test]
fn scripted_harness_returns_successful_run_outcome_with_raw_event_steps() {
    let command = Command::new("tool").arg(Arg::new("path").long("path"));

    let run = ScriptedHarness::new(command)
        .script([
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::raw(AppEvent::Paste("/tmp/raw".to_string())),
            ScriptedStep::ctrl(AppKeyCode::Char('r')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(
        run.outcome,
        ScriptedOutcome::Run(vec![
            "tool".to_string(),
            "--path".to_string(),
            "/tmp/raw".to_string(),
        ])
    );
}

#[test]
fn scripted_harness_returns_cancellation_outcome_deterministically() {
    let run = ScriptedHarness::new(Command::new("tool"))
        .script([ScriptedStep::ctrl(AppKeyCode::Char('c'))])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(run.outcome, ScriptedOutcome::Cancelled);
    assert_eq!(run.frames.len(), 1);
}

#[test]
fn semantic_footer_helper_targets_current_footer_layout() {
    let run = ScriptedHarness::new(Command::new("tool"))
        .with_size(120, 24)
        .with_config(TuiConfig::default())
        .script([
            ScriptedStep::click_footer(HoverTarget::Search),
            ScriptedStep::type_text("build"),
            ScriptedStep::expect_text("build"),
            ScriptedStep::ctrl(AppKeyCode::Char('c')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(run.outcome, ScriptedOutcome::Cancelled);
    assert!(last_frame(&run).text.contains("build"));
}

#[test]
fn semantic_dropdown_helper_targets_rendered_popup_layout() {
    let command = Command::new("tool")
        .arg(Arg::new("first").long("first"))
        .arg(Arg::new("second").long("second"))
        .arg(
            Arg::new("color")
                .long("color")
                .value_parser(["red", "green", "blue"]),
        );

    let run = ScriptedHarness::new(command)
        .with_size(60, 16)
        .script([
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::key(AppKeyCode::Down),
            ScriptedStep::key(AppKeyCode::Down),
            ScriptedStep::open_dropdown("color"),
            ScriptedStep::select_dropdown_option("red"),
            ScriptedStep::ctrl(AppKeyCode::Char('r')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(
        run.outcome,
        ScriptedOutcome::Run(vec![
            "tool".to_string(),
            "--color".to_string(),
            "red".to_string(),
        ])
    );
}

#[test]
fn happy_path_scripted_flow_mixes_navigation_editing_and_run() {
    let command = Command::new("tool")
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["debug", "release"]),
        )
        .arg(Arg::new("name").long("name").required(true));

    let run = ScriptedHarness::new(command)
        .script([
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::raw(AppEvent::Key(AppKeyEvent::new(
                AppKeyCode::Char(' '),
                AppKeyModifiers::default(),
            ))),
            ScriptedStep::key(AppKeyCode::Down),
            ScriptedStep::key(AppKeyCode::Enter),
            ScriptedStep::key(AppKeyCode::Down),
            ScriptedStep::key(AppKeyCode::Enter),
            ScriptedStep::key(AppKeyCode::Down),
            ScriptedStep::type_text("demo"),
            ScriptedStep::ctrl(AppKeyCode::Char('r')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(
        run.outcome,
        ScriptedOutcome::Run(vec![
            "tool".to_string(),
            "--verbose".to_string(),
            "--mode".to_string(),
            "release".to_string(),
            "--name".to_string(),
            "demo".to_string(),
        ])
    );
}

#[test]
fn invalid_scripted_flow_blocks_run_and_keeps_feedback_visible() {
    let command = Command::new("tool").arg(Arg::new("name").long("name").required(true));

    let run = ScriptedHarness::new(command)
        .script([
            ScriptedStep::click_footer(HoverTarget::Run),
            ScriptedStep::ctrl(AppKeyCode::Char('c')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(run.outcome, ScriptedOutcome::Cancelled);
    assert!(
        last_frame(&run)
            .text
            .contains("Missing required argument: --name")
    );
    assert!(run.frames.len() >= 2);
}

#[test]
fn keyboard_scripted_flow_keeps_preview_and_run_alignment_after_resize() {
    let command = Command::new("tool")
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["debug", "release"]),
        )
        .arg(Arg::new("name").long("name").required(true));

    let run = ScriptedHarness::new(command)
        .script([
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::key(AppKeyCode::Tab),
            ScriptedStep::key(AppKeyCode::Enter),
            ScriptedStep::select_dropdown_option("release"),
            ScriptedStep::key(AppKeyCode::Down),
            ScriptedStep::type_text("demo"),
            ScriptedStep::expect_text("tool --mode release --name demo"),
            ScriptedStep::resize(100, 32),
            ScriptedStep::expect_text("tool --mode release --name demo"),
            ScriptedStep::ctrl(AppKeyCode::Char('r')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(
        run.outcome,
        ScriptedOutcome::Run(vec![
            "tool".to_string(),
            "--mode".to_string(),
            "release".to_string(),
            "--name".to_string(),
            "demo".to_string(),
        ])
    );
    assert!(
        last_frame(&run)
            .text
            .contains("tool --mode release --name demo")
    );
}

#[test]
fn invalid_scripted_flow_keeps_validation_alignment_after_resize() {
    let command = Command::new("tool")
        .arg(Arg::new("a").long("a").required(true))
        .arg(Arg::new("b").long("b").required(true));

    let run = ScriptedHarness::new(command)
        .with_size(120, 24)
        .script([
            ScriptedStep::click_footer(HoverTarget::Run),
            ScriptedStep::expect_text("Missing required arguments"),
            ScriptedStep::click_input("a"),
            ScriptedStep::type_text("alpha"),
            ScriptedStep::expect_text("tool --a alpha"),
            ScriptedStep::resize(100, 30),
            ScriptedStep::click_footer(HoverTarget::Run),
            ScriptedStep::expect_text("Missing required argument: --b"),
            ScriptedStep::ctrl(AppKeyCode::Char('c')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(run.outcome, ScriptedOutcome::Cancelled);
    assert!(
        last_frame(&run)
            .text
            .contains("Missing required argument: --b")
    );
    assert!(last_frame(&run).text.contains("tool --a alpha"));
}

#[test]
fn mouse_oriented_scripted_flow_uses_layout_derived_dropdown_and_footer_hits() {
    let command = Command::new("tool").arg(
        Arg::new("color")
            .long("color")
            .value_parser(["red", "green", "blue"]),
    );

    let run = ScriptedHarness::new(command)
        .script([
            ScriptedStep::open_dropdown("color"),
            ScriptedStep::select_dropdown_option("blue"),
            ScriptedStep::click_footer(HoverTarget::Run),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(
        run.outcome,
        ScriptedOutcome::Run(vec![
            "tool".to_string(),
            "--color".to_string(),
            "blue".to_string(),
        ])
    );
}

#[test]
fn resize_scripted_flow_triggers_an_additional_observed_redraw() {
    let run = ScriptedHarness::new(Command::new("tool"))
        .script([
            ScriptedStep::resize(120, 40),
            ScriptedStep::ctrl(AppKeyCode::Char('c')),
        ])
        .run()
        .expect("scripted run should succeed");

    assert_eq!(run.outcome, ScriptedOutcome::Cancelled);
    assert_eq!(run.frames.len(), 2);
    assert_eq!(
        run.frames[0].snapshot.layout.footer,
        run.frames[1].snapshot.layout.footer
    );
}
