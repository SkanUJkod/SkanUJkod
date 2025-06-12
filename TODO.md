# Project TODO List

## 🚨 Critical Issues / High Priority

### Architecture & Core
- [x] **Fix ABI parameter passing** - COMPLETED ✅
  - Fixed parameter passing between CLI and plugins
  - Issue was incorrect parameter access using string literals instead of RString::from()
  - Fixed in `crates/plugins/cfg_plugin/src/lib.rs` lines ~170 and ~240
  - CLI function filtering now works: `./skan-uj-kod cfg --project-path . --function main`
  
- [ ] **Debug plugin execution output** - IN PROGRESS 🔄
  - Function filtering logic works but debug output from plugin functions not visible
  - Need to investigate stderr/stdout handling in plugin context
  - Location: `crates/plugins/cfg_plugin/src/lib.rs` build_cfg_pf and export_dot_pf functions

### Output Formats
- [ ] **Implement JSON export for CFG plugin**
  - Currently only DOT format is fully implemented
  - Location: `crates/plugins/cfg_plugin/src/export.rs`

- [ ] **Implement text export for CFG plugin**
  - Add human-readable text format
  - Location: `crates/plugins/cfg_plugin/src/export.rs`

### Plugin Functionality
- [x] **Add CLI function filtering for CFG analysis** - COMPLETED ✅
  - Added `--function <name>` parameter to CFG command
  - Implements exact function name matching
  - CLI: `./skan-uj-kod cfg --project-path . --function main`
  - Location: `src/cli.rs`, `src/commands/cfg.rs`, `crates/plugins/cfg_plugin/src/lib.rs`
  
- [ ] **Improve CFG graph readability** - PENDING 🔄
  - Current CFG graphs have too many connections and unnecessary "Empty" blocks
  - Need to simplify graph structure and reduce complexity
  - Consider node merging and edge optimization
  - Location: `crates/plugins/cfg_plugin/src/cfg.rs` and `src/export.rs`

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
- [x] **CFG case sensitivity fix** - Function names with mixed case now properly handled in DOT output

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

## ✅ Recent Accomplishments

### Function Filtering Implementation (2025-06-09)
- **CLI Parameter Addition**: Added `--function <name>` option to CFG command
- **Parameter Passing Fix**: Fixed ABI parameter passing from CLI to plugins  
- **Plugin Chain Updates**: Updated build_cfg_pf and export_dot_pf to accept function_filter parameter
- **Exact Matching**: Implemented exact function name matching (not substring matching)
- **Cross-component Integration**: Updated src/cli.rs, src/main.rs, src/commands/cfg.rs, and plugin implementations

**Technical Details:**
- Fixed parameter access using `user_params.get(&RString::from("function_filter"))` instead of string literals
- Added function_filter to user_params list in plugin connector configuration
- Verified CLI shows "Filtering for function: <name>" and parameter reaches plugin chain
- Testing: `./target/release/skan-uj-kod cfg --project-path ../go-code/example-go --function main`

**Next Steps:**
- Investigate debug output visibility from plugin functions
- Address CFG graph complexity (too many Empty blocks and connections)
- Consider implementing partial function name matching as alternative to exact matching

---

**Note**: This TODO list should be updated as issues are resolved and new requirements are identified. Priority levels may change based on user feedback and project goals.
