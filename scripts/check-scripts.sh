#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
shopt -s nullglob

status=0
for f in *.sh; do
  printf 'Checking %s\n' "$f"
  if ! bash -n "$f"; then
    echo "Syntax error in $f" >&2
    status=1
  fi
done

if [[ $status -eq 0 ]]; then
  echo "OK"
fi

exit $status
