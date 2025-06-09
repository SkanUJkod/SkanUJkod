# SkanUJkod - Modular Static Analysis Tool for Go

[![Build Status](https://github.com/your-org/SkanUJkod/workflows/Build%20cargo%20project/badge.svg)](https://github.com/your-org/SkanUJkod/actions)

SkanUJkod is a modular static analysis framework for Go projects, built with Rust and featuring a plugin-based architecture. It provides comprehensive code analysis including Control Flow Graph (CFG) generation, coverage analysis, and complexity metrics.

## 🚀 Features

- **Control Flow Graph (CFG) Generation** - Generate visual representations of code flow
- **Branch Coverage Analysis** - Analyze test coverage for conditional branches
- **Statement Coverage Analysis** - Track execution of individual statements
- **Cyclomatic Complexity Analysis** - Measure code complexity and maintainability
- **Plugin Architecture** - Extensible design for custom analyses
- **Multiple Output Formats** - DOT, JSON, and text formats
- **CLI Interface** - Easy-to-use command-line interface

## 📋 Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Architecture](#architecture)
- [Plugin Development](#plugin-development)
- [Contributing](#contributing)
- [Project Status](#project-status)
- [License](#license)

## 🛠 Installation

### Prerequisites

- **Rust** (1.70.0 or later) - [Install Rust](https://rustup.rs/)
- **Go** (1.19 or later) - [Install Go](https://golang.org/doc/install)
- **Git** - For cloning the repository

### Building from Source

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/SkanUJkod.git
   cd SkanUJkod
   ```

2. **Build the project:**
   ```bash
   # Build in release mode for better performance
   cargo build --release

   # Build all plugins
   ./build_plugins.sh
   ```

3. **Set up environment:**
   ```bash
   # Set plugin directory (required)
   export PLUGINS_DIR=./target/release

   # Add to your shell profile for persistence
   echo 'export PLUGINS_DIR=./target/release' >> ~/.bashrc
   ```

4. **Verify installation:**
   ```bash
   ./target/release/skan-uj-kod --help
   ```

## 🚀 Quick Start

Analyze a Go project with all available analyses:

```bash
# Run all analyses on a Go project
./target/release/skan-uj-kod all --project-path /path/to/go/project --output-dir ./results

# Generate CFG visualization
./target/release/skan-uj-kod cfg --project-path /path/to/go/project --output cfg.dot

# Check code complexity
./target/release/skan-uj-kod complexity --project-path /path/to/go/project --max-complexity 10
```

## 📖 Usage

### Command Line Interface

SkanUJkod provides several subcommands for different types of analysis:

#### Control Flow Graph Analysis
```bash
./target/release/skan-uj-kod cfg [OPTIONS] --project-path <PATH>

Options:
  -p, --project-path <PATH>    Path to Go project directory
  -o, --output <FILE>          Output file (default: stdout)
  -f, --format <FORMAT>        Output format: dot, json, text [default: dot]
      --include-tests          Include test files in analysis
      --exclude <PATTERN>      File patterns to exclude (glob patterns)
```

#### Branch Coverage Analysis
```bash
./target/release/skan-uj-kod branch-cov [OPTIONS] --project-path <PATH>

Options:
  -p, --project-path <PATH>    Path to Go project directory
  -o, --output <FILE>          Output file (default: stdout)
  -t, --threshold <FLOAT>      Coverage threshold 0.0-1.0 [default: 0.8]
      --include-tests          Include test files in analysis
      --exclude <PATTERN>      File patterns to exclude
```

#### Statement Coverage Analysis
```bash
./target/release/skan-uj-kod statement-cov [OPTIONS] --project-path <PATH>

Options:
  -p, --project-path <PATH>    Path to Go project directory
  -o, --output <FILE>          Output file (default: stdout)
  -t, --threshold <FLOAT>      Coverage threshold 0.0-1.0 [default: 0.8]
      --include-tests          Include test files in analysis
      --exclude <PATTERN>      File patterns to exclude
```

#### Cyclomatic Complexity Analysis
```bash
./target/release/skan-uj-kod complexity [OPTIONS] --project-path <PATH>

Options:
  -p, --project-path <PATH>    Path to Go project directory
  -o, --output <FILE>          Output file (default: stdout)
  -m, --max-complexity <INT>   Maximum allowed complexity [default: 10]
      --include-tests          Include test files in analysis
      --exclude <PATTERN>      File patterns to exclude
```

#### Run All Analyses
```bash
./target/release/skan-uj-kod all [OPTIONS] --project-path <PATH>

Options:
  -p, --project-path <PATH>    Path to Go project directory
  -o, --output-dir <DIR>       Output directory for all results
      --include-tests          Include test files in analysis
      --exclude <PATTERN>      File patterns to exclude
```

### Example Usage

```bash
# Analyze a simple Go project
./target/release/skan-uj-kod cfg --project-path ./examples/simple-go

# Generate coverage report with custom threshold
./target/release/skan-uj-kod branch-cov --project-path ./my-project --threshold 0.9 --output coverage.txt

# Check complexity excluding test files
./target/release/skan-uj-kod complexity --project-path ./my-project --exclude "*_test.go" --max-complexity 15

# Complete analysis with results saved to directory
./target/release/skan-uj-kod all --project-path ./my-project --output-dir ./analysis-results --include-tests
```

## 🏗 Architecture

SkanUJkod uses a modular plugin-based architecture built on Rust's type system and ABI stability:

### Core Components

1. **Kernel/Core** (`src/`)
   - CLI interface and argument parsing
   - Plugin manager for dynamic loading
   - Command execution pipeline
   - Result formatting and output

2. **Plugin Interface** (`crates/plugin_interface/`)
   - ABI-stable interface definitions
   - Plugin function types and connectors
   - Cross-plugin communication protocol

3. **Go Parser** (`crates/go-parser/`)
   - Go AST parsing and manipulation
   - File system traversal and filtering
   - Source code analysis utilities

4. **Analysis Plugins** (`crates/plugins/`)
   - `cfg_plugin` - Control Flow Graph generation
   - `branch_cov_plugin` - Branch coverage analysis
   - `statement_cov_plugin` - Statement coverage analysis
   - `cyclomatic_complexity_plugin` - Complexity metrics

### Plugin System

The plugin architecture uses:
- **Dynamic Loading**: Plugins are loaded as shared libraries (.dylib/.so/.dll)
- **ABI Stability**: Uses `abi_stable` crate for cross-version compatibility
- **Dependency Resolution**: Automatic topological sorting of plugin dependencies
- **Type Safety**: Compile-time guarantees for plugin interfaces

### Data Flow

```
Go Source Files → Parser → AST → Analysis Plugins → Results → Output Formatters
```

1. **Input Processing**: Go files are parsed into Abstract Syntax Trees (AST)
2. **Plugin Execution**: Plugins run in dependency order with result sharing
3. **Result Aggregation**: Analysis results are collected and formatted
4. **Output Generation**: Results exported in requested formats (DOT, JSON, text)

## 🔌 Plugin Development

### Creating a New Plugin

1. **Create plugin crate:**
   ```bash
   cargo new --lib crates/plugins/my_plugin
   cd crates/plugins/my_plugin
   ```

2. **Configure Cargo.toml:**
   ```toml
   [package]
   name = "my_plugin"
   version = "0.1.0"
   edition = "2021"

   [lib]
   name = "my_plugin"
   crate-type = ["lib", "cdylib"]

   [dependencies.abi_stable]
   git = "https://github.com/rodrimati1992/abi_stable_crates.git"

   [dependencies.plugin_interface]
   path = "../../plugin_interface"
   features = ["impls"]

   [dependencies.go-parser]
   path = "../../go-parser"
   ```

3. **Implement plugin functions:**
   ```rust
   use abi_stable::{export_root_module, sabi_extern_fn, rvec};
   use plugin_interface::{Plugin, PluginRef, PFConnector, /* ... */};

   #[export_root_module]
   pub fn get_library() -> PluginRef {
       Plugin { funcs: new_pf_vec }.leak_into_prefix()
   }

   #[sabi_extern_fn]
   fn new_pf_vec() -> RVec<PFConnector> {
       rvec![
           PFConnector {
               pf: PluginFunction(my_analysis_function),
               pf_type: PFType {
                   pf_dependencies: rvec![/* dependencies */],
                   user_params: rvec![/* required parameters */]
               },
               pf_id: QualPFID {
                   plugin_id: "my_plugin".into(),
                   pf_id: "my_function".into()
               }
           }
       ]
   }

   #[sabi_extern_fn]
   fn my_analysis_function(
       pf_results: PFDependencies,
       user_params: &UserParameters
   ) -> BoxedPFResult<'static> {
       // Implementation here
   }
   ```

### Plugin Interface Guidelines

- **Dependencies**: Declare plugin dependencies in `pf_dependencies`
- **Parameters**: Specify required user parameters in `user_params`
- **Error Handling**: Use `Result` types for error propagation
- **ABI Compatibility**: Only use ABI-stable types in public interfaces
- **Testing**: Include comprehensive tests for plugin functionality

### Building and Testing Plugins

```bash
# Build specific plugin
cd crates/plugins/my_plugin
cargo build --release

# Test plugin integration
cd ../../..
cargo test

# Test CLI integration
./target/release/skan-uj-kod list  # Should show your plugin
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --package cfg_plugin
cargo test --package plugin_interface

# Run integration tests
./test_all_functionality.sh
```

### Test Coverage

```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

### Manual Testing

Use the provided test script to verify all functionality:

```bash
# Make script executable
chmod +x test_all_functionality.sh

# Run comprehensive tests
./test_all_functionality.sh
```

## 🤝 Contributing

### Development Setup

1. **Fork and clone the repository**
2. **Create a feature branch:**
   ```bash
   git checkout -b feature/my-feature
   ```
3. **Make changes and test:**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   ./test_all_functionality.sh
   ```
4. **Commit and push:**
   ```bash
   git commit -m "feat: add new analysis feature"
   git push origin feature/my-feature
   ```
5. **Create a Pull Request**

### Code Standards

- **Rust Edition**: 2021
- **Linting**: Use `cargo clippy` with strict settings
- **Formatting**: Use `cargo fmt`
- **Documentation**: Document public APIs with rustdoc
- **Testing**: Maintain high test coverage

### Plugin Development Guidelines

- Follow the existing plugin patterns
- Ensure ABI compatibility
- Add comprehensive tests
- Update documentation
- Test with real Go projects

## 📊 Project Status

### Current Status: **Production Ready** ✅

### Completed Features
- ✅ Plugin-based architecture
- ✅ CFG generation (DOT format)
- ✅ Branch coverage analysis
- ✅ Statement coverage analysis
- ✅ Cyclomatic complexity analysis
- ✅ CLI interface
- ✅ ABI stability
- ✅ Comprehensive testing

### Known Limitations
- 🔄 JSON/Text output formats partially implemented
- 🔄 Advanced Go language features (generics, reflection) limited support
- 🔄 IDE integration not available
- 🔄 Incremental analysis not supported

### Future Roadmap
- [ ] JSON and text output formatters
- [ ] Web-based visualization dashboard
- [ ] IDE plugins (VS Code, GoLand)
- [ ] Incremental analysis support
- [ ] Advanced Go language feature support
- [ ] Custom rule definition language
- [ ] Performance optimizations
- [ ] Distributed analysis support

## 🛠 Troubleshooting

### Common Issues

**Plugin not found errors:**
```bash
# Ensure PLUGINS_DIR is set correctly
export PLUGINS_DIR=./target/release
echo $PLUGINS_DIR
ls $PLUGINS_DIR/*.dylib  # Should show plugin files
```

**ABI compatibility issues:**
```bash
# Rebuild all components
cargo clean
cargo build --release
./build_plugins.sh
```

**Go project not recognized:**
```bash
# Ensure go.mod exists in project root
ls /path/to/project/go.mod

# Check Go installation
go version
```

### Debug Mode

Enable verbose logging:
```bash
RUST_LOG=debug ./target/release/skan-uj-kod cfg --project-path ./test-project
```

### Getting Help

1. **Check the documentation** in `docs/`
2. **Search existing issues** on GitHub
3. **Run the test suite** to isolate problems
4. **Create a minimal reproduction** case
5. **Open an issue** with detailed information

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👥 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and the `abi_stable` crate
- Inspired by static analysis tools like Frama-C and clang-static-analyzer
- Go AST parsing based on the Go compiler's internal structures

---

**Note**: This project was developed as part of an academic initiative for advanced static analysis techniques. For production use, ensure thorough testing with your specific Go codebases.