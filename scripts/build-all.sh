#!/usr/bin/env bash
set -e

echo "=== Building all ink! contracts ==="

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
CONTRACTS_DIR="$ROOT_DIR/contracts"

for contract in "$CONTRACTS_DIR"/*; do
  if [ -d "$contract" ] && [ -f "$contract/Cargo.toml" ]; then
    echo "-> Building contract: $(basename "$contract")"
    (cd "$contract" && cargo contract build --release)
    echo "✅ Finished: $(basename "$contract")"
    echo
  fi
done

echo "=== All contracts built successfully ==="
