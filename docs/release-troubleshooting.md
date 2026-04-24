# clap-tui release troubleshooting

Use this page when the routine flow in [release-readiness.md](release-readiness.md) is not enough.

## Verification-only mode

If `CLAP_TUI_PUBLISH_MODE` is unset or set to an unsupported value, `.github/workflows/publish.yml` still validates the tag, reruns baseline verification, and then stops without calling `cargo publish`.

That is expected behavior while automated publishing is intentionally disabled.

## Local simulation

These commands let you rehearse the release path locally:

```bash
./scripts/verify.sh
./scripts/check-tag-version.sh vX.Y.Z
cargo publish -p clap-tui --locked --dry-run
```

Use `./scripts/verify.sh --allow-dirty` only for local iteration before you have a clean commit. CI and the publish workflow intentionally use the clean-tree default.

## Tag/version mismatch failures

If `./scripts/check-tag-version.sh` fails, the pushed tag does not match `crates/clap-tui/Cargo.toml`. Fix the manifest version or the tag name, then try again.

## GitHub Release pages

GitHub Release notes are the canonical human-facing release-notes artifact, but GitHub Release pages are not the authoritative publish trigger. Creating or editing a GitHub Release page does not publish to crates.io; pushing the `vX.Y.Z` tag does.

## Bad release recovery

If a bad version reaches crates.io:

1. yank the affected version on crates.io
2. fix the problem on the default branch
3. rerun `./scripts/verify.sh`
4. cut a corrected patch release with a new `vX.Y.Z` tag
