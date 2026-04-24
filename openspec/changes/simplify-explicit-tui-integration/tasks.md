## 1. Primary API Surface

- [x] 1.1 Introduce the primary typed returning API `Tui::<T>::run() -> Result<Option<T>, TuiError>` and wire it to the existing typed TUI execution path.
- [x] 1.2 Ensure `Tui::<T>::run()` preserves the explicit return contract for submit, cancel, clap-display errors, and runtime failures without automatic printing or `std::process::exit`.
- [x] 1.3 Rework public exports and item docs so `Tui` is the canonical 0.1.0 direct-entrypoint surface and `TypedTuiApp` / `run_parse` are not presented as the primary spelling.

## 2. Remove Launcher And Macro Scope

- [x] 2.1 Remove launcher-centric public APIs, docs, errors, and tests from the 0.1.0 surface, including synthetic-launcher-specific behavior that is no longer part of the primary integration story.
- [x] 2.2 Remove the `clap-tui-macros` crate from the workspace and release surface, including the proc-macro re-export, macro docs, and trybuild coverage.
- [x] 2.3 Simplify examples and integration tests around the explicit application-owned `Command::Tui` dispatch pattern.

## 3. Dependency And Release Surface Cleanup

- [x] 3.1 Configure `tui-textarea` in backendless mode and update any editor/runtime glue needed to keep `clap-tui` owning terminal/backend integration.
- [x] 3.2 Update README, crate-level docs, and example guidance to use `Tui::<T>::run()` as the canonical explicit integration model.
- [x] 3.3 Simplify package, release-readiness, and publish workflow configuration for a single-crate 0.1.0 release surface.

## 4. Verification

- [x] 4.1 Add or update tests covering `Tui::<T>::run()` success, cancellation, returned clap errors, and non-clap runtime failures.
- [x] 4.2 Add or update checks that confirm the default dependency graph does not reintroduce a second backend path through `tui-textarea`.
- [x] 4.3 Run the relevant formatting, test, and package-surface verification steps for the simplified release surface.
