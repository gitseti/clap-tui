## MODIFIED Requirements

### Requirement: Screen chrome adapts to constrained terminal sizes
The TUI SHALL reduce the space consumed by non-form chrome when terminal height or width is constrained so the form remains the primary visible workspace, while preserving recognizable navigation, context, result, and action surfaces from the redesigned visual language.

#### Scenario: Terminal height is limited
- **WHEN** the app renders below a compact-layout budget of 20 terminal rows
- **THEN** the layout enters compact mode
- **AND** the preview consumes at most one row of content
- **AND** the form remains visible and editable without hidden focus

#### Scenario: Terminal width is limited
- **WHEN** the app renders below a compact-layout budget of 80 terminal columns
- **THEN** the layout enters compact mode
- **AND** low-priority footer hints yield space before primary actions or validation feedback
- **AND** the form remains visible and editable without hidden focus

#### Scenario: Command description is absent
- **WHEN** the selected command has no `about` text or other header content
- **THEN** the layout does not reserve empty header rows for that missing content

#### Scenario: Compact mode preserves redesigned identity
- **WHEN** the app renders in compact mode
- **THEN** the sidebar, workspace header, preview, and footer still read as distinct product surfaces
- **AND** compact mode removes decorative weight before removing the cues that identify those surfaces

#### Scenario: Compact mode preserves dense row scanability
- **WHEN** the app renders a redesigned dense form in compact mode
- **THEN** the layout preserves enough label and control alignment for rows to remain scannable
- **AND** compact mode does not fall back to a looser multi-line resting layout for ordinary controls unless content overflow requires it

### Requirement: Critical status remains visible in narrow layouts
The TUI SHALL preserve primary actions and critical validation or status feedback in narrow layouts through priority-aware placement or truncation.

#### Scenario: Validation summary appears on a narrow terminal
- **WHEN** the current command is invalid and horizontal space is limited
- **THEN** the UI still renders a visible validation summary
- **AND** primary command actions remain accessible in the same layout

#### Scenario: Low-priority hints compete for footer width
- **WHEN** there is not enough horizontal space to render every footer hint
- **THEN** low-priority hints are truncated, collapsed, or omitted before critical actions or validation feedback

#### Scenario: Main panel has few visible fields
- **WHEN** the selected command renders only a small number of visible fields
- **THEN** the layout avoids adding extra empty chrome rows around auxiliary surfaces
- **AND** the main panel preserves the available space for active content
