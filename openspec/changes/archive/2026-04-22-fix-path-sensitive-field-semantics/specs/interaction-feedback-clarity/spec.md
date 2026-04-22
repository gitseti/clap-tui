## MODIFIED Requirements

### Requirement: Required and inherited field states explain themselves
The TUI SHALL use field copy and state treatments that make effective required empty states and inherited values understandable without external documentation while keeping resting invalid states calmer than focused or summary-level errors. All required indicators, placeholder wording, label width logic, and field-level missing styling MUST derive from path-sensitive field semantics rather than raw declared `ArgModel.required`.

#### Scenario: Required repeated field is empty
- **WHEN** a required repeated-value field has no values
- **AND** derived field semantics mark the field as required for the selected command path
- **THEN** the field shows instructional empty-state text that tells the user how to add the first value
- **AND** the message reads as an action prompt rather than passive status text

#### Scenario: Required single-value text field is empty
- **WHEN** a required single-value text field has no user-provided value
- **AND** derived field semantics mark the field as required for the selected command path
- **THEN** the field presents an instructional empty-state prompt
- **AND** the prompt communicates that the user needs to enter a value
- **AND** the field does not use the strongest invalid-container treatment until the field is focused invalid or the command is in an error-summary state

#### Scenario: Required choice field is empty
- **WHEN** a required choice field is still unselected
- **AND** derived field semantics mark the field as required for the selected command path
- **THEN** the control presents an instructional empty-state prompt
- **AND** its treatment is stronger than an ordinary neutral placeholder
- **AND** the resting state stays visually calmer than a submit-level error summary

#### Scenario: Declared required field is not effectively required
- **WHEN** a visible field has declared required metadata
- **AND** derived field semantics do not mark the field as required for the selected command path
- **THEN** the field does not show required placeholder wording
- **AND** the label does not reserve or render a required marker for that field
- **AND** the field is not styled as missing solely because of declared metadata

#### Scenario: Inherited field is selected
- **WHEN** a field value is inherited from an ancestor command and the user focuses that field
- **THEN** the UI identifies the field as inherited
- **AND** the UI identifies the command or path that owns the field
- **AND** the UI explains the effect of editing in terms that match the actual ownership model
- **AND** inherited, default, environment, or implicit state markers use compact badge-like treatments instead of sentence-length metadata when space allows
- **AND** the inherited indicator remains visually secondary to the editable value

#### Scenario: Inherited field is inactive on selected path
- **WHEN** a visible inherited field is neutral or disabled because of the selected subcommand path
- **THEN** the UI presents the field as inactive rather than missing
- **AND** the UI provides a concise reason when space or focus state allows
- **AND** the field does not imply that editing creates a descendant-local override
