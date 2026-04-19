## MODIFIED Requirements

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
