## Context

`clap-tui` already has the core pieces needed for an ergonomic derive-based launcher:

- `TuiApp` can build a TUI from a `clap::Command` and return argv or execute typed handlers.
- the crate already treats `TuiApp`, `TuiConfig`, and `Runtime` as its intended stable public seams.
- the command model builder in `spec.rs` already filters hidden args, so hiding synthetic clap surface from the rendered TUI has a natural home.
- the project already identified a public API risk in the current unbound `run_with_parser` helper and wants parser-backed execution to stay tied to the clap schema that generated the TUI.

What is missing is a canonical typed way to attach a discoverable `tool tui` launcher to a derive-based CLI without forcing users to add fake enum variants or manual pre-parse glue in every binary. The design needs to improve the user-facing entrypoint while preserving clap-correct help, parse errors, and typed execution, with any macro surface layered on top rather than becoming the only real API.

## Goals / Non-Goals

**Goals:**

- Add a first-class typed launcher API for derive-based root parser types.
- Expose a synthetic root `tui` subcommand such as `tool tui` without requiring user-defined parser variants.
- Add a `#[clap_tui::main]` convenience layer over that typed launcher.
- Keep clap help, version output, and diagnostics aligned with the augmented command surface that includes the synthetic launcher.
- Make the TUI launch path parse back into the same root parser type that produced the command, avoiding the current unbound parser footgun for this flow.
- Keep the synthetic launcher out of the TUI command tree and form views.
- Keep v1 scoped tightly enough to ship with clear behavior and good test coverage.

**Non-Goals:**

- Supporting arbitrary nested launcher placement such as `tool admin tui` in v1.
- Supporting non-derive `clap` APIs as the primary macro target.
- Reworking all existing `TuiApp` construction APIs as part of this change beyond what is needed to add a typed launcher.
- Turning proc-macro-generated launcher internals into a broad new stable extension surface.

## Decisions

### 1. Make the typed launcher the canonical API and the macro additive sugar

The canonical public surface should be an ordinary Rust API that binds a root parser type to the synthetic launcher flow. The macro should remain available as convenience syntax, but it should delegate to the typed launcher instead of defining the feature’s only meaningful contract.

This fits the project’s library-first posture better than making the proc macro the fundamental surface. It also aligns with the existing desire to move parser-backed execution toward schema-bound typed APIs rather than convenience helpers that hide too much behavior.

Alternatives considered:

- Make the proc macro the canonical entrypoint.
  Rejected because it would make the deepest integration surface macro-driven when the core behavior fits naturally into typed library APIs.
- Provide only macro-free APIs and no convenience syntax.
  Rejected because the one-line attribute remains a valuable ergonomic path for common cases.

### 2. Implement a typed launcher wrapper around `TuiApp` rather than making users compose `TuiApp` manually

The crate should expose a typed launcher wrapper that owns synthetic launcher setup, augmented-command parsing, non-TUI fallthrough, and typed post-TUI parsing for a root type `T` that can both produce and parse the clap schema. `TuiApp` remains the runtime-building primitive underneath, but users should not have to compose the full launcher flow by hand for the common derive-based case.

This gives the library a reusable core with a clear contract. It keeps launcher semantics in normal Rust, leaves room for advanced non-macro use, and makes the eventual macro a trivial adapter over the typed API.

Alternatives considered:

- Continue requiring users to wire `TuiApp::from_factory(...).run_with_parser(...)` manually.
  Rejected because it keeps the verbose setup and does not create a first-class discoverable `tool tui` launcher path.
- Replace `TuiApp` entirely with the typed launcher.
  Rejected because `TuiApp` is still the right lower-level primitive for advanced usage and non-derived command sources.

### 3. Keep most launcher logic in ordinary library code and make the proc macro thin

The proc macro will live in a new `crates/clap-tui-macros` crate, but the clap augmentation and launch decision logic should primarily live in ordinary library code inside `clap-tui`. The macro should expand into a small wrapper that delegates to the typed launcher API.

This keeps the most complex behavior in normal Rust where it is easier to test, reason about, and evolve. It also makes the macro mostly syntax validation and crate-path wiring rather than a large code generator with embedded business logic.

Alternatives considered:

- Put all launcher behavior directly into macro expansion.
  Rejected because it would make the most delicate clap behavior harder to test and maintain.
- Make the support helpers alone public without a typed wrapper API.
  Rejected because users would still have to assemble the common launcher flow themselves.

### 4. Treat the augmented clap command as the authoritative help and diagnostics surface

The typed launcher will first build the root `clap::Command`, validate that synthetic launcher attachment is safe, augment that command with a synthetic root `tui` subcommand, and parse argv against the augmented command before choosing the execution path. Help, version, and parse failures should therefore come from the augmented command surface, so users see `tui` in the same CLI help that launches it.

This avoids a confusing split where `tool --help` would omit `tui` even though `tool tui` is supported, or where parse errors would refer to a different command shape than the one users can actually invoke.

Alternatives considered:

- Keep help and diagnostics on the original typed parser surface and only special-case `tool tui`.
  Rejected because it would make the synthetic launcher feel bolted on and would produce inconsistent clap UX.
- Reconstruct typed values from augmented matches without reparsing.
  Rejected because it couples the launcher to clap internals and risks divergence from the normal typed parse path.

### 5. Bind the TUI launch path to the same parser type that produced the command

The typed launcher will require a root type that can both build the command and parse argv again after the TUI returns. The successful TUI path should therefore parse the returned argv into that same root type and call the user handler directly. The non-TUI path should also parse through the typed parser instead of reconstructing typed state from augmented matches.

This aligns with the existing design direction to prefer schema-bound parser execution over the current unbound `run_with_parser` shape. The new entrypoint becomes a safer path by construction rather than more sugar over the existing footgun.

Alternatives considered:

- Reuse `TuiApp::run_with_parser` internally.
  Rejected because it preserves the unbound helper shape and makes the macro less explicit about typed binding.
- Parse only once against the augmented command and translate `ArgMatches` into the user type.
  Rejected because clap derive users expect their normal typed parser semantics, not a custom reconstruction layer.

### 6. Keep v1 root-only and reject ambiguous launcher hosts

In v1, the synthetic launcher will attach only at the CLI root as `tool tui`. The support layer should reject attachment when the root already has a real `tui` subcommand or alias, when the unmodified grammar already accepts `tool tui` as ordinary input, or when external subcommands or trailing/raw positional behavior make the synthetic launcher ambiguous.

This keeps the first release understandable and avoids subtle parse interactions that would be expensive to explain and test. A narrower v1 contract is preferable to a more flexible launcher whose corner cases are hard to predict.

Alternatives considered:

- Support nested attachment points immediately.
  Rejected because command-path insertion rules and ambiguity checks become much more complex.
- Allow synthetic insertion even when clap grammars are ambiguous.
  Rejected because silent shadowing of real grammar would break user trust.

### 7. Hide the synthetic `tui` command from the rendered TUI by honoring hidden subcommands

The synthetic launcher should be visible in ordinary clap help but hidden from the TUI command tree itself. The simplest way to achieve that is to mark the synthetic subcommand hidden on the command passed into `TuiApp` and update `spec.rs` to skip hidden subcommands just as it already skips hidden args.

This reuses existing clap concepts instead of inventing a second filtering mechanism specific to the TUI renderer.

Alternatives considered:

- Teach rendering code to special-case a synthetic launcher name.
  Rejected because it introduces ad hoc knowledge about one feature into unrelated rendering paths.
- Leave the synthetic `tui` node visible inside the TUI.
  Rejected because it would expose an internal launcher path as a nonsensical interactive command choice.

## Risks / Trade-offs

- [Two entrypoint layers confuse users] -> Document the typed launcher as the canonical API, describe the macro as convenience syntax, and keep both paths behaviorally identical.
- [Clap grammar ambiguity checks miss an edge case] -> Keep v1 scope narrow, reject known ambiguous host patterns up front, and add scenario tests for conflicts, aliases, and raw/external-subcommand hosts.
- [The new launcher surface overlaps awkwardly with existing parser execution APIs] -> Position the typed launcher as the preferred schema-bound path for derive-based CLIs while leaving lower-level `TuiApp` construction available for advanced cases.
- [Proc-macro crate and re-export wiring increase maintenance cost] -> Keep the macro API intentionally small and the expansion thin so most future changes stay in ordinary library code.

## Migration Plan

1. Add launcher support in `clap-tui` for a canonical typed API that owns conflict checks, synthetic subcommand insertion, launch detection, and typed execution.
2. Update command/spec generation to skip hidden subcommands so the synthetic launcher can remain visible in clap help while disappearing from the TUI command tree.
3. Add a new `crates/clap-tui-macros` workspace member and implement `#[clap_tui::main]` as a thin wrapper over the typed launcher.
4. Add tests covering the typed launcher core, valid macro shapes, launcher conflicts, help/diagnostic behavior, TUI launch success and cancellation, and ordinary non-TUI fallthrough.
5. Document the typed launcher as the canonical API and the macro as convenience syntax in the README and any crate-level public docs that describe the primary launch path.

Rollback is straightforward because the feature is additive: the macro and support helpers can be removed without changing the underlying manual `TuiApp` flow.

## Open Questions

- Should the macro support a custom launcher name in the future, or should `tui` remain fixed to preserve predictability?
- Should the typed launcher live as a new wrapper type beside `TuiApp`, or as a typed constructor/builder layered directly onto `TuiApp`?
- How strict should the launcher conflict error messages be in v1: terse API errors, or more educational messages that explain exactly which clap host pattern caused rejection?
