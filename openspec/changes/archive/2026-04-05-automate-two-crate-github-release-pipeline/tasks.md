## 1. Extend release helpers

- [x] 1.1 Add `xtask` support for reading the tagged `clap-tui` release plan, including the
      referenced `clap-tui-macros` version.
- [x] 1.2 Add `xtask` support for validating `clap-tui-macros-vX.Y.Z` tags against
      `crates/clap-tui-macros/Cargo.toml`.
- [x] 1.3 Add `xtask` support for checking whether a specific crate version already exists on
      crates.io.

## 2. Enforce the proc-macro prerequisite in the publish workflow

- [x] 2.1 Update `.github/workflows/publish.yml` to compute the release plan and keep
      verification-only behavior when publishing is disabled.
- [x] 2.2 Fail early when the referenced `clap-tui-macros` version is not already published on
      crates.io.
- [x] 2.3 Publish `clap-tui` only after the proc-macro prerequisite check succeeds, keeping
      trusted publishing as the default path and `CRATES_IO_TOKEN` as the fallback.
- [x] 2.4 Add `.github/workflows/publish-macros.yml` for `clap-tui-macros-vX.Y.Z` tags with the
      same credential model and verification-first behavior.

## 3. Align verification and documentation

- [x] 3.1 Update release-readiness documentation to describe the independent `clap-tui-macros`
      workflow, tag format, `clap-tui` prerequisite, skip behavior, and failure expectations.
- [x] 3.2 Adjust any shared verification helpers that need to support the new workflow semantics
      without weakening PR CI checks.
- [x] 3.3 Validate the new workflow locally where possible and record any limits of local
      verification, such as `act` runtime constraints.
- [x] 3.4 Add `cargo-release` repository configuration that matches the workflow tag conventions
      and documents it as a release-prep convenience layer.
