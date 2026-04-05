## Context

The release review found that `clap-tui` is closer to ready than its docs suggest. The public API is relatively small, but the crate docs still present type inventory before decision-making, the README quick start is not minimal enough, and docs.rs users do not see a second supported flow in code.

The main API clarity risks are concentrated in naming. The current launcher name describes its implementation more than its user-facing purpose, and the typed direct-TUI path is still more coupled to `clap::Parser` terminology than it needs to be. `#[clap_tui::main]` is good convenience syntax, and `TuiApp` is already a clear untyped primitive.

## Goals / Non-Goals

**Goals:**
- Present one concise public narrative across the README, crate-level rustdoc, and key item docs.
- State project provenance clearly so users understand both the inspiration and the non-official status.
- Make entry-point choice obvious for first-time users.
- Give the typed direct-TUI path one clear public story before `1.0`.
- Keep advanced runtime customization documented without letting it dominate the landing docs.

**Non-Goals:**
- Changing launcher semantics or broadening the supported feature set.
- Reworking the runtime event model or moving it into a new public module in this change.
- Expanding examples into a large tutorial set.

## Decisions

### 1. Public docs will follow the same usage-first order everywhere

The README and crate-level rustdoc will use the same narrative shape:

1. why `clap-tui` exists
2. provenance and project status
3. minimal quick start
4. choosing an entry point
5. one short second example
6. customization and advanced notes

This keeps docs.rs, crates.io, and the repository aligned.

Alternatives considered:
- Keep richer guidance only in the README.
  Rejected because docs.rs is a primary discovery surface.
- Keep the current inventory-first rustdoc structure.
  Rejected because it makes the surface feel larger than it is.
- Mention inspiration or project status only in the repository.
  Rejected because crates.io and docs.rs users also need that context.

### 2. `TuiLauncher` becomes the canonical derive-based entry point

The public docs will explicitly position the main surfaces:

- `TuiLauncher` for most derive-based CLIs
- `#[clap_tui::main]` as convenience syntax over `TuiLauncher`
- `TypedTuiApp` for direct typed TUI execution without the synthetic launcher
- `TuiApp` for hand-built `clap::Command` values

`TuiLauncher` is more direct than `ParserLauncher`: it tells users what the type does without making them know or care that the derive-based path happens to use `clap::Parser`.

Alternatives considered:
- Present all entry points as peers.
  Rejected because that preserves the current confusion.
- Keep `ParserLauncher`.
  Rejected because the name is more coupled to implementation terminology than to user intent.

### 3. Rename `ParserTuiApp` to `TypedTuiApp` and `from_factory` to `from_parser`

The typed direct-TUI path will be renamed to `TypedTuiApp`. The current name is accurate, but it reads too close to `ParserLauncher` and forces the docs to explain the distinction instead of letting the type name do more of the work.

`TuiApp::from_parser::<T>()` will become the preferred construction spelling in docs and examples. `from_parser` states the user intent more clearly than `from_factory`, while `TypedTuiApp` remains the named type users see in item docs and signatures.

Alternatives considered:
- Keep `ParserTuiApp`.
  Rejected because it is the only public type name that repeatedly caused confusion in review.
- Keep `from_factory`.
  Rejected because it leaks trait mechanics instead of the user-facing concept.
- Hide the type and only document `TuiApp::from_parser::<T>()`.
  Rejected because the type remains part of the public surface and still needs a clear name.

### 4. Item docs will prefer user-facing language over architectural vocabulary

Type and method docs for the main public surfaces will explain recommended usage, return behavior, and relationships between entry points in plain language. Repetitive phrases such as "public surface" or "integration surface" will be reduced unless they add real precision.

Alternatives considered:
- Leave item docs technically correct but terse.
  Rejected because first-release users need orientation, not just correctness.

## Risks / Trade-offs

- [Pre-`1.0` rename churn] -> Update examples, docs, tests, and macro-facing references in the same change so users see one name everywhere.
- [README and rustdoc drift apart again] -> Mirror the same section order and core wording across both landing surfaces.
- [Shorter landing docs hide advanced behavior] -> Keep advanced constraints and runtime details in later sections and item docs.

## Migration Plan

1. Rename `ParserLauncher` to `TuiLauncher`, `ParserTuiApp` to `TypedTuiApp`, and `TuiApp::from_factory::<T>()` to `TuiApp::from_parser::<T>()`.
2. Update README, crate-level rustdoc, and main item docs to use the same usage-first narrative.
3. Add project provenance language that credits Trogon and states that `clap-tui` is not an official `clap` crate.
4. Refresh examples and references so the renamed entry points and construction paths stay consistent.
5. Verify the release surface after the doc and API updates.

Rollback is straightforward before release: the rename and doc changes can be reverted together if they create unexpected confusion.

## Open Questions

- Should the README embed a screenshot in this change, or is a repository link to maintained visuals enough for the first release?
