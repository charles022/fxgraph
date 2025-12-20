#!/bin/bash
set -e

echo ">>> Cleaning generated files..."

# 1. Clean Root Rust artifacts (Server & Client builds)
if [ -d "target" ]; then
    echo "Removing target/..."
    rm -rf target
fi

# 2. Clean WASM Client artifacts
if [ -d "client/pkg" ]; then
    echo "Removing client/pkg/..."
    rm -rf client/pkg
fi

# 3. Clean Web dependencies and artifacts
if [ -d "www/node_modules" ]; then
    echo "Removing www/node_modules/..."
    rm -rf www/node_modules
fi

# Clean www/dist if it exists (potential build output)
if [ -d "www/dist" ]; then
    echo "Removing www/dist/..."
    rm -rf www/dist
fi


# remove Cargo.toml
echo "Removing project Cargo.lock..."
rm -rf Cargo.lock



echo ">>> Clean complete!"
