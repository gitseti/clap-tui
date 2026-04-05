## Context

`clap-tui` is a Rust workspace with a public library crate and a dependent proc-macro crate that are already usable locally, but their packaging and release posture is incomplete. The `clap-tui` manifest now has the main discovery metadata filled in, while `clap-tui-macros` is still missing repository-facing metadata such as readme, repository, homepage, documentation, keywords, and categories. The repository also has no changelog and no documented release sequence that explains when the proc-macro crate must be published before the library crate's publish dry-run can succeed.

The project is a library rather than an application, so the main operational goal is a repeatable publish flow instead of binary artifact deployment. Because the crate uses terminal UI dependencies such as `crossterm`, the verification design should keep cross-platform confidence in mind without making the initial release process too heavy.

## Goals / Non-Goals

**Goals:**
- Make `crates/clap-tui` and the release-critical `crates/clap-tui-macros` dependency ready for crates.io release with complete manifest metadata and packaging validation once the canonical GitHub repository URL is known.
- Add GitHub verification that catches formatting, linting, test, and packaging regressions before release changes merge.
- Establish a release workflow that is safe for a first public release and then uses low-secret automated publishing for `clap-tui` once the dependent proc-macro version already exists on crates.io.
- Document the release checklist so maintainers can repeat the process consistently, including required GitHub settings outside the repo.

**Non-Goals:**
- Introduce a full multi-crate release orchestration system.
- Add binary packaging or GitHub Release asset distribution for platform-specific executables.
- Automate semantic version calculation or changelog generation in this first pass.
- Support nightly-only tooling or a complex matrix that slows down routine contribution checks excessively.

## Decisions

### Decision: Scope automation to the library crate, but treat the proc-macro crate as a release prerequisite
The workspace contains a primary library crate and a proc-macro dependency crate. This change will keep the GitHub release workflow focused on `crates/clap-tui`, but it must also make `crates/clap-tui-macros` publish-ready because `cargo publish -p clap-tui --dry-run` cannot succeed until the referenced proc-macro version is already on crates.io. Maintainer documentation will therefore require publishing any new `clap-tui-macros` version manually before relying on the `clap-tui` publish path.

Alternatives considered:
- Build a generic multi-crate release pipeline now. Rejected because the current repo does not need the extra abstraction.
- Ignore the proc-macro crate in release planning. Rejected because it leaves the documented publish verification path impossible to satisfy.

### Decision: Split release readiness into repository metadata, verification, and documentation
The implementation will update Cargo metadata, add a `CHANGELOG.md` and release instructions, and make packaging validation part of automation. Treating these as one release-readiness unit avoids the common failure mode where the manifest becomes publishable but the operational steps remain tribal knowledge.

Alternatives considered:
- Only update `Cargo.toml`. Rejected because it leaves the release process fragile and hard to repeat.

### Decision: Use a two-tier GitHub workflow model
The repository will use one verification workflow for pushes and pull requests, and one release workflow for pushed `v*` tags. The verification workflow will expose a stable required job named `verify` and run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, and package-surface verification for both `clap-tui-macros` and `clap-tui`. The shared verification script will also provide an opt-in `cargo publish -p clap-tui --locked --dry-run` mode for the point when the referenced proc-macro version has already been published. Maintainer documentation will require that the `verify` job be configured as a required status check in GitHub branch protection.

The release workflow will live at `.github/workflows/publish.yml`, trigger on pushed `vX.Y.Z` tags, strip the leading `v`, validate that the tag version matches `crates/clap-tui/Cargo.toml`, rerun pre-publish verification, and only later grow into actual crates.io publication after the documented proc-macro prerequisite and trusted publishing setup are complete. Maintainer release instructions will require that tags be created only from merged commits whose required `verify` check has already passed, so the tag-driven publish path inherits the same verification gate as pull requests.

This separation keeps contributor feedback fast while ensuring publication logic is isolated behind an intentional trigger.

Alternatives considered:
- Publish directly from a catch-all CI workflow. Rejected because it increases the chance of accidental release behavior.
- Use GitHub release events as the primary trigger. Rejected because pushed version tags are simpler to validate against `Cargo.toml` and map cleanly to crates.io publish intent.
- Use only manual local publishing. Rejected because it leaves too much room for environment drift and missed checks.

### Decision: Use manual first release, then trusted publishing by default
The first crates.io release will remain manual because trusted publishing requires the crate to exist before the GitHub workflow can be registered as a trusted publisher. In practice, the first public release sequence is `clap-tui-macros` first, then `clap-tui`, because the library crate's publish dry-run resolves the proc-macro dependency from crates.io. Once automated publishing exists for real, the release path should publish `clap-tui-macros`, wait for the new version to appear in the crates.io index, and only then publish `clap-tui`. After that first release, the default automation path will use GitHub OIDC via `rust-lang/crates-io-auth-action@v1` with `id-token: write` to obtain a temporary crates.io token at publish time. If trusted publishing cannot be configured yet, maintainer documentation will define a fallback using a repository secret named `CRATES_IO_TOKEN`.

Alternatives considered:
- Use a long-lived token from the start. Rejected because trusted publishing better matches the goal of minimizing standing secrets.
- Fully automate publishing on every version tag before the first manual release. Rejected because crates.io trusted publishing setup depends on the crate already existing.

### Decision: Keep the verification matrix pragmatic
The required checks will run on Linux with the project's stable toolchain and include packaging validation. Non-Linux smoke builds and MSRV verification are explicitly out of scope for this change so the initial release pipeline stays small, stable, and easy to operate.

Alternatives considered:
- Require a broad OS and toolchain matrix from day one. Rejected because it adds maintenance overhead before the baseline release path exists.

## Risks / Trade-offs

- [Canonical repository URL is not configured locally] → Treat the real GitHub repository URL and crates.io owner setup as prerequisites for completing publish metadata; do not use placeholders.
- [The library crate depends on an unpublished proc-macro crate version] → Document and validate the release order so maintainers publish `clap-tui-macros` before relying on `clap-tui` publish dry-runs, and add a wait step for crates.io index visibility before any future automated `clap-tui` publish.
- [Trusted publishing setup depends on a successful first release] → Keep the first release manual and document the exact handoff to automated publishing.
- [Cross-platform behavior may differ from Linux CI] → Keep Linux verification required in v1 and defer extra platform coverage to a later change.
- [Token-based publishing can create secret management burden] → Prefer trusted publishing when available and document the fallback path explicitly.
- [More release checks can slow feedback loops] → Keep heavy release-only steps out of the main PR workflow unless they protect publishability.

## Migration Plan

1. Record the canonical GitHub repository URL and intended crates.io owners so final publish metadata can be written without placeholders.
2. Update workspace and crate metadata so both published crates describe their public release surface.
3. Add release-facing docs such as `CHANGELOG.md`, maintainer instructions, required status-check documentation, and first-release guidance, including the proc-macro prerequisite.
4. Introduce the verification workflow and make it green on the current codebase.
5. Introduce the tag-driven publish workflow and document the OIDC-first plus `CRATES_IO_TOKEN` fallback setup.
6. Perform the first manual `clap-tui-macros` release, then the first manual `clap-tui` release, register trusted publishing for `.github/workflows/publish.yml`, and use the documented checklist for subsequent automated releases.

## Open Questions

- None. The canonical GitHub repository URL is `https://github.com/gitseti/clap-tui`, and the intended crates.io owner for the first release path is `gitseti`.
