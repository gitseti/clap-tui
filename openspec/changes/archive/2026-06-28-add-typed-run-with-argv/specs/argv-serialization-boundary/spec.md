## ADDED Requirements

### Requirement: Typed submissions expose the authoritative argv unchanged
When typed execution returns canonical argv, the system SHALL expose the same `Vec<OsString>` token sequence produced by the authoritative serializer and used for clap reparsing. The returned argv MUST include the executable token and MUST NOT be replaced by shell-rendered preview or clipboard text.

#### Scenario: Typed submission preserves canonical tokens
- **WHEN** `Tui::<T>::run_with_argv()` returns a successful submission
- **THEN** `submission.argv` exactly matches the canonical argv used for validation and typed reparsing
- **AND** no second serialization path reconstructs argv from the parsed value

#### Scenario: Derived clap values do not alter returned argv
- **WHEN** clap derives a default, environment, or conditional value while parsing canonical argv
- **THEN** the derived value may appear in `submission.command`
- **AND** `submission.argv` does not materialize an additional token for that derived value

#### Scenario: Returned argv remains distinct from display text
- **WHEN** a caller receives `submission.argv`
- **THEN** it receives executable `OsString` tokens including the executable token
- **AND** it does not receive POSIX or PowerShell rendered command text
