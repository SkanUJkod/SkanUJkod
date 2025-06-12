#!/bin/bash
set -euo pipefail
source .envrc

case $(uname -s) in
    Linux*)     DYNLIB_SUFFIX=".so";;
    Darwin*)    DYNLIB_SUFFIX=".dylib";;
esac

PLUGINS_DIR="$HOME/.cargo/skanujkod-plugins"
mkdir -p "$PLUGINS_DIR" 2>/dev/null || true

pushd "crates/plugins"
for item in *; do
    pushd "$item/iface"
    cargo build
    popd
    cp "../../target/debug/lib"${item}"_plugin"${DYNLIB_SUFFIX} "$PLUGINS_DIR/" || echo "Could not find a library file for $item, skipping..."
done
popd

