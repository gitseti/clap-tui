#!/usr/bin/env bash

set -euo pipefail

tree_output="$(cargo tree -p clap-tui -e normal --locked --prefix none --format '{p}')"

normalized_versions() {
  local package_name="$1"
  printf '%s\n' "${tree_output}" \
    | rg "^${package_name} v" \
    | sed 's/ (\*)$//' \
    | sort -u
}

ratatui_versions="$(normalized_versions ratatui)"
crossterm_versions="$(normalized_versions crossterm)"

if [[ -z "${ratatui_versions}" ]]; then
  echo "Expected ratatui to appear in the clap-tui dependency graph." >&2
  exit 1
fi

if [[ -z "${crossterm_versions}" ]]; then
  echo "Expected crossterm to appear in the clap-tui dependency graph." >&2
  exit 1
fi

if [[ "$(printf '%s\n' "${ratatui_versions}" | wc -l | tr -d ' ')" != "1" ]]; then
  echo "Expected exactly one ratatui version in the default dependency graph." >&2
  printf '%s\n' "${ratatui_versions}" >&2
  exit 1
fi

if [[ "$(printf '%s\n' "${crossterm_versions}" | wc -l | tr -d ' ')" != "1" ]]; then
  echo "Expected exactly one crossterm version in the default dependency graph." >&2
  printf '%s\n' "${crossterm_versions}" >&2
  exit 1
fi

if cargo tree -p clap-tui -e normal --locked -i tui-textarea --prefix none --format '{p}' | rg '^crossterm v' >/dev/null; then
  echo "tui-textarea still pulls crossterm into the default dependency graph." >&2
  exit 1
fi

echo "Terminal stack check passed:"
printf '  %s\n' "${ratatui_versions}"
printf '  %s\n' "${crossterm_versions}"
