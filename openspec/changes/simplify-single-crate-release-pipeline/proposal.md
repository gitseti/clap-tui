## Why

`clap-tui` already behaves like a single published crate repository, but the release story is still split across old names, mixed responsibilities, and stale OpenSpec requirements from the removed `clap-tui-macros` era.

Repo inspection shows a mismatch between the intended model and the documented/spec'd model:

- The workspace only contains `crates/clap-tui` and `xtask`; `crates/clap-tui-macros` is already gone from the tree and root [Cargo.toml](/Users/tillseeberger/Projects/clap-tui/Cargo.toml).
- The active workflows are already single-crate only: [`.github/workflows/publish.yml`](/Users/tillseeberger/Projects/clap-tui/.github/workflows/publish.yml) publishes only `clap-tui`, and `.github/workflows/publish-macros.yml` no longer exists.
- The shared verification entry point is still named [scripts/verify-release-readiness.sh](/Users/tillseeberger/Projects/clap-tui/scripts/verify-release-readiness.sh) even though CI runs it on every pull request through the stable `verify` job in [`.github/workflows/ci.yml`](/Users/tillseeberger/Projects/clap-tui/.github/workflows/ci.yml).
- The publish workflow compiles a whole `xtask` binary for a single tag/version comparison, and it reruns the baseline verification twice when publishing is enabled because `--publish-dry-run` is bundled into the same script.
- Maintainer docs in [docs/release-readiness.md](/Users/tillseeberger/Projects/clap-tui/docs/release-readiness.md) mix recurring release steps, one-time publishing setup, local troubleshooting, and `cargo release` details in one page, while still mentioning `CHANGELOG.md` even though no such file exists in the repository.
- Canonical OpenSpec requirements in [`openspec/specs/github-release-pipeline/spec.md`](/Users/tillseeberger/Projects/clap-tui/openspec/specs/github-release-pipeline/spec.md) and [`openspec/specs/crate-publishing-readiness/spec.md`](/Users/tillseeberger/Projects/clap-tui/openspec/specs/crate-publishing-readiness/spec.md) still describe the removed proc-macro crate and its release prerequisite, even though the completed `simplify-explicit-tui-integration` change already established the single-crate direction.

This leaves the repo harder to run under pressure than it needs to be. The important invariants are still correct, but the structure around them is more complicated than the current repository warrants.

## Scope

In scope:

- Align OpenSpec, workflows, scripts, and maintainer docs with a single published crate model centered on `clap-tui`
- Remove remaining release-process assumptions that depend on `clap-tui-macros`
- Keep release publication tag-driven on pushed `vX.Y.Z` tags
- Preserve the stable `verify` CI job as the required status check contract
- Preserve verification-before-release, tag/version validation, opt-in publish gating, and trusted-publishing-first behavior
- Decide the future of `xtask check-tag-version`
- Decide the future role, naming, and scope of the shared verification script
- Reorganize release documentation into a clearer routine/setup/troubleshooting structure

Out of scope:

- Changing `clap-tui` runtime behavior or public API unrelated to release automation
- Reintroducing launcher or proc-macro support
- Switching publication to GitHub Release events
- Broad CI expansion such as adding matrices, extra platforms, or non-release policy changes

## Current-State Findings

### Repository structure and dependency impact

- The workspace members are `crates/clap-tui` and `xtask` only.
- `crates/clap-tui/Cargo.toml` has no dependency on `clap-tui-macros`.
- The repository tree contains no `crates/clap-tui-macros` directory.
- A repo-wide search outside archived OpenSpec artifacts found no remaining README, docs, workflow, script, example, or test references that require proc-macro support.
- The remaining macros-era references are concentrated in canonical OpenSpec specs that have not yet been updated.

### Release process simplification

- [scripts/verify-release-readiness.sh](/Users/tillseeberger/Projects/clap-tui/scripts/verify-release-readiness.sh) is the de facto shared verification contract for local maintainers and CI, but its name implies a release-only purpose.
- The script currently combines two concerns:
  - baseline repository verification: fmt, clippy, tests, terminal-stack check, package inspection
  - optional crates.io publish dry-run
- Because [`.github/workflows/publish.yml`](/Users/tillseeberger/Projects/clap-tui/.github/workflows/publish.yml) always reruns the script once and reruns it again with `--publish-dry-run` when publishing is enabled, the baseline checks run twice in the publish path.
- [scripts/check-terminal-stack.sh](/Users/tillseeberger/Projects/clap-tui/scripts/check-terminal-stack.sh) still protects a meaningful ongoing invariant for the default dependency graph and is not macros-specific.
- `cargo package -p clap-tui --list` remains a meaningful baseline check because the repository publishes a library crate whose packaged surface matters independently of release day.

### GitHub Actions simplification

- `publish.yml` is already single-crate and already preserves most desired safety invariants:
  - tag-driven trigger on `v*`
  - tag/version validation
  - verification rerun
  - publish dry-run before real publish
  - opt-in publish gating with `CLAP_TUI_PUBLISH_MODE`
  - OIDC-first trusted publishing with token fallback
- `.github/workflows/publish-macros.yml` is already absent, so the remaining work is to update specs/docs and simplify naming/structure rather than delete an active workflow.
- [`.github/workflows/ci.yml`](/Users/tillseeberger/Projects/clap-tui/.github/workflows/ci.yml) still runs on all pushes, including release tags. Because `publish.yml` already reruns baseline verification on release tags, this likely creates redundant tag-push verification noise.

### Tag/version validation

- `xtask` currently exists only to implement `check-tag-version`.
- The current command reads a single manifest, checks for a leading `v`, and compares the tag version to `crates/clap-tui/Cargo.toml`.
- Keeping a compiled Rust helper for this one invariant is heavier than the repository now needs, while fully inlining the logic into a workflow would make the invariant less reusable and less visible to maintainers.

### Documentation structure

- [docs/release-readiness.md](/Users/tillseeberger/Projects/clap-tui/docs/release-readiness.md) currently acts as:
  - baseline verification guide
  - publish preflight guide
  - trusted publishing setup guide
  - `cargo release` setup guide
  - end-to-end runbook
  - troubleshooting notes
- The single page is workable but not optimized for a maintainer cutting a release under time pressure.
- The page also contains at least one stale instruction (`CHANGELOG.md`) that does not match current repo contents.

## Proposed Change

### Release model

Treat `clap-tui` as the repository's only published crate everywhere: workflows, docs, and canonical OpenSpec requirements.

- Publication remains triggered by pushing `vX.Y.Z` tags.
- GitHub Release notes become the canonical human-facing release-notes artifact.
- A GitHub Release page may still be created later for notes or discoverability, but it is not the publish trigger.
- Publish remains opt-in through `CLAP_TUI_PUBLISH_MODE`.
- Trusted publishing via OIDC remains the preferred path; token mode stays as an explicit fallback.

### CI model

Keep the stable required job name `verify`, but narrow its purpose to baseline repository verification rather than release-flavored wording.

Recommended direction:

- Rename/reframe the shared script as a general repo verification entry point, preferably `./scripts/verify.sh`.
- Keep it as the canonical baseline contract used by:
  - local maintainers
  - pull request verification
  - the publish workflow's pre-publish rerun
- Keep its baseline scope:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-targets --all-features`
  - `./scripts/check-terminal-stack.sh`
  - `cargo package -p clap-tui --list`
- Keep `--allow-dirty` for local iteration, because the package-surface check is still useful before a clean commit exists.
- Update `ci.yml` to ignore `v*` tag pushes so release tags are verified by the publish workflow only once.

### Publish workflow

Keep `publish.yml`, but simplify the division of responsibilities:

1. Validate tag/version match.
2. Rerun the baseline shared verification contract.
3. If publishing is enabled, run an explicit `cargo publish -p clap-tui --locked --dry-run`.
4. If publishing is enabled, publish with trusted publishing first or token fallback second.
5. If publishing is disabled, stop after successful verification with a clear explanation.

This keeps the safety invariants while removing the current baseline-verification duplication caused by `--publish-dry-run` living inside the shared script.

### Tag validation mechanism

Replace `cargo run -q -p xtask -- check-tag-version` with a small repository script, not inline workflow shell and not the current `xtask`.

Recommendation:

- Add a tiny shell helper such as `./scripts/check-tag-version.sh` that:
  - accepts `vX.Y.Z`
  - validates the `v` prefix
  - reads `crates/clap-tui/Cargo.toml`
  - fails clearly on mismatch
- Call that helper from `publish.yml`.
- Document it as the local way to verify a release tag before pushing.

Rationale:

- Smaller and simpler than compiling `xtask` for one comparison.
- More reusable and visible than embedding ad hoc shell inside GitHub Actions.
- Keeps the invariant repo-owned and testable without preserving a now-thin Rust maintenance surface.

### Verification script role and scope

Keep one shared verification contract, but make it explicitly baseline-only.

Recommended direction:

- Rename the script away from release-specific wording.
- Remove `--publish-dry-run` from the shared script.
- Keep package inspection in baseline verification.
- Keep `check-terminal-stack.sh` in baseline verification.
- Document publish dry-run as a separate release-preflight command used:
  - locally when maintainers want a release preflight
  - in `publish.yml` when publishing is enabled

This preserves local/CI symmetry where it matters most, while avoiding a script whose name and options imply that every CI run is part of a release.

### Release documentation

Restructure release docs into three clear audiences:

1. Routine release runbook
   - the happy path a maintainer follows for a normal `vX.Y.Z` release
   - baseline verify command
   - optional publish dry-run
   - tag creation and push
   - publish workflow expectations
   - prepare GitHub Release notes as the canonical human-facing release notes

2. Publishing setup / bootstrap
   - crates.io owner expectations
   - trusted publishing registration
   - `CLAP_TUI_PUBLISH_MODE`
   - token fallback setup
   - `cargo release` as an optional maintainer helper rather than a required release step

3. Troubleshooting / rationale
   - verification-only mode
   - local dry-run behavior
   - tag/version mismatch failures
   - what to do after a bad publish or yank
   - any optional GitHub Release-page notes

The docs should stop referencing nonexistent artifacts, explicitly direct maintainers to GitHub Release notes for release communication, and describe `cargo release` as optional tooling rather than the canonical release path.

## Alternatives Considered

### Tag-driven publishing vs GitHub Release-driven publishing

- Keep tag-driven publishing: recommended.
  - Matches current workflow design.
  - Keeps the publish trigger tied to a reviewed git ref.
  - Avoids splitting authority between tags and Release events.
- Switch to GitHub Release-driven publishing: rejected.
  - Adds indirection without solving a current repo problem.
  - Conflicts with the already-decided tag-authoritative model.

### Keep `xtask` vs use a repo script vs inline the check

- Keep `xtask`: rejected.
  - The repo no longer appears to need a dedicated Rust task runner for one small command.
- Inline shell in GitHub Actions: rejected.
  - Smallest line count, but hides a release-critical invariant inside workflow YAML.
- Small repo script: recommended.
  - Smallest maintainable shared solution.

### Keep the verification script vs remove it

- Remove the shared script and duplicate commands in CI/docs/workflows: rejected.
  - Increases drift risk for the baseline verification contract.
- Keep a shared script: recommended.
  - The repo still benefits from one canonical baseline verification entry point shared between local maintainers and CI.

### Baseline-only verification script vs script with optional publish dry-run

- Keep `--publish-dry-run` in the shared script: rejected.
  - Encourages mixed responsibilities.
  - Causes baseline duplication in the publish workflow.
- Baseline-only shared script plus explicit publish dry-run command: recommended.
  - Cleaner separation between repo verification and release preflight.

### One long release page vs split runbook/setup/troubleshooting docs

- Keep one long page: workable, but not preferred.
  - Lower file count, but keeps different maintainer needs interleaved.
- Split by purpose: recommended.
  - Better for release-day ergonomics and future maintenance.

## Risks And Migration Notes

- Stale references may remain in docs or specs even after workflow cleanup unless the implementation explicitly searches for old macros-era wording.
- Branch protection may depend on the stable `verify` job name, so the workflow can be reorganized but the required job name must not change.
- Over-simplifying the shared verification script could accidentally drop package-surface or dependency-graph checks that still matter.
- Removing `xtask` without replacing its documented behavior would weaken the tag/version invariant.
- If CI starts ignoring `v*` tags, the publish workflow must continue rerunning baseline verification so release tags are still gated.
- `release.toml` is not macros-specific, but the docs need to make clear whether `cargo release` is a recommended happy-path tool or just an optional helper.
- The docs currently reference `CHANGELOG.md`, which does not exist; implementation must either remove that step or replace it with the actual release-notes practice rather than silently leaving the mismatch behind.
- If the docs leave `cargo release` in the happy path, maintainers may incorrectly treat it as the authoritative release boundary instead of a helper around version bumps and tags.

## Acceptance Criteria

- The repository no longer contains active release automation or canonical OpenSpec requirements that assume a separate `clap-tui-macros` publish path.
- `publish.yml` reflects a single-crate `clap-tui` release model only.
- Release publication remains tag-driven on pushed `v*` tags rather than GitHub Release events.
- Tag/version mismatch still fails before any crates.io authentication or publish attempt.
- The stable CI job name remains `verify`.
- The repository has one clear, justified decision on tag/version validation, and the chosen mechanism is documented.
- The repository has one clear, justified decision on the shared verification script, and its name and scope match its actual role.
- If the shared verification script remains, it serves as the canonical baseline verification contract for local maintainers and CI.
- Baseline verification still includes package-surface inspection and terminal-stack validation.
- Verification-only mode still works when publishing is disabled.
- Trusted publishing remains the preferred mode and token publishing remains the explicit fallback.
- Release docs describe a single-crate happy path without obsolete proc-macro prerequisites.
- Release docs identify GitHub Release notes as the canonical human-facing release-notes artifact and do not require `CHANGELOG.md`.
- Release docs describe `cargo release` as an optional helper rather than a required happy-path step.
- Release docs separate recurring runbook guidance from one-time publishing setup and troubleshooting guidance.
- Any stale references to nonexistent or removed release artifacts are removed or replaced with accurate instructions.

## Open Questions

- What is the canonical release-notes artifact for this repository going forward? The current docs say `CHANGELOG.md`, but no such file exists. The implementation should document the real source of release notes rather than preserve a placeholder step.
- Should `cargo release` remain part of the recommended routine release flow, or should it be demoted to an optional helper documented only in setup/troubleshooting? The existing `release.toml` supports it, but the current repo does not prove it must be the main happy path.
