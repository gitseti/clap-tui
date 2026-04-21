## Purpose

Define how the TUI preserves and presents clap metadata such as display order, value metadata, value sources, and validation feedback.

## Requirements

### Requirement: Form and command navigation respect clap display metadata
The TUI SHALL present arguments and subcommands using clap display metadata rather than convenience ordering. The form and sidebar MUST be able to use clap display order, headings, long help, aliases, value names, and combined labels where available.

#### Scenario: Arguments render in clap display order
- **WHEN** a command assigns display order to its arguments
- **THEN** the form renders those arguments in clap-defined order rather than alphabetic order

#### Scenario: Headings and richer labels improve discoverability
- **WHEN** an argument provides a help heading or both short and long spellings
- **THEN** the form groups the argument under the corresponding heading when headings are present
- **THEN** the label shown to the user includes enough spelling information to make the argument easy to recognize

#### Scenario: Long help is available from the form
- **WHEN** an argument provides long help that adds information beyond the short help text
- **THEN** the user can access that long help from the form without leaving the TUI

#### Scenario: Sidebar respects subcommand metadata
- **WHEN** a command defines subcommand ordering, aliases, or subcommand help-grouping metadata
- **THEN** the sidebar or command tree uses that metadata to present subcommands in a discoverable order
- **THEN** the UI exposes the relevant aliases or labels closely enough for users to recognize the intended subcommand

#### Scenario: Value names improve input placeholders
- **WHEN** an argument or command defines value-name metadata for expected inputs
- **THEN** the TUI uses that metadata to improve placeholders, labels, or contextual help
- **THEN** keyboard-driven editing remains understandable without relying on raw usage text alone

### Requirement: Choice editors reflect value-level metadata
The TUI SHALL use clap value-level metadata in choice editors when that metadata affects understanding of available values.

#### Scenario: Choice list shows per-value descriptions
- **WHEN** a possible value defines descriptive help text
- **THEN** the corresponding choice editor can surface that description to the user

#### Scenario: Hidden values are not presented as ordinary choices
- **WHEN** a possible value is hidden from help-style presentation
- **THEN** the TUI does not present it as a normal visible choice unless the interaction explicitly calls for showing hidden values

### Requirement: Value sources are surfaced clearly
The TUI SHALL distinguish between user-entered values and values sourced from defaults, environment variables, or conditional default rules. Effective values MUST be derived by parsing canonical argv with clap and inspecting clap value sources. Displaying those sources MUST NOT cause the serializer to invent argv tokens for clap-derivable values.

#### Scenario: Environment-provided value is identified as sourced
- **WHEN** an argument value comes from an environment variable
- **THEN** the form indicates that the displayed value is environment-derived
- **AND** canonical argv omits the argument unless the user explicitly emits it

#### Scenario: Conditional default is explained without pretending the user typed it
- **WHEN** clap provides a conditional default for an argument
- **THEN** the form identifies that value as default-derived rather than user-entered
- **AND** canonical argv does not materialize extra tokens solely to mirror that effective value

#### Scenario: Serialization ambiguity is distinct from validation failure
- **WHEN** invocation state cannot be serialized into unique canonical argv
- **THEN** the UI presents that as a serialization ambiguity
- **AND** it does not label the condition as a clap validation failure or derived-value source

### Requirement: Validation feedback appears inline in the form
The TUI SHALL surface field-linked validation feedback directly in the form when clap validation can attribute an error to one or more arguments, including required groups represented through composite clap references.

#### Scenario: Required field is highlighted inline
- **WHEN** clap validation reports a missing required argument that can be linked to a field
- **THEN** the corresponding form field is styled as invalid
- **THEN** the user can see the field error without relying only on the footer or toast summary

#### Scenario: Missing required group is linked to member fields
- **WHEN** clap validation reports a missing required group using a composite reference such as `<--fast|--safe>`
- **THEN** the validation adapter resolves that failure to the corresponding member fields in the active command form
- **AND** each resolved member field is marked invalid in the derived validation state
- **AND** the TUI does not fall back to a generic invalid state with no field-linked feedback

#### Scenario: Conflict or value error is attached to the relevant field
- **WHEN** clap validation reports a field-linked conflict or invalid value
- **THEN** the form highlights the referenced argument or arguments inline
- **THEN** the footer summary remains consistent with the inline error state
