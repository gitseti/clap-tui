## MODIFIED Requirements

### Requirement: Dense forms establish a scannable information hierarchy
The TUI SHALL visually distinguish field labels, editable values, metadata badges, help text, and default-derived state so dense forms can be scanned without reading each line in order even when different widgets share one control family.

#### Scenario: Field renders label, value, metadata, and help
- **WHEN** a form field shows a label, current value, inherited or default metadata, and descriptive help
- **THEN** the editable value is visually more prominent than the label
- **AND** the label is visually more prominent than descriptive help text
- **AND** metadata badges remain compact and visually secondary to the editable value

#### Scenario: Default-derived value is shown in a control
- **WHEN** a visible field displays an untouched default-derived, environment-derived, or otherwise non-user-provided value
- **THEN** that value uses a muted treatment distinct from user-entered text
- **AND** the muted treatment remains readable inside the control

#### Scenario: User-provided value replaces default-derived state
- **WHEN** the user enters or confirms a value for a field that previously showed a muted default-derived state
- **THEN** the control promotes that value to the primary input-text treatment
- **AND** the promoted treatment remains in place after focus moves away

#### Scenario: Long form renders multiple sections
- **WHEN** the form shows multiple argument groups or long vertical runs of fields
- **THEN** section heading, control, metadata, help text, and spacing follow a consistent vertical rhythm
- **AND** adjacent sections remain distinguishable without introducing extra chrome or blank filler rows
