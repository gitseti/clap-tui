## ADDED Requirements

### Requirement: Occurrence-aware repeated value editing
The TUI SHALL allow users to edit repeated and multi-value arguments without collapsing them into a single newline-encoded text blob. The editing model MUST preserve the difference between repeated occurrences and multiple values supplied within one occurrence when that distinction affects clap parsing.

#### Scenario: Append-style option keeps repeated occurrences
- **WHEN** a command defines an append-style option and the user adds the option multiple times
- **THEN** the invocation state preserves each occurrence in order
- **THEN** preview and run use that preserved occurrence order

#### Scenario: Multi-value input preserves grouped occurrence shape
- **WHEN** a command defines an argument that accepts multiple values in one occurrence
- **THEN** the user can edit those values as one occurrence rather than only as repeated single-value entries
- **THEN** argv synthesis preserves the grouped occurrence shape required by clap

### Requirement: Rich action arguments are directly editable
The TUI SHALL provide direct editing flows for clap actions that are not well represented by a single text field, including count flags and optional-value flags.

#### Scenario: Count flag is incremented interactively
- **WHEN** a command defines an argument with count semantics
- **THEN** the user can increase and decrease the occurrence count through the form
- **THEN** preview and run emit the correct number of flag occurrences

#### Scenario: Optional-value flag can be present with or without a value
- **WHEN** a command defines a flag-like argument whose value is optional
- **THEN** the user can choose whether the argument is absent, present without a value, or present with a value
- **THEN** preview and validation reflect the selected state accurately

#### Scenario: Optional-value flag can apply default-missing values
- **WHEN** a command defines an optional-value flag with one or more default-missing values
- **THEN** the user can choose the present-without-explicit-value state that triggers those implicit values
- **THEN** the TUI distinguishes that state from both an absent flag and a user-supplied explicit value

### Requirement: Inherited global values remain visible and attributable
The TUI SHALL show inherited global arguments inside descendant subcommand forms without duplicating storage. The UI MUST indicate when a value is inherited from an owning command rather than defined locally.

#### Scenario: Descendant form shows inherited global input
- **WHEN** a root or ancestor command owns a global argument and the user navigates to a descendant subcommand
- **THEN** the descendant form includes that argument in its effective input set
- **THEN** the UI indicates that the value is inherited from its owning command

#### Scenario: Editing inherited global input updates the owner
- **WHEN** the user changes an inherited global argument from a descendant subcommand form
- **THEN** the stored value is updated on the owning command rather than duplicated locally
- **THEN** preview and validation use the updated global value across the selected command path
