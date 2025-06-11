#!/bin/bash
set -euo pipefail
source .envrc

PLUGINS_DIR="$HOME/.cargo/skanujkod-plugins"
mkdir "$PLUGINS_DIR" 2>/dev/null || true

for item in crates/plugins/*; do
    pushd "$item"
    cargo build
    popd
done

cp target/debug/*.dylib "$PLUGINS_DIR/"
