#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/verify-release-readiness.sh [--allow-dirty] [--publish-dry-run]

Runs the release-readiness verification flow for clap-tui:
  - cargo fmt --all --check
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace --all-targets --all-features
  - cargo package -p clap-tui-macros --list
  - cargo package -p clap-tui --list

Pass --allow-dirty to let the package inspection include the current working tree.
Pass --publish-dry-run to additionally run:
  - cargo publish -p clap-tui-macros --locked --dry-run
  - cargo publish -p clap-tui --locked --dry-run

The publish dry-run only works after the referenced clap-tui-macros version has
already been published to crates.io as an independent release prerequisite.
CI intentionally runs the clean-tree default.
EOF
}

allow_dirty=0
publish_dry_run=0

for arg in "$@"; do
  case "${arg}" in
    --allow-dirty)
      allow_dirty=1
      ;;
    --publish-dry-run)
      publish_dry_run=1
      ;;
    *)
      usage
      exit 1
      ;;
  esac
done

echo "+ cargo fmt --all --check"
cargo fmt --all --check

echo "+ cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "+ cargo test --workspace --all-targets --all-features"
cargo test --workspace --all-targets --all-features

macro_package_cmd=(cargo package -p clap-tui-macros --list)
package_cmd=(cargo package -p clap-tui --list)
if [[ "${allow_dirty}" -eq 1 ]]; then
  macro_package_cmd+=(--allow-dirty)
  package_cmd+=(--allow-dirty)
fi

printf '+'
printf ' %q' "${macro_package_cmd[@]}"
printf '\n'
"${macro_package_cmd[@]}"

printf '+'
printf ' %q' "${package_cmd[@]}"
printf '\n'
"${package_cmd[@]}"

if [[ "${publish_dry_run}" -eq 1 ]]; then
  macro_publish_cmd=(cargo publish -p clap-tui-macros --locked --dry-run)
  publish_cmd=(cargo publish -p clap-tui --locked --dry-run)
  if [[ "${allow_dirty}" -eq 1 ]]; then
    macro_publish_cmd+=(--allow-dirty)
    publish_cmd+=(--allow-dirty)
  fi

  printf '+'
  printf ' %q' "${macro_publish_cmd[@]}"
  printf '\n'
  "${macro_publish_cmd[@]}"

  printf '+'
  printf ' %q' "${publish_cmd[@]}"
  printf '\n'
  "${publish_cmd[@]}"
fi
