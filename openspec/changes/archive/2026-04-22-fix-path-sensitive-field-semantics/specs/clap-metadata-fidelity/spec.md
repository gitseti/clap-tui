## ADDED Requirements

### Requirement: Declared metadata does not override clap validation
The TUI SHALL treat raw clap metadata extracted into `ArgModel` as declared parser metadata, not as authoritative current-path validation state. Static metadata MAY influence derived field presentation, but it MUST NOT create validation errors after canonical argv has been accepted by clap.

#### Scenario: Static required metadata is not a validation fallback after clap success
- **WHEN** canonical argv serialization succeeds
- **AND** clap validation accepts the canonical argv
- **AND** a visible field has declared `ArgModel.required` metadata
- **THEN** the derived validation state remains valid
- **AND** the field does not receive a missing-required validation error from static metadata

## MODIFIED Requirements

### Requirement: Validation feedback appears inline in the form
The TUI SHALL surface field-linked validation feedback directly in the form when serialization diagnostics or clap validation can attribute an error to one or more arguments, including required groups represented through composite clap references. Inline validation errors MUST come only from serialization diagnostics or clap validation projection.

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

#### Scenario: Metadata-only requiredness is presentation, not validation
- **WHEN** a field is declared required in clap metadata
- **AND** clap validation does not report that field as missing for the current canonical argv
- **THEN** the validation adapter does not mark that field invalid
- **AND** any required-looking UI treatment comes from derived field semantics rather than validation errors
