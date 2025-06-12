# SkanUJkod Contributing Guide

## Welcome Contributors! 🎉

Thank you for your interest in contributing to SkanUJkod! This guide will help you get started with contributing to this modular static analysis framework for Go projects.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Workflow](#contributing-workflow)
- [Code Standards](#code-standards)
- [Testing Guidelines](#testing-guidelines)
- [Plugin Development](#plugin-development)
- [Documentation](#documentation)
- [Issue Reporting](#issue-reporting)
- [Pull Request Process](#pull-request-process)
- [Community Guidelines](#community-guidelines)

## Getting Started

### Prerequisites

Ensure you have the following installed:

- **Rust** (1.70.0 or later) - [Install Rust](https://rustup.rs/)
- **Go** (1.19 or later) - [Install Go](https://golang.org/doc/install)
- **Git** - For version control
- **GitHub account** - For contributions

### Project Structure

```
SkanUJkod/
├── src/                     # Core application
├── crates/                  # Workspace crates
│   ├── go-parser/          # Go AST parsing
│   ├── go-types/           # Go type system
│   ├── kernel/             # Core kernel
│   ├── plugin_interface/   # Plugin ABI interface
│   └── plugins/            # Analysis plugins
├── tests/                   # Integration tests
├── docs/                   # Documentation
└── resources/              # Project resources
```

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/SkanUJkod.git
cd SkanUJkod

# Add upstream remote
git remote add upstream https://github.com/original-org/SkanUJkod.git
```

### 2. Environment Setup

```bash
# Set up environment variables
export PLUGINS_DIR=./target/debug
export RUST_LOG=debug  # For verbose logging

# Add to your shell profile
echo 'export PLUGINS_DIR=./target/debug' >> ~/.bashrc
```

### 3. Build and Test

```bash
# Build the project
cargo build

# Build plugins
./build_plugins.sh

# Run tests
cargo test

# Run integration tests
./test_all_functionality.sh
```

### 4. Verify Installation

```bash
# Test CLI
./target/debug/skan-uj-kod --help

# List available plugins
./target/debug/skan-uj-kod list
```

### 5. Quick Test Drive

Try these commands to see SkanUJkod in action:

```bash
# Create a simple Go test project
mkdir -p test-project
cat > test-project/main.go << 'EOF'
package main

import "fmt"

func main() {
    x := 10
    if x > 5 {
        fmt.Println("x is greater than 5")
        if x > 8 {
            fmt.Println("x is also greater than 8")
        }
    } else {
        fmt.Println("x is not greater than 5")
    }

    for i := 0; i < 3; i++ {
        fmt.Printf("Iteration %d\n", i)
    }
}
EOF

cat > test-project/go.mod << 'EOF'
module test-project

go 1.19
EOF

# Now test different analyses on this project:

# 1. Generate Control Flow Graph
./target/debug/skan-uj-kod cfg --project-path ./test-project --output cfg.dot
echo "CFG saved to cfg.dot - you can visualize it with Graphviz!"

# 2. Check Cyclomatic Complexity
./target/debug/skan-uj-kod complexity --project-path ./test-project --max-complexity 5
echo "This should show complexity analysis for the main function"

# 3. Run Branch Coverage Analysis
./target/debug/skan-uj-kod branch-cov --project-path ./test-project --threshold 0.8

# 4. Run Statement Coverage Analysis
./target/debug/skan-uj-kod statement-cov --project-path ./test-project --threshold 0.9

# 5. Run all analyses at once
mkdir -p analysis-results
./target/debug/skan-uj-kod all --project-path ./test-project --output-dir ./analysis-results
echo "All results saved to analysis-results/ directory"

# 6. View the CFG with Graphviz (if installed)
# dot -Tpng cfg.dot -o cfg.png && open cfg.png

# Clean up test project
# rm -rf test-project analysis-results cfg.dot
```

### 6. Test with Real Go Project

```bash
# Clone a real Go project to test with
git clone https://github.com/gin-gonic/gin.git test-gin
cd test-gin

# Run SkanUJkod on it
../target/debug/skan-uj-kod complexity --project-path . --max-complexity 10 --exclude "*_test.go"

# Generate CFG for specific package
../target/debug/skan-uj-kod cfg --project-path . --output gin-cfg.dot --exclude "*_test.go"

cd ..
# rm -rf test-gin  # Clean up when done
```

## Contributing Workflow

### Branch Strategy

We use a feature-branch workflow:

```bash
# Create feature branch
git checkout -b feature/amazing-new-feature

# Make changes and commit
git add .
git commit -m "feat: add amazing new feature"

# Push to your fork
git push origin feature/amazing-new-feature

# Create Pull Request on GitHub
```

### Branch Naming Convention

Branches should follow this pattern: `type/short-description`

| **Type**   | **Description**                             |
| ---------- | ------------------------------------------- |
| `feature`  | New functionality                           |
| `bug`      | Bug fixes                                   |
| `chore`    | Maintenance tasks (e.g., dependency bumps)  |
| `test`     | Adding or updating tests                    |
| `docs`     | Documentation updates                       |
| `refactor` | Code refactoring without functional changes |

**Examples:**

```bash
feature/add-verbose-flag
bug/fix-parse-error
chore/bump-dependencies
docs/update-readme
test/add-coverage-tests
refactor/simplify-instrumentation
```

### Commit Message Format

We use this **literal template** for commit messages:

```
TYPE(SCOPE): description (#issue)
```

**Types:** `FEAT`, `BUG`, `CHORE`, `TEST`, `DOCS`, `REFACTOR`
**Scope:** Module or area, e.g., `CLI`, `coverage`, `parser`

**Examples:**

```
FEAT(CLI): add --verbose flag for detailed output (#1)
BUG(Parser): prevent panic on empty input (#2)
DOCS(README): update installation instructions (#3)
REFACTOR(AST): extract instrumentation into helper module (#4)
TEST(Coverage): add tests for statement coverage function (#5)
CHORE(Deps): bump go_parser to v0.1.6 (#6)
```

## Code Standards

### Rust Code Style

We enforce strict code quality standards:

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings -D clippy::pedantic -D clippy::nursery

# Run all checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Code Quality Guidelines

1. **Follow Rust conventions**:

   - Use `snake_case` for functions and variables
   - Use `PascalCase` for types and traits
   - Use `SCREAMING_SNAKE_CASE` for constants

2. **Documentation**:

   - Document all public APIs with rustdoc
   - Include examples in documentation
   - Keep comments up-to-date

3. **Error Handling**:

   - Use `Result` types for fallible operations
   - Provide meaningful error messages
   - Use `anyhow` for error context

4. **Performance**:
   - Avoid unnecessary allocations
   - Use efficient algorithms
   - Profile performance-critical code

### Plugin Development Standards

When developing plugins:

1. **ABI Stability**:

   - Only use ABI-stable types in public interfaces
   - Test with different Rust versions
   - Follow plugin interface patterns

2. **Error Handling**:

   - Never panic in plugin code
   - Handle all error cases gracefully
   - Provide clear error messages

3. **Testing**:
   - Include comprehensive unit tests
   - Test with real Go projects
   - Test error conditions

### Quick Plugin Development Example

Here's how to create a simple plugin quickly:

```bash
# 1. Create new plugin
cd crates/plugins
cargo new --lib my_analysis_plugin
cd my_analysis_plugin

# 2. Set up Cargo.toml (copy structure from existing plugins)
cp ../cfg_plugin/Cargo.toml .
sed -i '' 's/cfg_plugin/my_analysis_plugin/g' Cargo.toml

# 3. Implement basic plugin structure
cat > src/lib.rs << 'EOF'
use abi_stable::{export_root_module, sabi_extern_fn, rvec};
use plugin_interface::*;

#[export_root_module]
pub fn get_library() -> PluginRef {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![
        PFConnector {
            pf: PluginFunction(my_analysis),
            pf_type: PFType {
                pf_dependencies: rvec![],
                user_params: rvec!["project_path".into()]
            },
            pf_id: QualPFID {
                plugin_id: "my_analysis_plugin".into(),
                pf_id: "my_analysis".into()
            }
        }
    ]
}

#[sabi_extern_fn]
fn my_analysis(
    _pf_results: PFDependencies,
    user_params: &UserParameters
) -> BoxedPFResult<'static> {
    // Your analysis implementation here
    println!("Running my analysis!");
    let result = "Analysis complete".to_string();
    Box::new(result).into()
}
EOF

# 4. Build the plugin
cargo build
cd ../../..

# 5. Test the plugin
cargo build
./target/debug/skan-uj-kod list  # Should show your plugin

# 6. Add CLI command (optional)
# Edit src/commands/mod.rs and src/cli.rs to add your command
```

## Testing Guidelines

### Test Categories

1. **Unit Tests** - Test individual components
2. **Integration Tests** - Test component interactions
3. **Plugin Tests** - Test plugin functionality
4. **CLI Tests** - Test command-line interface

### Running Tests

```bash
# Run all tests
cargo test

# Run specific package tests
cargo test --package cfg_plugin

# Run integration tests
./test_all_functionality.sh

# Run with verbose output
cargo test -- --nocapture
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_behavior() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result.expected_field, expected_value);
        assert!(result.is_valid());
    }

    #[test]
    fn test_error_handling() {
        let invalid_input = create_invalid_input();
        let result = function_under_test(invalid_input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Expected error message");
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_integration() {
    let temp_dir = TempDir::new().unwrap();
    create_test_go_project(&temp_dir);

    let output = Command::new("./target/debug/skan-uj-kod")
        .args(&["cfg", "--project-path", temp_dir.path().to_str().unwrap()])
        .env("PLUGINS_DIR", "./target/debug")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("digraph"));
}
```

### Test Coverage

We aim for high test coverage:

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

## Plugin Development

### Plugin Contribution Process

1. **Plan the Plugin**:

   - Identify the analysis need
   - Design the plugin interface
   - Consider dependencies and outputs

2. **Implement the Plugin**:

   - Follow [Plugin Development Guide](PLUGIN_DEVELOPMENT.md)
   - Implement core analysis logic
   - Add comprehensive tests

3. **Integration**:
   - Add CLI command support
   - Update documentation
   - Test with real projects

### Plugin Review Criteria

- **Functionality**: Does it provide valuable analysis?
- **Code Quality**: Follows project standards?
- **Testing**: Comprehensive test coverage?
- **Documentation**: Well documented with examples?
- **Performance**: Efficient implementation?
- **ABI Compatibility**: Follows plugin interface guidelines?

## Documentation

### Documentation Types

1. **Code Documentation** - Rustdoc comments
2. **User Documentation** - README, usage guides
3. **Developer Documentation** - Architecture, contributing guides
4. **Plugin Documentation** - Plugin development guides

### Writing Documentation

#### Rustdoc

````rust
/// Analyzes the control flow graph for cyclomatic complexity.
///
/// # Arguments
///
/// * `cfg` - The control flow graph to analyze
/// * `options` - Analysis configuration options
///
/// # Returns
///
/// Returns a `ComplexityResult` containing the analysis results.
///
/// # Examples
///
/// ```
/// let cfg = build_cfg_from_function(&func);
/// let options = ComplexityOptions::default();
/// let result = analyze_complexity(&cfg, &options)?;
/// println!("Complexity: {}", result.cyclomatic_complexity);
/// ```
///
/// # Errors
///
/// Returns an error if the CFG is malformed or analysis fails.
pub fn analyze_complexity(
    cfg: &ControlFlowGraph,
    options: &ComplexityOptions,
) -> Result<ComplexityResult> {
    // Implementation
}
````

#### User Documentation

- Clear, step-by-step instructions
- Real-world examples
- Troubleshooting guides
- Screenshots where helpful

### Documentation Standards

- Keep documentation up-to-date with code changes
- Use clear, concise language
- Provide examples for complex concepts
- Include troubleshooting information

## Issue Reporting

### Before Creating an Issue

1. **Search existing issues** - Check if it's already reported
2. **Check documentation** - Might be a usage question
3. **Test with latest version** - Issue might be fixed
4. **Create minimal reproduction** - Simplify the problem

### Issue Types

#### Bug Reports

Use the bug report template:

```markdown
**Describe the bug**
A clear description of the bug.

**To Reproduce**
Steps to reproduce the behavior:

1. Run command '...'
2. With input '...'
3. See error

**Expected behavior**
What you expected to happen.

**Environment:**

- OS: [e.g., macOS 12.0]
- Rust version: [e.g., 1.70.0]
- SkanUJkod version: [e.g., 0.1.0]

**Additional context**
Any other context about the problem.
```

#### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
A clear description of the problem.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Other solutions you've considered.

**Additional context**
Any other context about the feature request.
```

### Issue Labels

- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Documentation improvements
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention needed
- `plugin` - Plugin-related issues
- `performance` - Performance improvements

## Pull Request Process

### Pre-submission Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review of the changes
- [ ] Comments added for hard-to-understand areas
- [ ] Documentation updated if needed
- [ ] Tests added for new functionality
- [ ] All tests pass locally
- [ ] No breaking changes (or properly documented)

### Pull Request Template

```markdown
## Description

Brief description of changes and their purpose.

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass

## Additional Notes

Any additional information about the changes.
```

### Review Process

1. **Automated Checks**: CI runs tests and linting
2. **Code Review**: Maintainers review the code
3. **Feedback**: Address review comments
4. **Approval**: Get approval from maintainers
5. **Merge**: Maintainer merges the PR

### Review Criteria

- **Correctness**: Does the code work as intended?
- **Code Quality**: Follows project standards?
- **Testing**: Adequate test coverage?
- **Documentation**: Properly documented?
- **Performance**: No performance regressions?
- **Security**: No security vulnerabilities?

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment:

- **Be respectful** - Treat all community members with respect
- **Be inclusive** - Welcome people of all backgrounds and identities
- **Be collaborative** - Work together constructively
- **Be patient** - Help newcomers learn and grow
- **Be professional** - Maintain professional communication

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General questions and discussions
- **Pull Requests** - Code contributions and reviews

### Getting Help

1. **Check documentation** - README, guides, and API docs
2. **Search issues** - Someone might have had the same problem
3. **Ask questions** - Use GitHub Discussions for questions
4. **Join the community** - Participate in discussions and reviews

## Recognition

We appreciate all contributions! Contributors will be:

- Listed in the project's contributor list
- Recognized in release notes for significant contributions
- Invited to join the maintainer team for sustained contributions

## Common Development Scenarios & Commands

### Debugging Build Issues

```bash
# Clean and rebuild everything
cargo clean
cargo build --release
./build_plugins.sh

# Check for specific plugin issues
cargo build --package cfg_plugin --verbose

# Debug plugin loading
RUST_LOG=debug ./target/debug/skan-uj-kod list
```

### Testing Changes

```bash
# Run specific tests while developing
cargo test --package cfg_plugin -- --nocapture

# Test CLI changes
./target/debug/skan-uj-kod cfg --project-path ./test-project --output /tmp/test.dot

# Run comprehensive tests
./test_all_functionality.sh

# Performance testing with larger project
time ./target/release/skan-uj-kod all --project-path /path/to/large/go/project --output-dir /tmp/results
```

### Working with Git

```bash
# Keep your fork updated
git fetch upstream
git checkout main
git merge upstream/main
git push origin main

# Rebase feature branch
git checkout feature/my-feature
git rebase main

# Squash commits before PR
git rebase -i HEAD~3  # Interactive rebase for last 3 commits
```

### Code Quality Checks

```bash
# Format code
cargo fmt

# Comprehensive linting
cargo clippy -- -D warnings -D clippy::pedantic

# Check documentation
cargo doc --no-deps --document-private-items

# Security audit
cargo audit

# Check for unused dependencies
cargo machete
```

### Plugin Testing Workflow

```bash
# Test plugin in isolation
cd crates/plugins/my_plugin
cargo test

# Test plugin integration
cd ../../..
cargo build
export PLUGINS_DIR=./target/debug
./target/debug/skan-uj-kod list | grep my_plugin

# Test with real Go code
echo 'package main; func main() { println("test") }' > /tmp/test.go
echo 'module test\ngo 1.19' > /tmp/go.mod
./target/debug/skan-uj-kod my-command --project-path /tmp
```

### Troubleshooting Common Issues

```bash
# Plugin not found
ls -la $PLUGINS_DIR/*.dylib
export PLUGINS_DIR=./target/debug  # Make sure this is set

# ABI compatibility issues
cargo clean && cargo build --release

# Go parsing errors
./target/debug/skan-uj-kod cfg --project-path ./problematic-project 2>&1 | head -20

# Performance issues
time ./target/release/skan-uj-kod cfg --project-path ./large-project
RUST_LOG=info ./target/release/skan-uj-kod cfg --project-path ./large-project
```

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH`
- Major: Breaking changes
- Minor: New features (backward compatible)
- Patch: Bug fixes (backward compatible)

### Release Schedule

- **Patch releases**: As needed for bug fixes
- **Minor releases**: Monthly for new features
- **Major releases**: When breaking changes are needed

Thank you for contributing to SkanUJkod! Your contributions help make static analysis more accessible and powerful for the Go community. 🚀
