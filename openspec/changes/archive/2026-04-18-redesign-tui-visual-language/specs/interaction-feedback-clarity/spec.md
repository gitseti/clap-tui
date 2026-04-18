## MODIFIED Requirements

### Requirement: Choice and counter widgets expose accurate interaction hints
The TUI SHALL present control affordances and inline hints that match each widget's real interaction model, and SHALL make the redesigned choice, counter, toggle, and repeated-value widgets visually distinguishable before interaction.

#### Scenario: Multi-select dropdown is focused
- **WHEN** a multi-select choice widget is selected or opened
- **THEN** the UI shows that toggling choices and finishing selection use explicit controls
- **AND** the instructions distinguish multi-select behavior from single-select behavior
- **AND** the widget uses a visible affordance that identifies it as a choice picker rather than a plain text field

#### Scenario: Counter widget is rendered
- **WHEN** a counter field is visible
- **THEN** the field uses stepper-oriented affordances rather than dropdown-oriented affordances
- **AND** the inline hint matches the increment and decrement controls the widget supports
- **AND** the counter remains visually distinct from ordinary choice controls even when not focused

#### Scenario: Boolean or multi-value widget is rendered
- **WHEN** the UI renders a boolean toggle or a multi-value text field
- **THEN** the control uses an affordance pattern that identifies its interaction model before the user edits it
- **AND** entered multi-value items remain visually distinguishable from a single plain text value

### Requirement: Required and inherited field states explain themselves
The TUI SHALL use field copy and state treatments that make required empty states and inherited values understandable without external documentation while keeping resting invalid states calmer than focused or summary-level errors.

#### Scenario: Required repeated field is empty
- **WHEN** a required repeated-value field has no values
- **THEN** the field shows instructional empty-state text that tells the user how to add the first value
- **AND** the message reads as an action prompt rather than passive status text

#### Scenario: Required single-value text field is empty
- **WHEN** a required single-value text field has no user-provided value
- **THEN** the field presents an instructional empty-state prompt
- **AND** the prompt communicates that the user needs to enter a value
- **AND** the field does not use the strongest invalid-container treatment until the field is focused invalid or the command is in an error-summary state

#### Scenario: Required choice field is empty
- **WHEN** a required choice field is still unselected
- **THEN** the control presents an instructional empty-state prompt
- **AND** its treatment is stronger than an ordinary neutral placeholder
- **AND** the resting state stays visually calmer than a submit-level error summary

#### Scenario: Inherited field is selected
- **WHEN** a field value is inherited from an ancestor command and the user focuses that field
- **THEN** the UI identifies the field as inherited
- **AND** the UI identifies the command or path that owns the field
- **AND** the UI explains the effect of editing in terms that match the actual ownership model
- **AND** inherited, default, environment, or implicit state markers use compact badge-like treatments instead of sentence-length metadata when space allows
- **AND** the inherited indicator remains visually secondary to the editable value

### Requirement: Feedback styling distinguishes severity and priority
The TUI SHALL visually distinguish primary actions, passive hints, validation summaries, success feedback, warning-like inherited or implicit metadata, and error feedback.

#### Scenario: Primary action is rendered alongside secondary actions
- **WHEN** the footer renders the primary Run action together with secondary actions
- **THEN** the Run action uses a stronger visual treatment than secondary actions
- **AND** that priority remains distinguishable through more than color alone

#### Scenario: Validation summary is rendered
- **WHEN** the current command has a validation summary
- **THEN** the summary uses a stronger feedback treatment than passive footer hints
- **AND** it remains visually identifiable as status or error feedback
- **AND** its treatment corresponds to the invalid fields it summarizes

#### Scenario: Success toast is rendered
- **WHEN** the app displays a success toast
- **THEN** the toast uses a success-oriented treatment distinct from both passive hints and error toasts

#### Scenario: Error toast is rendered
- **WHEN** the app displays an error toast
- **THEN** the toast uses error-oriented styling rather than the neutral default border treatment

#### Scenario: Selected default option appears in a dropdown
- **WHEN** the currently highlighted dropdown row is also a default option
- **THEN** the highlighted row still uses a readable selected-state foreground treatment

#### Scenario: State hierarchy is viewed in any theme
- **WHEN** the UI renders primary actions, inherited badges, validation summaries, success feedback, warning-like metadata, or error surfaces in any supported theme
- **THEN** their hierarchy remains distinguishable through text, border, emphasis, label, or placement cues rather than color alone

#### Scenario: Footer actions are rendered with low-priority hints
- **WHEN** the footer renders primary and secondary actions alongside search, focus, or help hints
- **THEN** the actions use compact keycap-like treatments or equivalent high-contrast utility affordances
- **AND** the low-priority hints remain visually quieter than those actions
