## ADDED Requirements

### Requirement: Parser-backed execution is bound to the rendered clap schema
The crate SHALL provide a parser-backed execution path that is tied to the same clap schema used to generate the TUI. Callers MUST NOT be required to supply an unrelated parser type after the interactive session has already rendered.

#### Scenario: Typed parser execution uses the same schema that built the TUI
- **WHEN** a caller constructs a parser-bound TUI application for a specific `Parser` type
- **THEN** running the parser-backed execution path parses argv with that same bound parser type
- **THEN** the execution path cannot silently drift to a different clap schema

### Requirement: Untyped applications retain only schema-safe execution paths
Untyped TUI application construction SHALL keep execution APIs limited to schema-safe forms such as argv output or `ArgMatches` handling, or otherwise guide callers to the parser-bound path.

#### Scenario: Command-based app does not encourage mismatched parser execution
- **WHEN** a caller constructs a TUI from a raw `clap::Command`
- **THEN** the available parser-execution guidance and API surface do not require supplying an arbitrary parser type after rendering
- **THEN** callers are directed toward argv-based, `ArgMatches`-based, or explicitly bound parser execution flows
