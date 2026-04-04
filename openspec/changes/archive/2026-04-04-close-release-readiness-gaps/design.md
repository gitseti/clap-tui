## Context

`clap-tui` already builds as a library crate and compiles its public examples, but the package still presents itself more like an in-repo project than a polished public dependency. The manifest is missing crates.io discovery metadata, the workspace-root README is not included in the packaged crate, and the crate-level docs do not yet provide a strong docs.rs entry point for new users.

The release-readiness review also found a second class of issues: some public docs no longer match implementation behavior. `TuiApp::run` documents clap validation failure even though validation remains inside the TUI flow, cancellation is exposed as a public error variant even though the main entry points normalize it into non-error returns, and `LayoutConfig.sidebar_ratio` is described as an unconstrained percentage even though layout logic clamps the realized width. These are not architectural bugs, but they weaken trust in the first public release.

This change is intentionally narrower than the existing publishing/CI preparation work. It focuses on the last-mile package and documentation polish needed so a crates.io release feels complete even before broader GitHub automation is finalized.

## Goals / Non-Goals

**Goals:**
- Make the published crate self-describing on crates.io by including a rendered README, complete public metadata, and clear example guidance.
- Improve docs.rs onboarding with a crate-level quick start and public documentation that accurately describes runtime behavior and supported extension points.
- Establish a lightweight release-readiness verification pass that confirms package contents and public docs are aligned before publishing.

**Non-Goals:**
- Add or redesign GitHub Actions workflows, tag-based publishing automation, or crates.io trusted publishing setup.
- Expand the supported public API surface beyond the currently exported types.
- Introduce new runtime behavior or UI features unrelated to documentation and release polish.
- Solve broader product questions such as long-term SemVer policy beyond what must be stated for the initial public release.

## Decisions

### Decision: Treat the root README as the canonical public entry point
The repository will keep a single canonical README at the workspace root and explicitly attach it to the crate manifest with `readme = "../../README.md"` or an equivalent crate-local copy if Cargo path behavior makes that clearer. This keeps GitHub and crates.io aligned while avoiding divergent public descriptions.

Alternatives considered:
- Maintain separate repo and crate READMEs. Rejected because it creates drift risk for a small project with one publishable crate.

### Decision: Rewrite the README for external adoption, not local development
The README will lead with what `clap-tui` does, how to add it as a dependency, the supported Rust version, feature flags, terminal/runtime caveats, and which examples demonstrate which use cases. Internal process notes about refactors or review outcomes will be removed or reframed as stable user-facing guarantees.

Alternatives considered:
- Keep the current README and add only missing metadata. Rejected because the current document still reads like project history rather than public onboarding.

### Decision: Mirror the most important onboarding guidance in crate-level rustdoc
The crate root docs will include a short quick-start example and link the reader toward the supported customization seams, feature expectations, and examples. The rustdoc content should complement the README rather than duplicate every section, giving docs.rs users enough context without sending them back to the repository immediately.

Alternatives considered:
- Rely on the README alone. Rejected because docs.rs is a primary discovery path for Rust users and should stand on its own.

### Decision: Make public docs match observable behavior exactly
Public API documentation will describe what callers can actually observe from the exported surface. That means documenting cancellation normalization at the entry points, clarifying when clap parsing errors can occur, describing configuration bounds such as sidebar width clamping, and removing contradictory wording like calling exported runtime types "crate-local" when they are intended extension seams.

Alternatives considered:
- Leave the current docs in place until a future API refactor. Rejected because misleading docs create avoidable trust and adoption problems for a first release.

### Decision: Use a focused readiness verification pass for package and docs quality
This change will define a repeatable verification pass centered on the public release surface: metadata inspection, packaged file inspection, README attachment, and rustdoc validation. It complements broader test and CI work without depending on that larger automation change to deliver user-visible polish.

Alternatives considered:
- Fold all verification into the broader publishing automation change. Rejected because this work benefits from a narrower acceptance bar that can be validated locally while the larger release pipeline is still in flight.

## Risks / Trade-offs

- [README shared between GitHub and crates.io can grow too broad] -> Keep the top sections optimized for adoption and move maintainer-only details into dedicated release documentation.
- [Crate-level docs can drift from the README] -> Reuse the same terminology, point to the same examples, and keep rustdoc focused on quick-start material rather than full narrative duplication.
- [Fixing documentation can expose ambiguous runtime semantics] -> Prefer documenting actual current behavior in this change and defer behavior changes to separate product or API work.
- [A lightweight verification pass can be mistaken for full release automation] -> Document it as a public-surface check that complements, rather than replaces, broader CI and publish workflows.

## Migration Plan

1. Update the crate manifest so packaged metadata and README attachment match the intended public release identity.
2. Rewrite the README and crate-level rustdoc to form a coherent onboarding path for crates.io and docs.rs.
3. Correct public API docs and configuration descriptions so they match current runtime behavior.
4. Add or document the release-readiness verification commands that confirm package contents and docs quality.
5. Run the readiness checks and confirm the crate is presentable for a first public release.

## Open Questions

- The canonical repository, homepage, and docs.rs URLs still need to use the real public repository identity chosen for release. This change assumes those values will be available before implementation is finalized.
