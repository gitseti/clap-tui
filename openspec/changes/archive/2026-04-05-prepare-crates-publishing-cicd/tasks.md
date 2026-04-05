## 1. Crate metadata and release docs

- [x] 1.1 Record the canonical GitHub repository URL and intended crates.io owners, and block final publish metadata work until those real values are known.
- [x] 1.2 Update the workspace, `crates/clap-tui/Cargo.toml`, and `crates/clap-tui-macros/Cargo.toml` metadata with `description`, `readme`, `repository`, `homepage`, `documentation`, `license`, `rust-version`, `keywords`, and `categories` using real repository values with no placeholders.
- [x] 1.3 Add `CHANGELOG.md`, maintainer release instructions, and README updates that document the crates' public release surface, the proc-macro prerequisite, the first manual releases, crates.io owner setup, and the move to automated publishing afterward.

## 2. Packaging verification

- [x] 2.1 Add a repeatable local verification command or script for `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, package-surface verification for `clap-tui-macros` and `clap-tui`, and an opt-in `cargo publish -p clap-tui --locked --dry-run` mode for use after the referenced proc-macro version is published.
- [x] 2.2 Run the verification flow locally, fix any issues it reveals, and document the expected success path for maintainers.

## 3. GitHub verification workflow

- [x] 3.1 Add a GitHub Actions verification workflow for pushes and pull requests on Linux with a stable required job name `verify` that runs the documented Rust verification checks.
- [x] 3.2 Keep the verification workflow scoped to the Linux stable toolchain in this change and avoid adding non-Linux or MSRV matrix jobs.
- [x] 3.3 Document that maintainers must configure the `verify` job as a required status check in GitHub branch protection so pull requests are actually gated.

## 4. Release workflow and publishing posture

- [x] 4.1 Add `.github/workflows/publish.yml` to trigger on pushed `v*` tags, validate that the tag version matches `crates/clap-tui/Cargo.toml`, rerun pre-publish verification, and only publish from that tagged revision after the proc-macro prerequisite and trusted publishing setup are complete.
- [x] 4.2 Wire the publish workflow for GitHub OIDC trusted publishing with `id-token: write` and `rust-lang/crates-io-auth-action@v1`, and document `CRATES_IO_TOKEN` as the explicit fallback when trusted publishing cannot be configured.
- [x] 4.3 Document the end-to-end release checklist, including version bumping, changelog updates, confirming a green `verify` check before cutting a `vX.Y.Z` tag, publishing any new `clap-tui-macros` version first, the first manual `clap-tui` release, trusted publisher registration for `.github/workflows/publish.yml`, and rollback or yank guidance.

## 5. Final validation

- [x] 5.1 Verify that the repository documentation, GitHub workflows, branch-protection instructions, and Cargo metadata are consistent with the new release process.
- [x] 5.2 Perform a final dry-run readiness pass and confirm there are no remaining placeholders or unresolved choices in the publish path.
