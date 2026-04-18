## ADDED Requirements

### Requirement: Repeated editors preserve row-local traversal without trapping form navigation
The TUI SHALL use `Up` and `Down` inside repeated occurrence editors to move between repeated rows when another row exists, and SHALL continue normal form traversal when the focused repeated row is already the first or last visible occurrence.

#### Scenario: User moves within a repeated editor
- **WHEN** a repeated-value field contains multiple occurrence rows and the focused row is not the last row
- **THEN** pressing `Down` moves focus to the next occurrence row in that same field
- **AND** the cursor column is preserved as closely as the next row allows

#### Scenario: User leaves the last repeated row with Down
- **WHEN** a repeated-value field is focused on its last occurrence row
- **THEN** pressing `Down` moves form selection to the next visible form field
- **AND** the user is not trapped inside the repeated editor

#### Scenario: User leaves the first repeated row with Up
- **WHEN** a repeated-value field is focused on its first occurrence row
- **THEN** pressing `Up` moves form selection to the previous visible form field
- **AND** the repeated editor does not consume the keypress as a no-op
