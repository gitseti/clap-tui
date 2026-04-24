# clap-tui publishing setup

Use this page for one-time or infrequent repository publishing setup. The routine release flow lives in [release-readiness.md](release-readiness.md).

## 1. Keep the stable CI gate in place

The GitHub Actions verification workflow exposes a stable required job named `verify`. Keep that job configured as a required status check in branch protection for the default branch.

## 2. Confirm crates.io ownership

Before relying on automated publishing:

- confirm the intended crates.io owners for `clap-tui`
- ensure the publishing account or organization is already recorded as an owner for `clap-tui`

## 3. Choose the publishing mode

The publish workflow is controlled by the repository variable `CLAP_TUI_PUBLISH_MODE`.

- Leave it unset to keep the tag workflow in verification-only mode.
- Set it to `trusted-publishing` after crates.io trusted publishing is registered for `.github/workflows/publish.yml` using the `release` environment.
- Set it to `token` only when trusted publishing cannot yet be configured and the repository secret `CRATES_IO_TOKEN` is available as the explicit fallback credential.

Trusted publishing is the preferred mode because it avoids long-lived credentials.

## 4. Register trusted publishing

If you use GitHub OIDC trusted publishing on crates.io, register:

- repository: `gitseti/clap-tui`
- workflow file: `publish.yml`
- environment: `release`

After registration, set `CLAP_TUI_PUBLISH_MODE=trusted-publishing`.

## 5. First publish and fallback setup

Before relying on the tag workflow for real publication, maintainers should:

- run `./scripts/verify.sh`
- run `cargo publish -p clap-tui --locked --dry-run`
- complete the first real `clap-tui` crates.io release through the chosen credential path if crates.io setup still requires a manual bootstrap step

If trusted publishing is unavailable, set `CLAP_TUI_PUBLISH_MODE=token` and configure `CRATES_IO_TOKEN` as the fallback secret.

## 6. Optional: use cargo-release as a helper

The root [release.toml](../release.toml) is available for maintainers who prefer `cargo release` to help with version bumps and tag creation. It is not the authoritative publishing mechanism for this repository.

`cargo release` remains optional because the real publish boundary is still:

1. version updated in the manifest
2. `verify` green on the merged commit
3. `vX.Y.Z` tag pushed
4. `.github/workflows/publish.yml` runs

If you want to use `cargo release`, install it first:

```bash
cargo install cargo-release
```

Then use it as a helper rather than a required release step:

```bash
cargo release -p clap-tui --dry-run <level-or-version>
cargo release -p clap-tui <level-or-version>
```
