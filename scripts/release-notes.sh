#!/usr/bin/env bash
set -euo pipefail

tag="${1:?usage: scripts/release-notes.sh <tag>}"
version="${tag#v}"

if ! grep -qE "^## \\[$version\\]" CHANGELOG.md; then
  echo "no CHANGELOG section for $version" >&2
  exit 1
fi

awk -v version="$version" '
  $0 ~ "^## \\[" version "\\]" { found = 1; print; next }
  found && /^## \[/ { exit }
  found && /^\[[^]]+\]:/ { exit }
  found { print }
' CHANGELOG.md

printf '\n**Container image:** `ghcr.io/%s:%s`\n' "${GITHUB_REPOSITORY:-vasilchenkome/todooshka}" "$tag"
