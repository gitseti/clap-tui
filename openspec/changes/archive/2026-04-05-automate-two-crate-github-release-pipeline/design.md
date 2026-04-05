## Context

The repository already has two pieces of the release story in place:

- `ci.yml` provides the stable `verify` job used for pull request gating.
- `publish.yml` validates tags, reruns verification, dry-runs `clap-tui-macros`, and can publish
  `clap-tui` when publishing is enabled.

The remaining gap is that `clap-tui` depends on `clap-tui-macros`, but the GitHub release workflow
does not yet make that dependency an explicit publish prerequisite. The workflow needs to handle
two cases cleanly:

1. the referenced proc-macro version is already on crates.io and can be reused
2. the proc-macro version is missing and maintainers need a clear failure that tells them to
   publish it independently before retrying the `clap-tui` release

This change should preserve the current verification-first posture and trusted-publishing default
while turning the enabled path into two independent publish workflows: one for
`clap-tui-macros`, and one for `clap-tui` with a checked proc-macro prerequisite.

## Goals / Non-Goals

**Goals:**

- Compute the tagged `clap-tui` release plan, including the referenced `clap-tui-macros` version.
- Fail fast if the required `clap-tui-macros` version is not already published on crates.io.
- Keep `publish.yml` focused on publishing `clap-tui`, not multiple crates in one run.
- Add a dedicated `publish-macros.yml` workflow for `clap-tui-macros` with an unambiguous tag
  format.
- Add a `cargo-release` configuration layer for maintainers so release prep commands produce the
  same tag names and dependency updates that the GitHub workflows expect.
- Keep release behavior explicit in documentation and aligned with the GitHub workflow.

**Non-Goals:**

- Changing the crate versioning model beyond what the current manifests already declare.
- Mechanically enforcing branch-protection history inside the publish workflow.
- Replacing the existing local verification entrypoint unless the new flow makes that necessary.
- Publishing both crates from a single GitHub workflow run.

## Decisions

### Keep `publish.yml` as a single main-crate publish job

The tag workflow will stay in `.github/workflows/publish.yml` as a single publish-oriented job. It
will:

1. validate the tag
2. rerun baseline verification
3. compute the `clap-tui` release plan
4. fail early if the referenced `clap-tui-macros` version is not already on crates.io
5. dry-run the main crate publish path
6. publish `clap-tui`

This keeps the workflow readable and focused on the crate it actually publishes while still making
the proc-macro dependency a first-class release gate.

Alternative considered: a multi-job two-crate pipeline. Rejected because it adds crates.io timing
complexity and turns an independently publishable support crate into part of a single release
transaction.

### Add a separate `publish-macros.yml` workflow

`clap-tui-macros` will get its own publish workflow at `.github/workflows/publish-macros.yml`.
That workflow will:

1. trigger only on `clap-tui-macros-vX.Y.Z` tags
2. validate that the tag matches `crates/clap-tui-macros/Cargo.toml`
3. rerun baseline verification
4. dry-run the proc-macro publish path
5. publish `clap-tui-macros`

Using a dedicated tag format avoids collisions with `clap-tui` tags and makes it clear which crate
is being released.

Alternative considered: reuse `vX.Y.Z` tags for both crates. Rejected because a shared tag space
would make workflow routing ambiguous and blur the release boundary between the two crates.

### Put structured release helpers in `xtask`

The release flow now needs manifest parsing and a crates.io version check. That is still a better
fit for Rust than Bash. The `xtask` crate will grow subcommands for:

- reading the tagged `clap-tui` version and referenced `clap-tui-macros` version
- validating `clap-tui-macros-vX.Y.Z` tags against the proc-macro manifest
- checking whether a specific crate version already exists on crates.io

Alternative considered: more shell scripts with `awk`, `curl`, and retry loops. Rejected because
the tag parsing cleanup already moved in the opposite direction and the new logic is more brittle in
shell.

### Use `cargo-release` for maintainer-side release prep only

The repository will include a small root `release.toml` and per-crate `package.metadata.release`
tag names so maintainers can use `cargo release -p ...` to prepare bumps and tags consistently.
This configuration will stop short of local publication by setting `disable-publish = true`,
because GitHub Actions remains the publish boundary.

Alternative considered: leave all version prep fully manual. Rejected because the repo now has
crate-specific tag schemes and exact dependent-version updates that are easy to get wrong by hand.

### Treat proc-macro publication as an external prerequisite

The automated workflow should not try to publish `clap-tui-macros` itself. Instead, it will inspect
the version referenced by `clap-tui` and query crates.io:

- if that proc-macro version already exists, continue with the normal `clap-tui` dry-run and
  publish steps
- if it does not exist, fail before authentication and tell maintainers to publish the proc-macro
  crate independently

This keeps the workflow aligned with the actual release boundary while still protecting against the
most common dependency mistake.

Alternative considered: publish the proc-macro crate in the same workflow and then wait for index
propagation. Rejected because it adds complexity and is unnecessary when the proc-macro crate can be
released independently.

## Risks / Trade-offs

- [Additional `xtask` code adds maintenance overhead] → Keep helper commands small and focused on
  release concerns already owned by the repo.
- [Proc-macro prerequisite failures may surprise maintainers] → Surface the computed release plan in
  workflow logs and give a specific failure message that points to the independent proc-macro
  release step.
- [crates.io API availability becomes part of the release path] → Keep the published-version check
  narrow and run it only when automated publishing is enabled.

## Migration Plan

1. Extend `xtask` with release-plan, proc-macro tag validation, and published-version helpers.
2. Update `publish.yml` to compute the release plan and enforce the proc-macro prerequisite before
   publishing `clap-tui`.
3. Add `publish-macros.yml` for `clap-tui-macros-vX.Y.Z` tags.
4. Keep verification-only mode intact for repositories that have not enabled automated publishing
   yet.
5. Add `cargo-release` configuration that matches the workflow tag conventions without taking over
   publishing.
6. Update maintainer documentation to describe both workflows, the macro tag format, the
   proc-macro prerequisite for `clap-tui`, and the `cargo-release` prep flow.
7. Validate the workflows locally as far as possible and with dry-runs before enabling automated
   publishing in GitHub.

Rollback: set `CLAP_TUI_PUBLISH_MODE` back to an unset state to return the tag workflow to
verification-only behavior while keeping the rest of the release-preparation checks intact.

## Open Questions

- Should the workflow expose release-plan data only in logs and step outputs, or also as an
  artifact for debugging?
