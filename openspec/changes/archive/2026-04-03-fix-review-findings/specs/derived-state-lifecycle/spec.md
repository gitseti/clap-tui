## ADDED Requirements

### Requirement: Rendering reuses previously derived preview and validation state
The TUI SHALL derive preview argv and validation state when relevant command input changes and MUST reuse that derived state for rendering until another relevant state transition occurs.

#### Scenario: Redraw without input mutation reuses derived state
- **WHEN** the app redraws because of resize, focus, hover, or other non-input UI changes
- **THEN** it reuses the most recent derived preview argv and validation state
- **THEN** it does not re-run full clap validation solely because a redraw occurred

### Requirement: Run uses the same derived validation contract shown in the UI
The TUI MUST keep preview, visible validation, and Run behavior aligned by using the same current derived command state for both rendering and execution gating.

#### Scenario: Run matches the latest rendered validation result
- **WHEN** the UI shows the current command state as invalid
- **THEN** invoking Run is blocked using that same invalid derived state
- **THEN** the app does not recompute a contradictory result from stale or unrelated state

