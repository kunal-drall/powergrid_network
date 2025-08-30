#!/bin/bash

echo "=== Validating Arithmetic Fixes ==="

# Check if any unsafe arithmetic operations remain
echo "Checking for unsafe arithmetic operations..."

# Look for patterns like: * number) / 
if grep -E "\* [0-9]+\) /" contracts/grid_service/src/lib.rs; then
    echo "❌ Found unsafe multiplication/division patterns"
    exit 1
else
    echo "✅ No unsafe multiplication/division patterns found"
fi

# Look for patterns like: + something + something (without saturating_add)
if grep -E "[a-z_]+ \+ [a-z_]+ \+ [a-z_]+" contracts/grid_service/src/lib.rs | grep -v saturating_add; then
    echo "❌ Found unsafe addition patterns"
    exit 1
else
    echo "✅ No unsafe addition patterns found"
fi

# Look for specific old patterns that should be fixed
if grep -F "load_mw * 100" contracts/grid_service/src/lib.rs; then
    echo "❌ Found old load_mw * 100 pattern"
    exit 1
else
    echo "✅ No old load_mw * 100 pattern found"
fi

if grep -F "as u16 * 250" contracts/grid_service/src/lib.rs; then
    echo "❌ Found old as u16 * 250 pattern"
    exit 1
else
    echo "✅ No old as u16 * 250 pattern found"
fi

echo "=== All arithmetic validations passed! ==="

# Check if the file compiles
echo "Testing compilation..."
cd /home/kunal/powergrid_network
if cargo check -p grid_service; then
    echo "✅ Grid service compiles successfully"
else
    echo "❌ Grid service compilation failed"
    exit 1
fi

echo "=== Validation complete ==="
