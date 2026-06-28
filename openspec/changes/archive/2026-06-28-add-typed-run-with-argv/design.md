## Context

`TuiApp::run()` already returns the canonical `Vec<OsString>`. The typed `Tui::run()` consumes that vector through clap and returns only `T`, so callers needing both representations must opt into `TuiApp` and repeat the typed parse. The new API must remain additive and preserve the established primary `run()` contract.

## Goals / Non-Goals

**Goals:**

- Return the parsed clap value and its exact canonical argv together.
- Keep one implementation path for typed reparsing, cancellation, and error mapping.
- Make the richer result explicit and discoverable without changing existing callers.

**Non-Goals:**

- Publishing shell rendering APIs.
- Changing `TuiApp`, callback execution, cancellation, or error semantics.
- Adding conversion conveniences or new lifetime constraints.

## Decisions

### Return a named invocation record

Add a root-exported, non-exhaustive `TuiInvocation<T>` with public `command` and `argv` fields. Named fields are clearer than a tuple, while `#[non_exhaustive]` permits compatible future additions. The type derives `Debug`, `Clone`, `PartialEq`, and `Eq` when `T` supports them.

### Name the richer method `run_with_argv`

The method consumes the runner and performs the same interactive operation as `run()`, so retaining the `run` verb is important. Naming argv directly makes the additional result discoverable; the zero-argument signature distinguishes returned argv from an input parameter. Generic names such as `run_full` and conversion names such as `into_invocation` obscure either the output or the fallible interactive operation.

### Make the richer method the shared typed implementation

`run_with_argv()` obtains argv from `TuiApp::run()`, passes `&argv` to `T::try_parse_from`, and returns both owned results. Borrowing the vector as clap input does not make `T` borrow from argv. `run()` delegates to this method and projects the invocation to its `command`, preventing behavior drift between typed paths.

## Risks / Trade-offs

- [A second typed method increases the public surface] -> Keep it narrowly scoped and retain `run()` as the documented default.
- [`with_argv` could be read as accepting argv] -> The method takes no argv parameter, and its rustdoc states that argv is returned.
- [Callers may mistake argv for printable shell text] -> Document that it is an executable token sequence and shell rendering remains separate.
