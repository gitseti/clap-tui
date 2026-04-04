## Why

`clap-tui` now has enough surface area that a few usability issues compound into a rough experience: large command trees can lose the active selection, fixed chrome consumes too much space on smaller terminals, and several interactive states do not communicate their purpose clearly enough. These issues are visible in the core navigation and feedback loops, so fixing them now will make the TUI feel more dependable before more features land on top of the current UI.

The review also exposed a gap between what the interface suggests and how it actually behaves. Search is not part of the normal focus cycle, multi-select dropdowns rely on undisclosed keyboard rules, command context is easy to lose in nested CLIs, and error/status surfaces do not establish clear visual priority. Bringing those behaviors back into alignment is important for both first-time discoverability and day-to-day efficiency.

## What Changes

- Add sidebar scrolling and selection visibility rules so deep command trees remain navigable with keyboard and mouse input.
- Rework terminal layout behavior so header, preview, footer, and sidebar sizing adapt better to narrow or short terminals instead of starving the form area.
- Restore stronger command orientation in the main workspace, including clearer nested-command context and more explicit preview affordances.
- Improve interaction discoverability for search focus, multi-select dropdowns, counters, required-field empty states, inherited-field behavior, preview copy, and dropdown dismissal behavior.
- Strengthen status and feedback styling so primary actions, validation summaries, required choice fields, inherited badges, error toasts, and success/error states are easier to distinguish at a glance.
- Refine visual hierarchy in the sidebar and footer so tree depth is easier to scan and global status does not read like a wall of equally weighted hints.

## Capabilities

### New Capabilities

- `sidebar-navigation-visibility`: The sidebar keeps the active command visible and supports scrolling for larger command trees.
- `adaptive-terminal-layout`: The main screen adapts its chrome and status surfaces to preserve usable form space across terminal sizes.
- `command-context-orientation`: The workspace surfaces enough command hierarchy and preview context for users to stay oriented inside nested CLIs.
- `interaction-feedback-clarity`: Interactive controls, inherited-state markers, empty states, and feedback surfaces communicate their behavior, severity, and action priority clearly.

### Modified Capabilities

- None.

## Impact

- Affected code will include `crates/clap-tui/src/ui/`, `crates/clap-tui/src/controller/`, `crates/clap-tui/src/update/`, `crates/clap-tui/src/frame_snapshot.rs`, and `crates/clap-tui/src/input.rs`.
- This change is expected to add or revise layout state, navigation state, and interaction hints, but it should not require new external dependencies.
- Test coverage will need to expand around sidebar scrolling, tree readability, small-terminal layout behavior, command-context rendering, required-field empty states, inherited-value communication, dropdown interaction rules, preview discoverability, and toast/footer feedback presentation.
