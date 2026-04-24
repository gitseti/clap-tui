#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/verify.sh [--allow-dirty]

Runs the baseline repository verification flow for clap-tui:
  - cargo fmt --all --check
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace --all-targets --all-features
  - ./scripts/check-terminal-stack.sh
  - cargo package -p clap-tui --list

Pass --allow-dirty to let the package inspection include the current working tree.
CI and tag workflows intentionally run the clean-tree default.
EOF
}

allow_dirty=0

for arg in "$@"; do
  case "${arg}" in
    --allow-dirty)
      allow_dirty=1
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

package_cmd=(cargo package -p clap-tui --list)
if [[ "${allow_dirty}" -eq 1 ]]; then
  package_cmd+=(--allow-dirty)
fi

echo "+ ./scripts/check-terminal-stack.sh"
./scripts/check-terminal-stack.sh

printf '+'
printf ' %q' "${package_cmd[@]}"
printf '\n'
"${package_cmd[@]}"
