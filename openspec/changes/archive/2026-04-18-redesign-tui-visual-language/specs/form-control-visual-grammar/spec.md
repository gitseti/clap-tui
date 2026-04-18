## ADDED Requirements

### Requirement: Dense command forms use a shared control family
The TUI SHALL render text fields, choice pickers, counters, toggles, optional values, and repeated values as members of one CLI-native control family with consistent label, value, metadata, and help ordering.

#### Scenario: Multiple widget types are visible together
- **WHEN** the form renders text, choice, counter, toggle, and repeated-value controls in the same workspace
- **THEN** the controls share a recognizably related container and spacing grammar
- **AND** each control still exposes an affordance pattern that identifies its interaction model before the user edits it

#### Scenario: Field metadata is visible beside the current value
- **WHEN** a field renders inherited, default, environment, or implicit metadata together with its editable value
- **THEN** the metadata appears in compact badge-like treatments that belong to the same control family
- **AND** the current editable value remains visually primary

### Requirement: Dense forms align command labels and controls
The TUI SHALL render dense command forms with a stable label column and control column so CLI-style option names remain easy to scan down the left edge while values and affordances align on the right.

#### Scenario: Form renders many option rows
- **WHEN** the selected command shows a long vertical run of options
- **THEN** option labels align within a consistent label column
- **AND** the corresponding controls align within a consistent control column
- **AND** the user can scan option names independently of current values

#### Scenario: Different widget types share one row rhythm
- **WHEN** adjacent rows mix text fields, choice pickers, counters, toggles, and repeated-value controls
- **THEN** the rows preserve a compact mostly single-line rhythm by default
- **AND** any taller presentation is reserved for overflow or exceptional content rather than the ordinary resting state

### Requirement: Form sections use lightweight framing
The TUI SHALL group dense command options with lightweight section framing that creates hierarchy without turning each group into a separate heavy panel.

#### Scenario: Form includes local and inherited fields
- **WHEN** the selected command renders both local options and inherited options
- **THEN** the workspace uses section labels, divider rules, spacing, or equivalent lightweight framing to separate those groups
- **AND** the form does not rely on nested bordered panels to create that distinction

#### Scenario: Form contains a long run of related fields
- **WHEN** a command exposes many options in sequence
- **THEN** section framing maintains a clear vertical rhythm across labels, controls, badges, and help text
- **AND** the additional framing does not crowd out editable rows with decorative chrome

### Requirement: Repeated values remain compact and scannable
The TUI SHALL render repeated-value content as compact, visually grouped items that remain easy to distinguish from a single plain-text value.

#### Scenario: Multi-value field has several entries
- **WHEN** a repeated-value field contains multiple values
- **THEN** each entered item is rendered as a compact grouped token or equivalent chip-like treatment
- **AND** the collection remains visually consistent with the broader control family

#### Scenario: Repeated-value field is empty
- **WHEN** a repeated-value field has no entered items
- **THEN** the empty state still reads as a repeated-value control rather than a generic text box
- **AND** the prompt communicates how to add the first item
