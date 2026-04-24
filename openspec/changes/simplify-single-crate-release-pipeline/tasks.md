## 1. Align canonical release specs with the single-crate repository

- [ ] 1.1 Update the `github-release-pipeline` spec delta to remove proc-macro workflow requirements, preserve the stable `verify` job, and define the single-crate tag-driven publish path.
- [ ] 1.2 Update the `crate-publishing-readiness` spec delta so it applies to `clap-tui` only, defines the shared baseline verification contract, and documents the separated release runbook/setup/troubleshooting structure.
- [ ] 1.3 Validate that the new change fully supersedes stale two-crate assumptions that still live in canonical OpenSpec requirements.

## 2. Simplify repository verification entry points

- [ ] 2.1 Replace `scripts/verify-release-readiness.sh` with a baseline-only shared verification entry point whose name reflects general repository verification, and update all callers and docs accordingly.
- [ ] 2.2 Keep baseline verification limited to format, lint, tests, terminal-stack validation, and `cargo package -p clap-tui --list`, while preserving an `--allow-dirty` local mode if still needed.
- [ ] 2.3 Remove publish dry-run behavior from the shared baseline script and make publish dry-run an explicit release-preflight command instead.

## 3. Simplify tag validation and publish workflow structure

- [ ] 3.1 Add a tiny repository script for tag/version validation, migrate `.github/workflows/publish.yml` and maintainer docs to it, and only then remove `xtask` if nothing else still justifies it.
- [ ] 3.2 Update `.github/workflows/publish.yml` so it reflects the single-crate model only: tag/version validation, baseline verification rerun, explicit publish dry-run when publishing is enabled, and actual publish with trusted-publishing-first behavior.
- [ ] 3.3 Preserve verification-only mode when `CLAP_TUI_PUBLISH_MODE` is unset or unsupported, with clear workflow messaging.
- [ ] 3.4 Remove the `xtask` workspace member and related files if no longer justified after the tag/version check migration.

## 4. Reduce redundant CI behavior

- [ ] 4.1 Update `.github/workflows/ci.yml` so the stable `verify` job still runs for pull requests and normal pushes while avoiding redundant `v*` tag-triggered verification if the publish workflow already reruns the baseline contract.
- [ ] 4.2 Confirm that branch-protection guidance and required-check documentation still point at the unchanged `verify` job name.

## 5. Reorganize maintainer release documentation

- [ ] 5.1 Rewrite the maintainer release docs into a short routine release runbook for the single-crate `clap-tui` flow.
- [ ] 5.2 Move one-time publishing setup details into a separate setup/bootstrap document or clearly separated section, including crates.io owners, trusted publishing registration, `CLAP_TUI_PUBLISH_MODE`, and token fallback.
- [ ] 5.3 Move troubleshooting and local simulation guidance into a separate troubleshooting/rationale document or clearly separated section.
- [ ] 5.4 Make GitHub Release notes the canonical human-facing release-notes artifact, remove any language that implies `CHANGELOG.md` is required, and keep any in-repo release-notes summary optional unless the repository adopts one intentionally.
- [ ] 5.5 Document `cargo release` as an optional maintainer helper supported by `release.toml`, not as a required step in the canonical release happy path.
- [ ] 5.6 Update `README.md` and any maintainer-facing references so they point to the renamed verification entry point and the new release-doc structure.

## 6. Final verification

- [ ] 6.1 Search the repository for remaining active references to `clap-tui-macros`, proc-macro release prerequisites, `verify-release-readiness`, and `xtask check-tag-version`, and resolve the ones that should no longer exist.
- [ ] 6.2 Run the relevant OpenSpec validation and repository verification steps to confirm the simplified release/CI model is internally consistent.
