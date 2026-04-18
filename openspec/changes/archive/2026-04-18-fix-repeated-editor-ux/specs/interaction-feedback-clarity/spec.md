## ADDED Requirements

### Requirement: Repeated-value editors keep external row controls and distinct visible rows
The TUI SHALL render repeated-value inputs as distinct row editors with row-scoped controls outside the textarea area, and SHALL preserve those row boundaries when the field is partially clipped by the form viewport.

#### Scenario: Non-terminal repeated row is rendered
- **WHEN** a repeated-value field shows a row that has a remove action but is not the last row
- **THEN** the row reserves a right-side control gutter outside the textarea border
- **AND** the remove control is centered within that gutter
- **AND** the textarea border remains visually separate from the control

#### Scenario: Last repeated row is rendered
- **WHEN** a repeated-value field shows its last visible occurrence row
- **THEN** the row reserves a right-side control gutter outside the textarea border
- **AND** both remove and add controls render in that external gutter rather than inside the textarea area

#### Scenario: Repeated editor is partially clipped by scrolling
- **WHEN** the form viewport shows only part of a repeated-value field because the form is scrolled
- **THEN** each fully visible repeated row still renders as its own row editor
- **AND** the visible controls continue to match those rows
- **AND** the field does not collapse into a single merged text block
