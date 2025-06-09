# Project TODO List

## 🚨 Critical Issues / High Priority

### Architecture & Core
- [ ] **Fix ABI parameter passing** - Currently hardcoded to avoid crashes
  - The plugin system works but parameters aren't properly passed between plugins
  - Need to investigate `UserParameters` serialization/deserialization
  - Location: `src/plugin_manager.rs` and plugin implementations

### Output Formats
- [ ] **Implement JSON export for CFG plugin**
  - Currently only DOT format is fully implemented
  - Location: `crates/plugins/cfg_plugin/src/export.rs`

- [ ] **Implement text export for CFG plugin**
  - Add human-readable text format
  - Location: `crates/plugins/cfg_plugin/src/export.rs`

### Plugin Functionality
- [ ] **Complete coverage analysis implementations**
  - Branch and statement coverage plugins currently return mock data
  - Need real Go code instrumentation and test execution
  - Locations: `crates/plugins/branch_cov_plugin/`, `crates/plugins/statement_cov_plugin/`

- [ ] **Enhance cyclomatic complexity calculation**
  - Currently simplified implementation
  - Need proper CFG-based complexity calculation
  - Location: `crates/plugins/cyclomatic_complexity_plugin/`

## 🔧 Medium Priority

### Testing & Quality
- [ ] **Add comprehensive integration tests**
  - Test all plugins with real Go projects
  - Add edge case testing
  - Location: `tests/`

- [ ] **Improve error handling and user messages**
  - Better error messages for common failures
  - Graceful degradation when plugins fail
  - Location: Throughout codebase

### Documentation
- [ ] **Add rustdoc documentation**
  - Document all public APIs
  - Add code examples
  - Generate and publish docs

- [ ] **Create plugin development tutorial**
  - Step-by-step guide for creating new plugins
  - Examples and best practices
  - Location: `docs/plugin-development.md`

### Performance
- [ ] **Optimize Go AST parsing**
  - Currently parses entire project for each analysis
  - Implement caching and incremental parsing
  - Location: `crates/go-parser/`

- [ ] **Parallel plugin execution**
  - Execute independent plugins in parallel
  - Location: `src/plugin_manager.rs`

## 🎯 Low Priority / Future Features

### Advanced Go Support
- [ ] **Support Go generics**
  - Update parser for Go 1.18+ features
  - Location: `crates/go-parser/`

- [ ] **Support Go modules and dependencies**
  - Analyze external dependencies
  - Cross-module analysis

### User Experience
- [ ] **Add configuration file support**
  - YAML/TOML configuration for common settings
  - Per-project configuration

- [ ] **Implement watch mode**
  - Re-run analysis on file changes
  - Incremental updates

### Visualization
- [ ] **Web-based dashboard**
  - Interactive visualization of results
  - HTML report generation

- [ ] **IDE integrations**
  - VS Code extension
  - GoLand/IntelliJ plugin

### Advanced Analysis
- [ ] **Data flow analysis**
  - Track variable usage and modification
  - Security vulnerability detection

- [ ] **Custom rule engine**
  - User-defined analysis rules
  - Rule DSL or configuration

## 🐛 Known Bugs

### Plugin System
- [x] **ABI stability crashes** - FIXED with hardcoded parameters workaround
- [ ] **Parameter passing not working** - Parameters aren't properly passed to plugins
- [ ] **Plugin dependencies can cause infinite loops** - Need better cycle detection

### Parser
- [ ] **Complex Go syntax edge cases** - Some advanced Go constructs not handled
- [ ] **Error recovery during parsing** - Parser fails on malformed code instead of continuing

### CLI
- [ ] **Inconsistent error codes** - Exit codes not standardized
- [ ] **Help text formatting** - Some help text is unclear or truncated

## 📋 Completed Items

### Core Architecture ✅
- [x] Plugin-based architecture implementation
- [x] Dynamic plugin loading
- [x] Basic dependency resolution
- [x] CLI interface with clap

### Basic Functionality ✅
- [x] CFG generation with DOT output
- [x] Basic branch coverage plugin structure
- [x] Basic statement coverage plugin structure
- [x] Cyclomatic complexity plugin structure
- [x] All analyses command

### Testing ✅
- [x] Basic unit tests for core components
- [x] Integration test script
- [x] CI/CD pipeline with GitHub Actions
- [x] ABI fix verification tests

## 🛠 Development Notes

### Architecture Decisions
- Using `abi_stable` for plugin ABI compatibility
- Rust workspace for modular development
- Dynamic library loading for plugins
- Topological sorting for plugin dependencies

### Key Files to Understand
- `src/plugin_manager.rs` - Core plugin loading and execution
- `crates/plugin_interface/src/lib.rs` - Plugin API definitions
- `src/cli.rs` - Command-line interface
- Plugin implementations in `crates/plugins/*/src/lib.rs`

### Dependencies
- `abi_stable` - Cross-version ABI compatibility
- `clap` - Command-line argument parsing
- `serde/serde_json` - Serialization
- `tempfile` - Temporary file handling for tests

### Testing Strategy
- Unit tests for individual components
- Integration tests with `test_all_functionality.sh`
- Plugin-specific tests in each plugin crate
- Real Go project testing for validation

---

**Note**: This TODO list should be updated as issues are resolved and new requirements are identified. Priority levels may change based on user feedback and project goals.
