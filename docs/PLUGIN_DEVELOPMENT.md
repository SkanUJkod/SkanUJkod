# SkanUJkod Plugin Development Guide

## Introduction

This guide provides comprehensive instructions for developing plugins for the SkanUJkod static analysis framework. SkanUJkod's plugin architecture allows you to extend the framework with custom analysis capabilities while maintaining type safety and ABI stability.

## Prerequisites

Before developing plugins, ensure you have:
- **Rust** (1.70.0 or later)
- **SkanUJkod development environment** set up
- **Understanding of Rust traits and FFI**
- **Basic knowledge of Go language** (for analysis logic)

## Plugin Architecture Overview

### Plugin Lifecycle
1. **Discovery**: Plugin shared libraries discovered in `PLUGINS_DIR`
2. **Loading**: Dynamic loading via `abi_stable` library system
3. **Registration**: Plugin functions registered with the kernel
4. **Dependency Resolution**: Automatic topological sorting
5. **Execution**: Functions called in dependency order
6. **Result Sharing**: Outputs passed to dependent plugins

### Plugin Components
- **Plugin Library**: Shared library (.dylib/.so/.dll)
- **Plugin Functions**: Individual analysis operations
- **Dependencies**: Required inputs from other plugins
- **Parameters**: User-configurable options
- **Results**: Analysis outputs

## Creating Your First Plugin

### Step 1: Create Plugin Crate

```bash
# Navigate to plugins directory
cd crates/plugins

# Create new plugin crate
cargo new --lib my_analysis_plugin
cd my_analysis_plugin
```

### Step 2: Configure Cargo.toml

```toml
[package]
name = "my_analysis_plugin"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_analysis_plugin"
crate-type = ["lib", "cdylib"]  # Required for dynamic loading

[dependencies.abi_stable]
git = "https://github.com/rodrimati1992/abi_stable_crates.git"

[dependencies.plugin_interface]
path = "../../plugin_interface"
features = ["impls"]

[dependencies.go-parser]
path = "../../go-parser"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

### Step 3: Implement Plugin Entry Point

Create `src/lib.rs`:

```rust
use abi_stable::{
    export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn,
    std_types::{RString, RVec}, rvec, DynTrait,
};
use plugin_interface::{
    BoxedPFResult, PFDependencies, UserParameters, PluginFunction, Plugin, PluginRef,
    PFConnector, PFType, QualPFID,
};
use std::fmt;

// Plugin metadata
const PLUGIN_NAME: &str = "my_analysis_plugin";

// Export the plugin root module
#[export_root_module]
pub fn get_library() -> PluginRef {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

// Define plugin functions
#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![
        // Define your plugin functions here
        PFConnector {
            pf: PluginFunction(my_analysis_function),
            pf_type: PFType {
                pf_dependencies: rvec![
                    // Declare dependencies on other plugins
                    QualPFID {
                        plugin_id: "cfg_plugin".into(),
                        pf_id: "build_cfg".into(),
                    }
                ],
                user_params: rvec![
                    // Declare required user parameters
                    "project_path".into(),
                    "analysis_depth".into(),
                ],
            },
            pf_id: QualPFID {
                plugin_id: PLUGIN_NAME.into(),
                pf_id: "analyze".into(),
            },
        }
    ]
}

// Result type for your analysis
#[derive(Debug, Clone)]
pub struct MyAnalysisResult {
    pub analysis_data: String,
    pub score: f64,
    pub recommendations: Vec<String>,
}

impl fmt::Display for MyAnalysisResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Analysis Score: {}, Recommendations: {}",
               self.score, self.recommendations.len())
    }
}

// Main analysis function
#[sabi_extern_fn]
fn my_analysis_function(
    pf_results: PFDependencies,
    user_params: &UserParameters
) -> BoxedPFResult<'static> {
    // Get dependencies
    let cfg_result_id = QualPFID {
        plugin_id: "cfg_plugin".into(),
        pf_id: "build_cfg".into(),
    };

    let cfg_result = pf_results.get(&cfg_result_id)
        .expect("Missing CFG dependency");

    // Get user parameters
    let project_path = user_params.get(&RString::from("project_path"))
        .expect("Missing project_path parameter");

    // Perform your analysis
    let result = MyAnalysisResult {
        analysis_data: "Sample analysis data".to_string(),
        score: 85.5,
        recommendations: vec![
            "Consider refactoring complex functions".to_string(),
            "Add more unit tests".to_string(),
        ],
    };

    // Return boxed result
    DynTrait::from_value(result).into()
}
```

## Plugin Interface Details

### Function Signatures

All plugin functions must have this exact signature:
```rust
fn plugin_function(
    pf_results: PFDependencies,
    user_params: &UserParameters
) -> BoxedPFResult<'static>
```

### Dependencies (`PFDependencies`)

Access results from other plugins:
```rust
// Get specific dependency result
let dependency_id = QualPFID {
    plugin_id: "source_plugin".into(),
    pf_id: "function_name".into(),
};

let dependency_result = pf_results.get(&dependency_id)
    .ok_or("Missing required dependency")?;

// Downcast to specific type (unsafe but necessary)
let typed_result = unsafe {
    dependency_result.unchecked_downcast_as::<SpecificResultType>()
};
```

### User Parameters (`UserParameters`)

Access user-provided configuration:
```rust
// Get string parameter
let project_path = user_params.get(&RString::from("project_path"))
    .and_then(|param| param.downcast_as::<RString>())
    .map(|s| s.as_str())
    .ok_or("Missing project_path parameter")?;

// Get boolean parameter
let include_tests = user_params.get(&RString::from("include_tests"))
    .and_then(|param| param.downcast_as::<bool>())
    .copied()
    .unwrap_or(false);

// Get numeric parameter
let threshold = user_params.get(&RString::from("threshold"))
    .and_then(|param| param.downcast_as::<f64>())
    .copied()
    .unwrap_or(0.8);
```

### Return Values (`BoxedPFResult`)

Return analysis results:
```rust
// For simple types
let result = "Analysis complete".to_string();
return DynTrait::from_value(result).into();

// For complex types
#[derive(Debug, Clone)]
struct AnalysisResult {
    // Your fields here
}

impl std::fmt::Display for AnalysisResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Custom display format")
    }
}

let result = AnalysisResult { /* ... */ };
DynTrait::from_value(result).into()
```

## Advanced Plugin Patterns

### Multi-Function Plugins

A single plugin can provide multiple functions:

```rust
#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![
        // Function 1: Data collection
        PFConnector {
            pf: PluginFunction(collect_data),
            pf_type: PFType {
                pf_dependencies: rvec![],
                user_params: rvec!["project_path".into()],
            },
            pf_id: QualPFID {
                plugin_id: PLUGIN_NAME.into(),
                pf_id: "collect_data".into(),
            },
        },
        // Function 2: Analysis (depends on Function 1)
        PFConnector {
            pf: PluginFunction(analyze_data),
            pf_type: PFType {
                pf_dependencies: rvec![
                    QualPFID {
                        plugin_id: PLUGIN_NAME.into(),
                        pf_id: "collect_data".into(),
                    }
                ],
                user_params: rvec!["analysis_mode".into()],
            },
            pf_id: QualPFID {
                plugin_id: PLUGIN_NAME.into(),
                pf_id: "analyze_data".into(),
            },
        },
        // Function 3: Report generation
        PFConnector {
            pf: PluginFunction(generate_report),
            pf_type: PFType {
                pf_dependencies: rvec![
                    QualPFID {
                        plugin_id: PLUGIN_NAME.into(),
                        pf_id: "analyze_data".into(),
                    }
                ],
                user_params: rvec!["output_format".into()],
            },
            pf_id: QualPFID {
                plugin_id: PLUGIN_NAME.into(),
                pf_id: "generate_report".into(),
            },
        }
    ]
}
```

### Error Handling

Robust error handling in plugins:

```rust
use anyhow::{Context, Result};

#[sabi_extern_fn]
fn my_analysis_function(
    pf_results: PFDependencies,
    user_params: &UserParameters
) -> BoxedPFResult<'static> {
    // Use Result for internal error handling
    let result = perform_analysis(pf_results, user_params);

    match result {
        Ok(analysis_result) => DynTrait::from_value(analysis_result).into(),
        Err(e) => {
            eprintln!("Analysis failed: {}", e);
            // Return error result or default value
            let error_result = format!("Error: {}", e);
            DynTrait::from_value(error_result).into()
        }
    }
}

fn perform_analysis(
    pf_results: PFDependencies,
    user_params: &UserParameters
) -> Result<MyAnalysisResult> {
    // Get dependencies with proper error handling
    let cfg_result_id = QualPFID {
        plugin_id: "cfg_plugin".into(),
        pf_id: "build_cfg".into(),
    };

    let cfg_result = pf_results.get(&cfg_result_id)
        .context("Missing CFG dependency - ensure cfg_plugin is available")?;

    // Get parameters with validation
    let project_path = user_params.get(&RString::from("project_path"))
        .and_then(|param| param.downcast_as::<RString>())
        .map(|s| s.as_str())
        .context("Missing or invalid project_path parameter")?;

    // Validate inputs
    if !std::path::Path::new(project_path).exists() {
        anyhow::bail!("Project path does not exist: {}", project_path);
    }

    // Perform analysis...

    Ok(MyAnalysisResult {
        // ...
    })
}
```

### Working with Go AST

Access parsed Go code through dependencies:

```rust
use go_parser::ast::{File, FuncDecl, Stmt};

fn analyze_go_code(parsed_project: &ParsedProjectResult) -> Result<Vec<String>> {
    let mut function_names = Vec::new();

    for parsed_file in &parsed_project.files {
        for decl in &parsed_file.ast.decls {
            if let Decl::Func(func_decl) = decl {
                let func_name = parsed_project.ast_objects
                    .idents[func_decl.name]
                    .name
                    .clone();

                function_names.push(func_name);

                // Analyze function body
                if let Some(body) = &func_decl.body {
                    analyze_statements(&body.list)?;
                }
            }
        }
    }

    Ok(function_names)
}

fn analyze_statements(stmts: &[Stmt]) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::If(if_stmt) => {
                // Analyze if statement
                println!("Found if statement");
            }
            Stmt::For(for_stmt) => {
                // Analyze for loop
                println!("Found for loop");
            }
            Stmt::Block(block_stmt) => {
                // Recursively analyze block
                analyze_statements(&block_stmt.list)?;
            }
            _ => {
                // Handle other statement types
            }
        }
    }
    Ok(())
}
```

## Testing Your Plugin

### Unit Tests

Create `src/lib.rs` test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use plugin_interface::{UserParameters, PFDependencies};
    use abi_stable::std_types::{RHashMap, RString};

    #[test]
    fn test_analysis_function() {
        // Setup test parameters
        let mut user_params = RHashMap::new();
        user_params.insert(
            RString::from("project_path"),
            DynTrait::from_value(RString::from("/test/path"))
        );

        // Setup mock dependencies
        let pf_results = RHashMap::new();

        // Call plugin function
        let result = my_analysis_function(pf_results, &user_params);

        // Verify result
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Create `tests/integration_test.rs`:

```rust
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_plugin_integration() {
    // Create test Go project
    let temp_dir = TempDir::new().unwrap();
    let go_file = temp_dir.path().join("main.go");

    std::fs::write(&go_file, r#"
package main

func main() {
    println("Hello, world!")
}
"#).unwrap();

    // Test plugin through CLI
    let output = Command::new("../../target/release/skan-uj-kod")
        .args(&["my-analysis", "--project-path", temp_dir.path().to_str().unwrap()])
        .env("PLUGINS_DIR", "../../target/release")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}
```

### Building and Testing

```bash
# Build plugin
cargo build --release

# Run unit tests
cargo test

# Copy to plugin directory
cp target/release/libmy_analysis_plugin.dylib ../../../target/release/

# Test with CLI
cd ../../..
export PLUGINS_DIR=./target/release
./target/release/skan-uj-kod list  # Should show your plugin
```

## Integrating with CLI

### Add Command Support

Add your plugin to the CLI by editing `src/cli.rs`:

```rust
pub enum Commands {
    // ... existing commands ...

    /// Run my custom analysis
    MyAnalysis {
        /// Path to Go project directory
        #[arg(short, long)]
        project_path: PathBuf,

        /// Analysis depth
        #[arg(short, long, default_value = "5")]
        depth: u32,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
```

Add command handler in `src/main.rs`:

```rust
Commands::MyAnalysis { project_path, depth, output } => {
    commands::my_analysis::run_my_analysis(
        &mut plugin_manager,
        project_path,
        *depth,
        output.as_deref(),
    )
}
```

Create command implementation in `src/commands/my_analysis.rs`:

```rust
use crate::plugin_manager::PluginManager;
use plugin_interface::QualPFID;
use std::path::Path;

pub fn run_my_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    depth: u32,
    output_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Prepare parameters
    let mut user_params = plugin_manager.create_user_params();
    plugin_manager.add_string_param(&mut user_params, "project_path",
                                   project_path.to_string_lossy().as_ref());
    plugin_manager.add_string_param(&mut user_params, "analysis_depth",
                                   &depth.to_string());

    // Execute plugin
    let target_function = QualPFID {
        plugin_id: "my_analysis_plugin".into(),
        pf_id: "analyze".into(),
    };

    let result = plugin_manager.execute_plugin_chain(&target_function, user_params)?;

    // Handle output
    match output_path {
        Some(path) => {
            std::fs::write(path, format!("{}", result))?;
            println!("Results written to: {}", path.display());
        }
        None => {
            println!("Analysis Results:\n{}", result);
        }
    }

    Ok(())
}
```

## Best Practices

### 1. Error Handling
- Always handle errors gracefully
- Provide meaningful error messages
- Use `anyhow` for error context
- Don't panic in plugin code

### 2. Performance
- Minimize memory allocations
- Cache expensive computations
- Use efficient algorithms
- Profile your plugin with real projects

### 3. ABI Compatibility
- Only use ABI-stable types in public interfaces
- Test with different Rust versions
- Avoid complex generic types
- Use `repr(C)` for shared structures

### 4. Documentation
- Document all public functions
- Provide usage examples
- Document parameter requirements
- Include performance characteristics

### 5. Testing
- Test with various Go projects
- Include edge case tests
- Test error conditions
- Verify ABI compatibility

## Common Pitfalls

### 1. ABI Stability Issues
```rust
// ❌ Don't use standard Rust types in plugin interfaces
fn bad_function() -> Vec<String> { ... }

// ✅ Use ABI-stable types
fn good_function() -> RVec<RString> { ... }
```

### 2. Parameter Access
```rust
// ❌ Don't unwrap parameters
let param = user_params.get(&key).unwrap();

// ✅ Handle missing parameters gracefully
let param = user_params.get(&key)
    .ok_or("Missing required parameter")?;
```

### 3. Dependency Loops
```rust
// ❌ Don't create dependency cycles
// Plugin A depends on Plugin B, Plugin B depends on Plugin A

// ✅ Design clear dependency hierarchies
// Base plugins → Analysis plugins → Report plugins
```

### 4. Memory Management
```rust
// ❌ Don't share mutable references across ABI
fn bad_function() -> &mut SomeData { ... }

// ✅ Return owned data or immutable references
fn good_function() -> SomeData { ... }
```

## Plugin Examples

Check the existing plugins for reference implementations:
- `cfg_plugin` - Complex AST processing and graph generation
- `branch_cov_plugin` - Analysis with mock data for testing
- `statement_cov_plugin` - Simple analysis pattern
- `cyclomatic_complexity_plugin` - Metrics calculation

## Getting Help

1. **Review existing plugins** for patterns and examples
2. **Check the plugin interface documentation** in `crates/plugin_interface/`
3. **Run the test suite** to understand expected behavior
4. **Use debug builds** for better error messages
5. **Open issues** for specific problems or questions

---

This guide should provide everything you need to develop effective plugins for SkanUJkod. Remember to test thoroughly and follow Rust best practices for safe, efficient code.
