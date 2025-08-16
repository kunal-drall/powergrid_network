#!/usr/bin/env bash
set -e

echo "=== Running tests for all ink! contracts ==="

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
CONTRACTS_DIR="$ROOT_DIR/contracts"

for contract in "$CONTRACTS_DIR"/*; do
  if [ -d "$contract" ] && [ -f "$contract/Cargo.toml" ]; then
    echo "-> Testing contract: $(basename "$contract")"
    (cd "$contract" && cargo test)
    echo "✅ Tests passed: $(basename "$contract")"
    echo
  fi
done

echo "=== All contract tests completed successfully ==="
