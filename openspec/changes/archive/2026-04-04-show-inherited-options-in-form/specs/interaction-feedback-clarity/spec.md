## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Required and inherited field states explain themselves
The TUI SHALL use field copy and state treatments that make required empty states and inherited values understandable without external documentation.

#### Scenario: Required repeated field is empty
- **WHEN** a required repeated-value field has no values
- **THEN** the field shows instructional empty-state text that tells the user how to add the first value
- **AND** the message reads as an action prompt rather than passive status text

#### Scenario: Required single-value text field is empty
- **WHEN** a required single-value text field has no user-provided value
- **THEN** the field presents an instructional empty-state prompt
- **AND** the prompt communicates that the user needs to enter a value

#### Scenario: Required choice field is empty
- **WHEN** a required choice field is still unselected
- **THEN** the control presents an instructional empty-state prompt
- **AND** its treatment is stronger than an ordinary neutral placeholder

#### Scenario: Inherited field is selected
- **WHEN** a field value is inherited from an ancestor command and the user focuses that field
- **THEN** the UI identifies the field as inherited
- **AND** the UI identifies the command or path that owns the field
- **AND** the UI explains the effect of editing in terms that match the actual ownership model
- **AND** the inherited indicator remains visually secondary to the editable value
