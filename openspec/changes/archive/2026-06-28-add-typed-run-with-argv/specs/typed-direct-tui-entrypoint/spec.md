## ADDED Requirements

### Requirement: Typed direct TUI execution can retain canonical argv
The crate SHALL expose `Tui::<T>::run_with_argv()` returning `Result<Option<TuiSubmission<T>>, TuiError>`, where a successful submission contains both the parsed value and the canonical argv used to produce it. `Tui::<T>::run()` SHALL retain its existing signature and behavior as the primary typed shortcut.

#### Scenario: Successful richer submission returns both representations
- **WHEN** a caller invokes `Tui::<T>::run_with_argv()` and the user submits a valid command
- **THEN** the result is `Ok(Some(submission))`
- **AND** `submission.command` is the parsed value of type `T`
- **AND** `submission.argv` is the canonical argv used for that parse

#### Scenario: Cancellation remains a normal optional outcome
- **WHEN** a caller invokes `Tui::<T>::run_with_argv()` and the user exits without submitting
- **THEN** the result is `Ok(None)`

#### Scenario: Existing typed shortcut remains compatible
- **WHEN** a caller invokes `Tui::<T>::run()`
- **THEN** it returns `Result<Option<T>, TuiError>` with the same submission, cancellation, and error behavior as before

#### Scenario: Clap reparsing failures remain errors
- **WHEN** canonical argv produces clap help, version, parse-display, or parsing behavior instead of a typed value
- **THEN** `run_with_argv()` returns `Err(TuiError::Clap(_))`
- **AND** it does not return a partial submission
