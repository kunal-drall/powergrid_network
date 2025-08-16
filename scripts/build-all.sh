#!/bin/bash
set -e

echo "=== Building all ink! contracts ==="

CONTRACTS=("governance" "grid_service" "resource_registry" "token")

for contract in "${CONTRACTS[@]}"; do
  echo "-> Building contract: $contract"
  pushd contracts/$contract > /dev/null
  cargo clippy --all-targets --all-features
  cargo contract build --release
  popd > /dev/null
  echo "✅ Finished: $contract"
done

echo "=== All contract builds completed successfully ==="
