## ADDED Requirements

### Requirement: Dense command forms use a shared textarea-like control family
The TUI SHALL render text fields, toggles, counters, choice pickers, and optional-value fields as members of one shared textarea-like control family while preserving each widget's existing interaction semantics.

#### Scenario: Multiple widget types are visible together
- **WHEN** the form renders text inputs, flags, counters, choice pickers, and optional-value fields in the same workspace
- **THEN** those controls share a recognizably related bordered container and value presentation grammar
- **AND** each control still exposes an affordance pattern that identifies its interaction model before the user edits it

#### Scenario: Non-text widget is activated inside the shared control family
- **WHEN** the user activates a flag, counter, choice picker, or optional-value field that uses the shared control treatment
- **THEN** the field preserves the same toggle, stepper, dropdown, or presence behavior it had before the visual unification
- **AND** the shared container does not require the user to type free-form text merely to trigger that existing behavior

### Requirement: Labels and metadata stack without crowding
The TUI SHALL place compact metadata badges beneath the field label when needed so option names remain readable without sacrificing metadata visibility.

#### Scenario: Field label includes metadata badges
- **WHEN** a field renders a label together with compact metadata such as inherited, default, environment, or implicit status
- **THEN** the primary option name remains on its own label row
- **AND** the metadata badges render beneath that label in a compact secondary row

#### Scenario: Option name is long or badge text is wide
- **WHEN** a field label or badge text would otherwise crowd a single horizontal row
- **THEN** the stacked label-and-badge layout preserves readable option naming
- **AND** the control column remains aligned with neighboring fields

### Requirement: Form sections use lightweight heading rules
The TUI SHALL separate grouped fields with section labels followed by horizontal rules rather than drawing a boxed frame around the full section.

#### Scenario: Form includes more than one section
- **WHEN** the selected command renders multiple field groups such as local and inherited options
- **THEN** each section begins with its label followed by a horizontal rule
- **AND** the grouped rows below read as part of that section without left, right, or bottom panel borders

#### Scenario: Section reaches its last row
- **WHEN** the final visible row of a section is rendered
- **THEN** the form does not draw a closing section cap or terminal rule beneath that row solely to complete a box
- **AND** the next section boundary or normal form spacing provides the separation instead
