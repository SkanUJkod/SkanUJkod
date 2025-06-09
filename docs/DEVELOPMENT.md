# Development Setup Guide

This guide provides detailed instructions for setting up a development environment for SkanUJkod.

## Prerequisites

### System Requirements
- **Operating System**: macOS 10.15+, Linux (Ubuntu 20.04+), Windows 10+
- **RAM**: Minimum 4GB, recommended 8GB+
- **Disk Space**: ~2GB for full development environment
- **Network**: Internet connection for dependency downloads

### Required Software

#### 1. Rust Toolchain
```bash
# Install Rust via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version  # Should show 1.70.0 or later
cargo --version

# Install additional components
rustup component add clippy rustfmt
cargo install cargo-tarpaulin  # For test coverage
```

#### 2. Go Language
```bash
# macOS (using Homebrew)
brew install go

# Linux (Ubuntu/Debian)
sudo apt update
sudo apt install golang-go

# Verify installation
go version  # Should show 1.19 or later
```

#### 3. Development Tools
```bash
# Git (if not already installed)
git --version

# VS Code (recommended IDE)
# Download from https://code.visualstudio.com/

# VS Code Extensions (recommended)
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension ms-vscode.go
```

## Project Setup

### 1. Clone Repository
```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/SkanUJkod.git
cd SkanUJkod

# Add upstream remote
git remote add upstream https://github.com/original-org/SkanUJkod.git

# Verify remotes
git remote -v
```

### 2. Environment Configuration
```bash
# Set essential environment variables
export PLUGINS_DIR=./target/debug
export RUST_LOG=info  # or debug for verbose logging
export RUST_BACKTRACE=1  # For better error traces

# Add to shell profile (bash/zsh)
echo 'export PLUGINS_DIR=./target/debug' >> ~/.bashrc
echo 'export RUST_LOG=info' >> ~/.bashrc
echo 'export RUST_BACKTRACE=1' >> ~/.bashrc
source ~/.bashrc
```

### 3. Build Development Environment
```bash
# Build all crates
cargo build

# Build plugins
./build_plugins.sh

# Run tests to verify setup
cargo test
./test_all_functionality.sh
```

### 4. IDE Configuration

#### VS Code Setup
Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "rust-analyzer.procMacro.enable": true,
    "editor.formatOnSave": true,
    "editor.rulers": [100],
    "files.exclude": {
        "**/target": true,
        "**/.git": true
    }
}
```

Create `.vscode/launch.json` for debugging:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug CLI",
            "cargo": {
                "args": ["build", "--bin=skan-uj-kod"],
                "filter": {
                    "name": "skan-uj-kod",
                    "kind": "bin"
                }
            },
            "args": ["cfg", "--project-path", "./tests/assets"],
            "cwd": "${workspaceFolder}",
            "env": {
                "PLUGINS_DIR": "./target/debug"
            }
        }
    ]
}
```

## Development Workflow

### Daily Development

#### 1. Start Development Session
```bash
# Pull latest changes
git pull upstream main

# Create feature branch
git checkout -b feature/my-feature

# Set environment
export PLUGINS_DIR=./target/debug
```

#### 2. Development Cycle
```bash
# Make changes to code
# ...

# Quick check
cargo check

# Run tests
cargo test

# Build plugins if changed
./build_plugins.sh

# Test integration
./test_all_functionality.sh
```

#### 3. Code Quality Checks
```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings -D clippy::pedantic

# Check all packages
cargo clippy --workspace -- -D warnings

# Run specific tests
cargo test --package cfg_plugin
```

### Plugin Development

#### 1. Create New Plugin
```bash
# Navigate to plugins directory
cd crates/plugins

# Create new plugin
cargo new --lib my_plugin
cd my_plugin

# Configure Cargo.toml (see PLUGIN_DEVELOPMENT.md)
```

#### 2. Plugin Development Cycle
```bash
# Build plugin
cargo build

# Test plugin
cargo test

# Copy to plugins directory
cp target/debug/libmy_plugin.dylib ../../../target/debug/

# Test integration
cd ../../..
./target/debug/skan-uj-kod list  # Should show your plugin
```

### Testing Strategy

#### Unit Tests
```bash
# Run all unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_function_name

# Run tests for specific package
cargo test --package cfg_plugin
```

#### Integration Tests
```bash
# Run integration tests
./test_all_functionality.sh

# Test specific functionality
./target/debug/skan-uj-kod cfg --project-path ./tests/assets
```

#### Coverage Analysis
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/tarpaulin-report.html
```

## Performance Development

### Profiling Setup
```bash
# Install profiling tools
cargo install cargo-flamegraph
cargo install cargo-criterion

# Profile specific test
cargo flamegraph --bin skan-uj-kod -- cfg --project-path ./tests/assets

# Benchmark performance
cargo bench
```

### Memory Analysis
```bash
# Install valgrind (Linux) or instruments (macOS)
# Run with memory checking
valgrind --tool=memcheck ./target/debug/skan-uj-kod cfg --project-path ./tests/assets
```

## Debugging

### Debug Builds
```bash
# Build with debug info
cargo build

# Build with optimizations but debug info
cargo build --profile dev-opt

# Debug with prints
RUST_LOG=debug ./target/debug/skan-uj-kod cfg --project-path ./tests/assets
```

### GDB/LLDB Debugging
```bash
# macOS with LLDB
lldb ./target/debug/skan-uj-kod
(lldb) env PLUGINS_DIR=./target/debug
(lldb) run cfg --project-path ./tests/assets

# Linux with GDB
gdb ./target/debug/skan-uj-kod
(gdb) set environment PLUGINS_DIR=./target/debug
(gdb) run cfg --project-path ./tests/assets
```

### Plugin Debugging
```bash
# Enable plugin debug output
RUST_LOG=debug,skan_uj_kod=trace ./target/debug/skan-uj-kod cfg --project-path ./tests/assets

# Check plugin loading
./target/debug/skan-uj-kod list
```

## Troubleshooting

### Common Issues

#### Build Failures
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check for conflicts
cargo tree --duplicates
```

#### Plugin Issues
```bash
# Verify plugin directory
echo $PLUGINS_DIR
ls -la $PLUGINS_DIR/*.dylib

# Rebuild plugins
./build_plugins.sh

# Check plugin exports
nm -D target/debug/libcfg_plugin.dylib | grep get_library
```

#### ABI Issues
```bash
# Check ABI compatibility
cargo check --package plugin_interface

# Rebuild everything
cargo clean
cargo build --release
./build_plugins.sh
```

#### Go Project Issues
```bash
# Verify Go installation
go version

# Check Go project structure
ls -la /path/to/go/project/go.mod

# Test Go parsing
go fmt /path/to/go/project/...
```

### Debug Environment Variables
```bash
# Enable all debug output
export RUST_LOG=trace
export RUST_BACKTRACE=full
export RUST_LIB_BACKTRACE=1

# Plugin-specific debugging
export PLUGINS_DIR=./target/debug
export PLUGIN_DEBUG=1
```

## Advanced Setup

### Custom Build Configurations

#### Release with Debug Info
```toml
# Add to Cargo.toml
[profile.release-with-debug]
inherits = "release"
debug = true
```

#### Development Optimized
```toml
[profile.dev-opt]
inherits = "dev"
opt-level = 1
debug = true
```

### Docker Development Environment
```dockerfile
# Dockerfile.dev
FROM rust:1.70

RUN apt-get update && apt-get install -y \
    golang-go \
    build-essential \
    pkg-config

WORKDIR /workspace
COPY . .

RUN cargo build
RUN ./build_plugins.sh
```

```bash
# Build development container
docker build -f Dockerfile.dev -t skanujkod-dev .

# Run development container
docker run -it -v $(pwd):/workspace skanujkod-dev bash
```

### Performance Monitoring
```bash
# Install monitoring tools
cargo install cargo-watch
cargo install cargo-expand

# Watch for changes and rebuild
cargo watch -x build

# Monitor for performance regressions
cargo bench --all
```

## Maintenance Tasks

### Regular Maintenance
```bash
# Update dependencies monthly
cargo update
cargo audit

# Check for outdated dependencies
cargo outdated

# Clean up build artifacts
cargo clean
```

### Security Checks
```bash
# Install security audit tool
cargo install cargo-audit

# Run security audit
cargo audit

# Check for vulnerable dependencies
cargo audit --db advisory-db
```

### Documentation Maintenance
```bash
# Generate and check documentation
cargo doc --open

# Check documentation links
cargo doc --all --no-deps

# Update README examples
# (Manual process - verify examples still work)
```

This development setup guide should get you productive quickly while following best practices for Rust development.
