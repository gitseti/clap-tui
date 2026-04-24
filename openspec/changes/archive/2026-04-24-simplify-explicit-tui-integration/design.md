## Context

`clap-tui` currently exposes and documents multiple derive-based integration stories at once: direct typed execution through `TypedTuiApp`, launcher-driven execution through `TuiLauncher`, and proc-macro convenience through `#[clap_tui::main]`. The active integration change doubles down on that by adding returning APIs on top of both synthetic and existing launcher modes.

That is now misaligned with the intended 0.1.0 product. The target release should present one explicit model: the application defines a normal `tui` subcommand in its own clap tree, matches that subcommand during ordinary dispatch, calls `Tui::<T>::run()`, and then dispatches the returned typed value itself. `clap-tui` should be a small typed TUI runner, not a launcher framework.

The change is cross-cutting because it affects public naming, crate docs, examples, tests, error semantics, workspace layout, and release assumptions. It also includes a dependency change in `tui-textarea`, which makes the design worth locking down before implementation.

## Goals / Non-Goals

**Goals:**
- Provide one primary explicit integration model for 0.1.0 based on a normal user-defined `tui` subcommand.
- Provide a small returning API for explicit integration that runs the TUI for a clap type and returns `Result<Option<T>, TuiError>`.
- Use `Tui::<T>::run()` as the primary 0.1.0 public spelling.
- Preserve the typed returning contract without automatic printing or process exit.
- Keep `clap-tui` in control of terminal/backend integration by making `tui-textarea` backendless.
- Reduce the 0.1.0 public and release surface to the minimum understandable set.

**Non-Goals:**
- Supporting synthetic launcher interception as a primary 0.1.0 story.
- Adding existing-launcher mode or any launcher-mode matrix.
- Preserving macro support as part of the 0.1.0 release surface.
- Introducing a broad command filtering API.
- Migrating to Ratatui 0.30 or the split `ratatui-*` crates.

## Decisions

### 1. Explicit user-owned `tui` dispatch is the canonical 0.1.0 integration model

The library will document and optimize for this shape:

```rust
match cli.command {
    Command::Tui => {
        if let Some(cmd) = Tui::<Command>::run()? {
            dispatch(cmd)?;
        }
    }
    other => dispatch(other)?,
}
```

This keeps outer clap parsing, help, completion, routing, logging, and exit handling owned by the application. `clap-tui` only runs when the application explicitly calls it.

Alternatives considered:
- Keep `TuiLauncher` as the primary surface and merely add returning variants.
  Rejected because it preserves the wrong mental model and keeps launcher complexity at the center of the API.
- Support both launcher and explicit-subcommand integration as equal first-class stories.
  Rejected because 0.1.0 should converge on one small understandable shape rather than document multiple competing flows.

### 2. API naming: use `Tui::run()` as the primary 0.1.0 surface

The primary 0.1.0 entry point should be:

- `Tui::<T>::run() -> Result<Option<T>, TuiError>`

This naming matches the user mental model more closely than `TypedTuiApp` or `run_parse`: the caller is running a TUI for a clap-parsable type and receiving an optional typed result back.

`TypedTuiApp` is too framework-like for the intended release surface, and `run_parse` describes an internal mechanic rather than the user-facing operation. Lower-level builders or argv-oriented entrypoints can remain internal, secondary, or deferred until real downstream needs justify them.

Alternatives considered:
- Keep `TypedTuiApp` as the named type and add `run_parse`.
  Rejected because it preserves a more abstract API story than the narrowed 0.1.0 product needs.
- Rename only the method while keeping `TypedTuiApp`.
  Rejected because the main simplification win comes from collapsing both the type and method story into one direct concept.

### 3. Returning semantics are defined on `Tui::<T>::run()`

`Tui::<T>::run()` is the canonical returning API, and its behavior is part of the contract:

- it never prints automatically
- it never calls `std::process::exit`
- `Ok(Some(T))` means successful submission and typed reparse
- `Ok(None)` means cancellation before submission only
- `Err(TuiError::Clap(_))` means clap saw argv and returned help, version, or parse-display behavior
- other `TuiError` values mean runtime, rendering, or internal failure

This keeps library behavior predictable for embedded or async applications and lets the outer CLI decide how clap errors are displayed or mapped to exit codes.

Alternative considered:
- Keep callback-oriented APIs as the primary documented surface.
  Rejected because callbacks hide the typed return value that explicit integration is trying to expose.

### 4. Launcher-specific and macro-specific surfaces should be removed from the 0.1.0 release plan

The cleanest implementation path is to remove launcher-centric APIs, docs, tests, and the proc-macro crate from the 0.1.0 surface now rather than marking them as “supported later.” The current macro crate is tightly coupled to the launcher model and adds a second published crate, extra docs branches, trybuild coverage, and release workflow complexity.

If the project later wants launcher or macro sugar again, it can be reconsidered from the smaller explicit core rather than carried forward as a legacy anchor.

Alternatives considered:
- Leave the proc-macro crate in the workspace but undocumented.
  Rejected because it still complicates publishing and makes the public surface ambiguous.
- Mark launcher/macro flows as deferred while preserving them in the release surface.
  Rejected because that still keeps implementation and documentation burden around immature APIs.

### 5. `tui-textarea` should be configured in backendless mode

`clap-tui` already owns runtime input translation and terminal/backend setup. It should therefore depend on `tui-textarea` without its default backend integration so the default dependency graph does not pull in a second backend path.

This preserves the existing dependency-cleanup goal without widening scope into a Ratatui 0.30 migration.

Alternative considered:
- Fold dependency cleanup into a broader Ratatui migration.
  Rejected because the migration is larger and not required to achieve a clean 0.1.0 integration surface.

### 6. Documentation, examples, and tests should all converge on the same single story

README, crate docs, examples, and tests should all use `Tui::<T>::run()` from a normal `Command::Tui` match branch as the canonical integration recipe. Any lower-level or legacy surfaces that remain temporarily during implementation should not be positioned as equivalent public choices.

This is necessary to prevent the docs from reintroducing launcher thinking after the code is simplified.

## Risks / Trade-offs

- [Removing launcher and macro surfaces breaks immature pre-1.0 integrations] -> Accept the break now while the crate is still stabilizing, and document the explicit replacement path clearly.
- [Applications may still want to hide the outer `tui` marker inside the rendered TUI tree] -> Keep caller-controlled command shaping possible without introducing a broad filtering API in 0.1.0.
- [`Tui::<T>::run()` may require internal refactoring from the current app-oriented structure] -> Treat naming and public entrypoint simplification as part of the same change rather than layering a thin alias over confusing internals.
- [Removing the macro crate changes publishing and CI assumptions] -> Update workspace layout, publishing specs, and release workflows in the same change so the repo reflects the intended 0.1.0 surface consistently.
- [Dependency cleanup may require small editor integration adjustments] -> Keep the `tui-textarea` upgrade narrowly scoped and verify the default dependency graph immediately.

## Migration Plan

1. Add the primary typed returning API `Tui::<T>::run() -> Result<Option<T>, TuiError>`.
2. Ensure it supports explicit integration from a normal `Command::Tui` dispatch branch.
3. Rework docs, examples, and tests to use `Tui::run()` as the canonical public surface.
4. Remove launcher-centric APIs, errors, docs, and tests from the 0.1.0 story.
5. Remove the proc-macro crate from the workspace and release surface, and simplify release-readiness and publish workflow assumptions accordingly.
6. Configure `tui-textarea` in backendless mode and verify the default dependency tree stays coherent.

Rollback is straightforward because the change is local to public API selection, docs, workspace composition, and dependency configuration. If needed, the old launcher code can be restored from history more easily than carrying both stories forward in parallel.

## Open Questions

- None blocking. The main product decisions are settled: use `Tui::<T>::run()` as the primary surface, keep explicit dispatch outside the library, and remove launcher and macro scope from 0.1.0.
