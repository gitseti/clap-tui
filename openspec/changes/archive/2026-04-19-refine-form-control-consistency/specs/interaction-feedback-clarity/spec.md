## MODIFIED Requirements

### Requirement: Choice and counter widgets expose accurate interaction hints
The TUI SHALL present control affordances and inline hints that match each widget's real interaction model and make different widget types visually distinguishable before interaction even when they share a textarea-like container treatment.

#### Scenario: Multi-select dropdown is focused
- **WHEN** a multi-select choice widget is selected or opened
- **THEN** the UI shows that toggling choices and finishing selection use explicit controls
- **AND** the instructions distinguish multi-select behavior from single-select behavior
- **AND** the widget uses a visible affordance that identifies it as a choice picker rather than a plain text field

#### Scenario: Counter widget is rendered
- **WHEN** a counter field is visible
- **THEN** the field uses stepper-oriented affordances rather than dropdown-oriented or plain-text-only affordances
- **AND** the inline hint matches the increment and decrement controls the widget supports
- **AND** the counter remains visually distinct from ordinary choice controls even when not focused

#### Scenario: Boolean or optional-value widget is rendered
- **WHEN** the UI renders a boolean toggle or an optional-value flag-like field
- **THEN** the control uses an affordance pattern that identifies its interaction model before the user edits it
- **AND** entering the shared textarea-like visual family does not change the field's underlying toggle or presence semantics

## ADDED Requirements

### Requirement: Focused search exposes an editable state
The TUI SHALL make the search field look immediately editable when it receives focus through pointer or keyboard interaction.

#### Scenario: User focuses an empty search field
- **WHEN** the empty `Search commands` field receives focus
- **THEN** the placeholder copy disappears from the editable area
- **AND** the TUI shows a visible cursor position that indicates typing can begin immediately

#### Scenario: User focuses a populated search field
- **WHEN** the search field already contains a query and receives focus
- **THEN** the current query remains visible
- **AND** the field shows a visible cursor position within the editable area rather than only a focused border treatment
