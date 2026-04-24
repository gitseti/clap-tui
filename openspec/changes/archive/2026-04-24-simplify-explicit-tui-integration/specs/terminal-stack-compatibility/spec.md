## ADDED Requirements

### Requirement: Default dependency resolution uses one coherent terminal stack
The default `clap-tui` dependency surface SHALL resolve to one coherent terminal stack for consumers, including one compatible `ratatui` generation and one compatible `crossterm` generation, rather than requiring downstream integrators to debug duplicated backend generations introduced by `clap-tui` itself.

#### Scenario: Default dependency tree is coherent
- **WHEN** a maintainer inspects the normal `clap-tui` dependency tree for default features
- **THEN** the tree contains one `ratatui` version line
- **THEN** the tree contains one `crossterm` version line attributable to the `clap-tui` dependency surface

### Requirement: `clap-tui` owns backend integration for its textarea dependency
`clap-tui` SHALL configure `tui-textarea` so the crate can reuse its editing widget without inheriting a second backend integration stack from that dependency.

#### Scenario: Textarea dependency is backendless
- **WHEN** a maintainer inspects the `clap-tui` manifest
- **THEN** `tui-textarea` is configured without its default backend feature set
- **THEN** the manifest enables the backendless integration mode that lets `clap-tui` keep owning terminal input and backend wiring

#### Scenario: Default features do not reintroduce a second backend path
- **WHEN** a maintainer inspects the dependency graph produced by default `clap-tui` features
- **THEN** `tui-textarea` does not pull a second backend integration path into that graph
