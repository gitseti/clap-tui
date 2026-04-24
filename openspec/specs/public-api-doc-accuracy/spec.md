## MODIFIED Requirements

### Requirement: Entry-point docs describe observable run semantics
The public documentation for `clap-tui` entry points SHALL describe behavior that callers can actually observe from the exported API. Documentation for the primary 0.1.0 entry point SHALL correctly describe the `Tui::<T>::run()` contract, including how cancellation is surfaced, when clap parsing errors can occur, and that the API does not print automatically or call `std::process::exit`.

#### Scenario: User reads `run` documentation
- **WHEN** a user reads the docs for `Tui::<T>::run()`
- **THEN** the docs describe the `Result<Option<T>, TuiError>` contract accurately
- **THEN** they explain that `Ok(None)` means cancellation only
- **THEN** they explain that `Err(TuiError::Clap(_))` includes help, version, and parse-display flows after argv exists

#### Scenario: User compares entry points
- **WHEN** a user reads the public docs for the main exported entry points
- **THEN** they can tell that `Tui::<T>::run()` is the primary explicit integration surface for 0.1.0
- **THEN** they do not have to infer that launcher interception or macros are still the recommended path

### Requirement: Supported extension points are described consistently
Public documentation SHALL describe intentionally exported runtime and customization seams consistently across the README, crate docs, and item docs. Exported runtime event and integration types SHALL be described in concise user-facing language and SHALL not crowd out the primary `Tui::<T>::run()` guidance.

#### Scenario: User evaluates runtime customization
- **WHEN** a user reads the public docs for runtime-related exported types
- **THEN** the wording identifies them as advanced integration seams
- **THEN** it does not contradict the crate-level description of `Tui::<T>::run()` as the primary public API surface
- **THEN** it uses terminology that is easier to understand than repeated architectural labels alone
