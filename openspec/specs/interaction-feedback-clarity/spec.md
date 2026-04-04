# interaction-feedback-clarity Specification

## Purpose
Make controls, validation, and feedback surfaces explain themselves clearly enough that users can navigate and correct state without hidden interaction knowledge.
## Requirements
### Requirement: Focus traversal includes the search field
The TUI SHALL let users reach the search field through the normal focus traversal model in addition to direct shortcuts or pointer input.

#### Scenario: User cycles focus with Tab
- **WHEN** the user advances focus using the standard focus-cycle control
- **THEN** focus advances in the order Sidebar -> Search -> Form

#### Scenario: User cycles focus with BackTab
- **WHEN** the user reverses focus using the reverse focus-cycle control
- **THEN** focus moves in the order Form -> Search -> Sidebar

#### Scenario: Footer advertises focus behavior
- **WHEN** the footer renders a focus hint
- **THEN** the hint matches the actual focus traversal behavior implemented by the TUI

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

### Requirement: Descendant forms expose invocation-relevant inherited options
The TUI SHALL show invocation-relevant inherited options in the active form panel when a descendant command is selected, even when those options are owned by an ancestor command.

#### Scenario: Preview includes an inherited ancestor option
- **WHEN** the selected descendant command inherits an option that contributes to the generated invocation
- **THEN** the active form panel shows a corresponding editable field for that option
- **AND** the user does not need to navigate to the ancestor command merely to inspect or edit that invocation-relevant setting

#### Scenario: Multiple ancestors contribute inherited options
- **WHEN** the selected descendant command inherits options from more than one ancestor level
- **THEN** the form panel groups or labels those fields in a way that identifies which ancestor owns each option
- **AND** the selected command's own local fields remain visually primary

### Requirement: Inherited option ownership and edit scope are explicit
The TUI SHALL explain where an inherited option comes from and what effect editing it has from the current descendant command view.

#### Scenario: Inherited option is visible in a descendant form
- **WHEN** the form renders an inherited option
- **THEN** the UI identifies the owning ancestor command or path for that option
- **AND** the inherited indicator remains visually secondary to the editable value itself

#### Scenario: User focuses an inherited option
- **WHEN** the user selects or focuses an inherited option from a descendant command
- **THEN** the UI explains the effect of editing that field in terms that match the app's actual state model
- **AND** the explanation does not imply a descendant-local override unless the system truly creates one

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

### Requirement: Dropdown dismissal does not require a sacrificial click
The TUI SHALL allow users to click another actionable surface while a dropdown is open without forcing a separate close-only click first.

#### Scenario: User clicks another field while a dropdown is open
- **WHEN** the user left-clicks a different form field outside the open dropdown
- **THEN** the dropdown closes
- **AND** the clicked field receives the same interaction that would have occurred if the dropdown had not been open

#### Scenario: User clicks search or sidebar while a dropdown is open
- **WHEN** the user clicks the search field or a sidebar row outside the open dropdown
- **THEN** the dropdown closes
- **AND** the clicked search or sidebar target receives the same interaction that would have occurred if the dropdown had not been open

#### Scenario: User clicks a non-form action while a dropdown is open
- **WHEN** the user clicks a footer action or preview action outside the open dropdown
- **THEN** the dropdown closes
- **AND** the clicked action executes in that same interaction

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
