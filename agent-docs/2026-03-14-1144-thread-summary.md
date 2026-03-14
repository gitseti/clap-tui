2026-03-14 11:44
# Thread Summary

This document summarizes the full request and implementation context from the conversation so far, for future agent work in this repo.

## Project Goal
- Build a Rust + clap equivalent of Python’s `trogon`: automatically generate a TUI from an existing CLI definition/program.
- Use Ratatui with modern, idiomatic Rust (2026‑style library architecture).
- Must support keyboard + mouse input, resizing, and a polished, discoverable UX for building commands.

## Major UX/Behavior Requirements (User‑Driven)
- Two‑pane layout: command tree on the left, form on the right.
- Footer bar with `Run` + `Exit` actions; `Exit` should be `Ctrl+C` and should *not* run the command.
- Run keybinding remains `Ctrl+Enter`.
- Command preview should show the **full** assembled command (not just the command name).
- Defaults:
  - Display defaults subtly (dim) within inputs.
  - When the user starts typing, defaults should be replaced.
  - Defaults should **not** be included in the final command unless user touched the field.
- Mouse interactions:
  - Hover effects for footer actions.
  - Clicking fields should reliably focus correct input.
  - Clicking checkboxes should toggle.
  - Dropdowns should be clickable and usable (with scrolling).
- Scrolling:
  - Scrollbar should be visible and accurate; thumb should reach bottom when fully scrolled.
  - Form content must not overlap the header/command area.
- Styling:
  - Non‑minimalist; more “designed” (Bubble Tea / Textual‑like).
  - Use a cohesive palette and panel layering.
  - Clear visual hierarchy: header, sidebar, form, help, footer, preview.

## Major UX Iterations and Findings
- **Inline compact form layout** attempt (label + description inline, right‑aligned input) was rejected as looking “completely bad”. Reverted to original vertically spaced field layout.
- **Exit behavior** initially returned `TuiError::Cancelled`, but user requested normal exit with no error output; cancel should end cleanly.
- **Mouse hit testing** was flaky due to row‑math with variable heights; needed per‑field rects or content‑space hit testing.
- **Dropdown UX** needed scroll, better alignment, and click/keyboard support.
- **Scrollbar visibility** and correctness were repeatedly adjusted; thumb positioning bug at end of scroll was noted.

## Current Architecture / Implementation Notes (as of last known state)
- **State and input** live in `crates/clap-tui/src/app.rs` and `crates/clap-tui/src/input.rs`.
- **Rendering** lives in `crates/clap-tui/src/ui.rs`.
- **Spec** lives in `crates/clap-tui/src/spec.rs`.
- **Theme** lives in `crates/clap-tui/src/config.rs`.

### Input/State
- `AppState` tracks:
  - focused field index
  - scroll position
  - dropdown open state + dropdown scroll
  - hover target for footer buttons
  - touched flags for args (to suppress default values in final command)
- `Ctrl+C` exits normally (no error).
- `Ctrl+Enter` triggers command execution.

### Defaults Behavior
- Default values render dim in inputs when not touched.
- On first user input, default is replaced with typed content.
- Defaults are excluded from command unless field touched.

### Mouse Hit‑Testing
- Click handling uses content‑space hit testing in the form to map clicks to fields reliably.
- Clicks on checkboxes toggle; clicks on input should focus.

### Dropdowns
- Enums render as dropdowns.
- Open via click or Enter/Space.
- Options can be navigated via arrows; scroll if many options.

### Scrollbar
- Scrollbar uses a track and thumb; thumb size based on viewport/content.
- Visuals: track `┃`, thumb `█` for higher contrast.

### Layout
- Background panel with overall container.
- Header band for command name and description.
- Left sidebar (commands tree + search).
- Right form panel with section headers and divider line.
- Bottom command preview bar (full command line).
- Footer pill‑style hints (`Run`, `Exit`, `Search`, `Focus`).

## Outstanding Issues Reported by User (Recent)
- After revert, `--verbose` wasn’t displayed (flag rendering regression).
- Clicking on inputs sometimes only works near the top of input field (hit‑box mismatch).
- Checkbox clicking was flaky (later fixed, but regression re‑reported).
- Scrollbar thumb visibility/position still not perfect in some cases.

## Requested Deliverable Now
- Provide a durable, internal summary of the thread in `agent-docs/` for future agent work. This file is that summary.
