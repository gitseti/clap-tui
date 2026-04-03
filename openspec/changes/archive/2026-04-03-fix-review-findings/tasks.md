## 1. Deterministic Input State

- [x] 1.1 Materialize environment-backed and default-backed input values during command initialization instead of re-reading them from effective-state query paths.
- [x] 1.2 Refactor effective form projection helpers so they operate only on stored app state plus static command metadata.
- [x] 1.3 Add tests proving that environment changes after initialization do not change rendered effective state, preview argv, or validation.

## 2. Interactive Runtime Integrity

- [x] 2.1 Add app-level paste handling for search focus and text-editing form widgets using the existing editor/update flow.
- [x] 2.2 Move toast expiry checks into the main event-loop timing flow so expired toasts clear under sustained input as well as idle polling.
- [x] 2.3 Add tests covering search paste, form paste, and toast expiration during continuous interaction.

## 3. Derived State Lifecycle

- [x] 3.1 Introduce cached derived argv and validation state with explicit invalidation on relevant domain-state mutations.
- [x] 3.2 Update rendering and Run handling to consume the shared derived-state cache instead of recomputing full clap validation on every redraw.
- [x] 3.3 Add tests proving redraw-only events reuse derived state while preview, visible validation, and Run remain aligned after edits.

## 4. Schema-Bound Parser Execution

- [x] 4.1 Add a parser-bound TUI construction and execution path that ties `Parser` execution to the same clap schema used to render the UI.
- [x] 4.2 Deprecate, constrain, or remove the unbound `run_with_parser` helper so command-based apps do not encourage mismatched parser execution.
- [x] 4.3 Update API tests and documentation to cover the preferred bound parser path and the remaining safe untyped execution paths.
