# path-sensitive-field-semantics Specification

## Purpose
Define how form fields derive current-path presentation semantics without competing with clap as the authority for invocation validity.

## Requirements
### Requirement: Derived field semantics describe current-path presentation
The TUI SHALL derive field-level presentation semantics for the selected command path as part of derived state. Field semantics MUST represent visibility, activity, conflict state, required presentation, editability, ownership, and an optional user-facing reason as separate dimensions. Field semantics MUST be keyed by a stable projected field instance identity, such as `(owner_path, arg_id)`, rather than by raw `arg_id` alone unless global uniqueness across projected field instances is guaranteed.

#### Scenario: Local field is active on the selected path
- **WHEN** a field is owned by the selected command path and no path rule makes it inactive
- **THEN** the derived field semantics mark the field visible and active
- **AND** the field remains editable when its widget type supports editing

#### Scenario: Inherited field remains visible with ownership
- **WHEN** a descendant command view includes a field owned by an ancestor command
- **THEN** the derived field semantics preserve the ancestor owner path
- **AND** the field can be presented as inherited without losing its storage ownership

#### Scenario: Projected field identity includes ownership
- **WHEN** field semantics are looked up for rendering, layout, navigation, or editing
- **THEN** the lookup identifies the projected field instance
- **AND** fields are not conflated solely because they share an argument id string

#### Scenario: Hidden field is excluded from interaction projections
- **WHEN** derived field semantics mark a field hidden
- **THEN** the field reserves no form layout space
- **AND** the field is not focusable
- **AND** the field is excluded from hit-testing and invalid-field navigation

### Requirement: Effective required presentation is path-sensitive
The TUI SHALL derive required presentation from selected path semantics rather than raw declared `ArgModel.required`. Declared required metadata MAY contribute to `required_badge`, but it MUST NOT be treated as authoritative when command-path rules make the field non-required for the current invocation.

#### Scenario: Selected subcommand negates ancestor requirements
- **WHEN** an ancestor command declares a required argument
- **AND** a descendant subcommand is selected through a command whose parser rules negate requirements when a subcommand is present
- **THEN** the ancestor argument remains allowed to appear in the descendant form when otherwise visible
- **AND** the derived semantics do not set `required_badge` for that ancestor argument

#### Scenario: Local required argument remains required
- **WHEN** the selected command owns a required argument and no selected-path rule negates that requirement
- **THEN** the derived semantics set `required_badge` for that argument

### Requirement: Ancestor subcommand conflicts distinguish potential and actual conflict
The TUI SHALL distinguish a potential path conflict from an actual validation conflict for ancestor-owned args affected by `args_conflicts_with_subcommands`.

#### Scenario: Untouched ancestor argument conflicts with selected subcommand path
- **WHEN** an ancestor command has `args_conflicts_with_subcommands` enabled
- **AND** a descendant subcommand is selected
- **AND** the ancestor-owned argument has no user-authored value
- **THEN** the derived semantics may mark the field as neutral, disabled, or potentially path-conflicting
- **AND** canonical argv does not materialize a token for the untouched inactive field
- **AND** validation does not report an actual conflict for that field

#### Scenario: Disabled field keeps authored state
- **WHEN** derived field semantics mark a field disabled
- **AND** the field already has user-authored invocation state
- **THEN** disabling affects editability and presentation only
- **AND** the existing authored state is not cleared
- **AND** the authored state continues to participate in canonical argv unless the user explicitly removes it through an allowed interaction

#### Scenario: User-authored ancestor argument conflicts with selected subcommand path
- **WHEN** an ancestor command has `args_conflicts_with_subcommands` enabled
- **AND** a descendant subcommand is selected
- **AND** the user has authored a value for an ancestor-owned argument
- **THEN** canonical argv preserves the user-authored argument token
- **AND** clap validation decides whether that token conflicts with the selected subcommand
- **AND** field semantics mark an actual validation conflict only when validation projection reports the field as invalid

#### Scenario: Potential and actual conflict remain distinct
- **WHEN** a field is potentially path-conflicting but serialization diagnostics and clap validation do not report a field-linked error
- **THEN** the field semantics do not mark an actual validation conflict
- **AND** the UI does not style the field as a validation error solely because a potential conflict exists

### Requirement: Field semantics are shared by UI projections
The TUI SHALL use the same derived field semantics for all form UI projections that depend on current-path field meaning.

#### Scenario: Required indicators use the shared semantics
- **WHEN** the form renders labels, placeholders, label widths, and missing-required visual states
- **THEN** each surface uses the field's derived `required_badge` value
- **AND** no surface uses raw `ArgModel.required` as current-path required truth

#### Scenario: Navigation follows semantic field state
- **WHEN** focus order, hit testing, or invalid-field navigation needs to decide whether a visible field is actionable or invalid
- **THEN** it uses the same derived field semantics consumed by rendering

#### Scenario: Field errors come from validation projection
- **WHEN** field-level error styling is rendered
- **THEN** the error source is a serialization diagnostic or clap-validation projection
- **AND** field semantics do not independently create validation errors
