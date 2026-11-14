#!/usr/bin/env bash
# Границы Clean Architecture: ядро не должно видеть инфраструктуру.
# Падает, если todooshka-domain или todooshka-app потянут axum/sqlx/tokio.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo не найден в PATH" >&2
  exit 2
fi

status=0
for crate in todooshka-domain todooshka-app; do
  leaks="$(
    cargo tree -e normal --prefix none -p "$crate" \
      | awk '{print $1}' \
      | grep -E '^(axum|hyper|sqlx|tokio|tower|tower-http)$' \
      | sort -u || true
  )"

  if [ -n "$leaks" ]; then
    echo "НАРУШЕНИЕ: $crate зависит от инфраструктуры:" >&2
    echo "$leaks" | sed 's/^/  - /' >&2
    status=1
  fi
done

if [ "$status" -eq 0 ]; then
  echo "границы чистые: domain и application не зависят от axum/sqlx/tokio"
fi

exit "$status"
