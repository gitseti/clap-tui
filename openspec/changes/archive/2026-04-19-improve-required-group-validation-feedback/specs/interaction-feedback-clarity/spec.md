## MODIFIED Requirements

### Requirement: Validation summary links errors back to fields
The TUI SHALL present validation summaries in the same top-to-bottom order as the invalid fields in the current form, SHALL preserve a clear visual link between the summary and those fields, and SHALL provide a concrete next correction target for missing required groups.

#### Scenario: Multiple invalid fields are present
- **WHEN** the current command has more than one invalid field
- **THEN** the validation summary lists or summarizes those errors in the same order the fields appear in the form
- **AND** the first invalid field uses the same error treatment family as the summary

#### Scenario: Missing required group is summarized actionably
- **WHEN** the current command is invalid because none of the members of a required group have been selected
- **THEN** the validation summary uses explicit corrective wording that identifies the available choices
- **AND** the summary remains visually linked to the member fields marked invalid for that group
- **AND** the summary does not degrade to an empty or purely generic invalid-state message

#### Scenario: User returns to a long invalid form
- **WHEN** the form is long enough that not all invalid fields are simultaneously visible
- **THEN** the UI identifies which invalid field is the next correction target
- **AND** the user does not need to infer correction order from the raw option names alone

#### Scenario: User activates the validation summary for a missing required group
- **WHEN** the user invokes correction navigation from a validation summary describing a missing required group
- **THEN** focus moves to the first visible member field associated with that required group
- **AND** the form scroll position updates as needed to reveal that correction target
