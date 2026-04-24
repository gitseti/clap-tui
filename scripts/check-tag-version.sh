#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/check-tag-version.sh vX.Y.Z

Validates that the pushed release tag matches the version declared in
crates/clap-tui/Cargo.toml.
EOF
}

if [[ "$#" -ne 1 ]]; then
  usage
  exit 1
fi

tag_name="$1"

if [[ "${tag_name}" != v* ]]; then
  echo "Expected a release tag starting with 'v' for clap-tui, got: ${tag_name}" >&2
  exit 1
fi

manifest_path="crates/clap-tui/Cargo.toml"
expected_version="$(
  awk '
    $0 == "[package]" { in_package = 1; next }
    /^\[/ && $0 != "[package]" { in_package = 0 }
    in_package && $1 == "version" {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' "${manifest_path}"
)"

if [[ -z "${expected_version}" ]]; then
  echo "Could not read package.version from ${manifest_path}" >&2
  exit 1
fi

tag_version="${tag_name#v}"

if [[ "${tag_version}" != "${expected_version}" ]]; then
  echo "Release tag ${tag_name} does not match clap-tui version ${expected_version} from ${manifest_path}" >&2
  exit 1
fi

echo "Release tag ${tag_name} matches clap-tui version ${expected_version}"
