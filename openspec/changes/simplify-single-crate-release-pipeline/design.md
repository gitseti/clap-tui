## Overview

This change does not alter the core release invariants. It removes obsolete structure around them.

The repository already sits in an in-between state:

```text
Current reality
---------------
workspace        -> clap-tui + xtask
published crate  -> clap-tui only
publish workflow -> single-crate
verify workflow  -> shared baseline script
canonical specs  -> still partly two-crate
docs/script names-> still release-specific and mixed-purpose
```

The design goal is to make the repo tell one coherent story:

```text
Desired end state
-----------------
baseline verify   -> one shared repo verification contract
release trigger   -> push vX.Y.Z tag
release notes     -> GitHub Release notes
publish gate      -> CLAP_TUI_PUBLISH_MODE
publish auth      -> OIDC first, token fallback
published crate   -> clap-tui only
docs              -> runbook + setup + troubleshooting
```

## Decision 1: Keep one shared verification contract, but make it baseline-only

### Current state

`./scripts/verify-release-readiness.sh` is already shared by local maintainers, CI, and the publish workflow. That symmetry is valuable and should be preserved.

The problem is role confusion:

- the script name says "release readiness"
- CI uses it on every pull request
- the optional `--publish-dry-run` mode makes the script straddle both baseline verification and release-only preflight

That mixed scope creates duplication in `publish.yml`:

1. run baseline verification
2. run baseline verification again plus publish dry-run

### Decision

Keep a shared script, but narrow it to baseline repository verification and rename it accordingly.

Recommended shape:

- `./scripts/verify.sh`
- retains `--allow-dirty`
- drops `--publish-dry-run`

### Why this is the best fit here

- The repo still benefits from one shared local/CI verification contract.
- Package inspection and terminal-stack validation are ongoing invariants, not release-only extras.
- A baseline-only script keeps the contract stable while making the publish preflight more explicit.

### Rejected alternatives

- Remove the script entirely:
  - simpler file tree, but more duplication across CI, docs, and publish workflow
- Keep the script name and options as-is:
  - preserves current behavior, but keeps the baseline/release ambiguity and duplicate publish-path work

## Decision 2: Replace the one-command `xtask` with a tiny repo script

### Current state

`xtask` only implements `check-tag-version`. The code reads one manifest and compares it against a `vX.Y.Z` tag.

That invariant matters, but the current implementation is not sized to the current repo:

- compile Rust helper
- maintain a separate workspace member
- use it only in the publish workflow

### Decision

Replace `xtask check-tag-version` with a small repository-owned script such as `./scripts/check-tag-version.sh`.

### Why this is the best fit here

- The logic is shell-sized.
- The invariant remains visible and reusable outside workflow YAML.
- The repo no longer needs a Rust task runner just to compare a tag string with a manifest version.

### Rejected alternatives

- Keep `xtask`:
  - justified only if more repo tasks are expected soon; current facts do not show that
- Inline the check into GitHub Actions:
  - lowest ceremony, but hardest to discover and easiest to duplicate later

## Decision 3: Publish workflow should separate baseline verification from publish preflight

### Current state

The current publish flow is already close to the target model:

- tag trigger on `v*`
- tag/version validation
- rerun verification
- publish dry-run before real publish
- OIDC preferred
- token fallback
- verification-only mode when publishing is disabled

The main cleanup opportunity is structural, not functional.

### Decision

Organize `publish.yml` into three distinct phases:

1. baseline verification
2. publish preflight
3. actual publish

Concrete mapping:

- baseline verification:
  - tag/version check
  - shared `./scripts/verify.sh`
- publish preflight:
  - `cargo publish -p clap-tui --locked --dry-run` only when publish mode is enabled
- actual publish:
  - trusted publishing or explicit token fallback

### Why this is the best fit here

- Matches the maintainers' mental model on release day.
- Makes verification-only mode clearer.
- Removes the accidental double-run of baseline checks.

## Decision 4: CI should ignore release tags

### Current state

`ci.yml` runs on every push and pull request. `publish.yml` already reruns verification on release tags.

That means a `vX.Y.Z` tag push likely causes:

- one `verify` workflow run
- one `publish` workflow run that also reruns verification

### Decision

Keep `verify` as the stable job name, but update `ci.yml` so branch and PR changes still run verification while `v*` tag pushes do not trigger a second redundant CI workflow.

### Why this is the best fit here

- Preserves branch protection expectations.
- Keeps release-tag feedback concentrated in the publish workflow that actually owns the release contract.
- Reduces noise without weakening the gate.

## Decision 5: Split release docs by maintainer task

### Current state

`docs/release-readiness.md` currently mixes:

- daily-ish verification guidance
- release-day runbook
- one-time trusted publishing setup
- `cargo release` usage
- troubleshooting

This is understandable when bootstrapping the release process, but it is not the clearest steady-state structure.

### Decision

Split or strongly separate docs into:

1. routine release runbook
2. publishing setup/bootstrap
3. troubleshooting/rationale

The exact filenames can be chosen in implementation, but the separation of purpose should be explicit.

Within that structure:

- the routine runbook should name GitHub Release notes as the canonical human-facing release-notes artifact
- `CHANGELOG.md` should not be required unless the repository later intentionally adopts one
- `cargo release` should move out of the happy path and be documented only as an optional maintainer helper supported by `release.toml`

### Why this is the best fit here

- Release-day maintainers get a short happy path.
- One-time setup stops crowding the routine flow.
- Troubleshooting can expand without bloating the runbook.
- The documented process stays aligned with the true publication boundary: manifest version, green `verify`, pushed tag, publish workflow.

## Current-state facts that must inform implementation

- `clap-tui-macros` is already absent from the workspace and repository tree.
- `.github/workflows/publish-macros.yml` is already absent.
- `release.toml` already matches a tag-driven publish model and is not itself macros-specific.
- `README.md` points maintainers to the current verification script and release docs.
- `docs/release-readiness.md` references `CHANGELOG.md`, which does not exist.
- Canonical OpenSpec specs still describe a two-crate publishing model even though the repo no longer implements one.
- The repository has no separate in-repo release-notes artifact today, so the docs should not imply one is mandatory.

## Migration notes

- Preserve job name `verify`.
- Preserve tag trigger `v*`.
- Preserve OIDC-first publishing.
- Preserve explicit publish gating through `CLAP_TUI_PUBLISH_MODE`.
- Preserve a documented local way to run the tag/version check before pushing.
- Preserve the ability to run baseline verification locally on a dirty worktree.
- Treat GitHub Release notes as the canonical human-facing release-notes artifact.
- Treat `cargo release` as optional tooling, not the authoritative release mechanism.
