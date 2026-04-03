## 1. Validation In The Form

- [x] 1.1 Render field-linked validation errors directly in form widgets using the existing validation state.
- [x] 1.2 Add form styling and copy for required, conflict, and invalid-value error states.
- [x] 1.3 Add tests covering inline validation rendering while keeping footer summaries consistent.

## 2. Occurrence-Aware Editing

- [x] 2.1 Replace newline-only repeated-value editing with occurrence-aware editing for repeated text inputs.
- [x] 2.2 Add grouped free-text editing for multiple values within one occurrence where clap parsing distinguishes grouped values from repeated occurrences.
- [x] 2.3 Complete multi-value enum and multi-select editing semantics on top of the existing widget paths.
- [x] 2.4 Complete count-style flag controls on top of the existing widget paths.
- [x] 2.5 Complete optional-value flag semantics for absent, present-without-value, and present-with-explicit-value states; handle default-missing metadata in 3.1 and 4.3.
- [x] 2.6 Show inherited global arguments in descendant forms with clear ownership and non-duplicated storage.

## 3. Argv And Parser Fidelity

- [x] 3.1 Expand extracted spec and invocation-state metadata for the remaining parser and source gaps, including external subcommand payloads, parse-boundary rules, default-missing behavior, and conditional-default metadata.
- [x] 3.2 Preserve the distinction between repeated occurrences and grouped values in one occurrence through input state and serialization.
- [x] 3.3 Implement serializer support for `require_equals`, delimiter and terminator behavior, `dont_delimit_trailing_values`, and raw or trailing capture semantics.
- [x] 3.4 Add an explicit external-subcommand interaction flow that stores an unknown subcommand name plus trailing values without forcing it into the known command tree.
- [x] 3.5 Support command-path edge cases including required subcommands, missing-positional behavior, and argument-versus-subcommand parse boundaries.
- [x] 3.6 Add focused tests that assert both preview argv shape and clap acceptance for each supported edge case.

## 4. Metadata And Source Fidelity

- [x] 4.1 Extend metadata-driven ordering and labeling to cover both form arguments and sidebar subcommands, including display order, headings, aliases, and combined labels.
- [x] 4.2 Surface long help, value names, and choice-level metadata where they improve discoverability in editors and help surfaces.
- [x] 4.3 Define the fallback behavior for partially extractable conditional-default metadata and then show supported user, default, environment, default-missing, and conditional-default sources without inventing preview argv tokens.
- [x] 4.4 Add UI tests for ordering, headings, source presentation, choice metadata, sidebar metadata, and preview behavior with sourced values.
