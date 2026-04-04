## ADDED Requirements

### Requirement: Validation summary links errors back to fields
The TUI SHALL present validation summaries in the same top-to-bottom order as the invalid fields in the current form and SHALL preserve a clear visual link between the summary and those fields.

#### Scenario: Multiple invalid fields are present
- **WHEN** the current command has more than one invalid field
- **THEN** the validation summary lists or summarizes those errors in the same order the fields appear in the form
- **AND** the first invalid field uses the same error treatment family as the summary

#### Scenario: User returns to a long invalid form
- **WHEN** the form is long enough that not all invalid fields are simultaneously visible
- **THEN** the UI identifies which invalid field is the next correction target
- **AND** the user does not need to infer correction order from the raw option names alone

## MODIFIED Requirements

### Requirement: Choice and counter widgets expose accurate interaction hints
The TUI SHALL present control affordances and inline hints that match each widget's real interaction model and make different widget types visually distinguishable before interaction.

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
- **AND** the UI explains that editing will create or change the effective local override behavior that matches the app's actual state model
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
