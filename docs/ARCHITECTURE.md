# SkanUJkod Architecture Documentation

## Overview

SkanUJkod is a modular static analysis framework for Go projects built in Rust. The architecture is designed around a plugin-based system that allows for extensible analysis capabilities while maintaining type safety and ABI stability.

## Core Architecture Principles

### 1. Plugin-Based Modularity
- **Separation of Concerns**: Each analysis type is implemented as a separate plugin
- **Extensibility**: New analyses can be added without modifying core system
- **Dynamic Loading**: Plugins are loaded at runtime as shared libraries
- **Dependency Management**: Automatic resolution of plugin dependencies

### 2. ABI Stability
- **Cross-Version Compatibility**: Using `abi_stable` crate for stable interfaces
- **Type Safety**: Compile-time guarantees for plugin interfaces
- **Memory Safety**: Rust's ownership system prevents common C-style FFI issues

### 3. Functional Pipeline Design
- **Immutable Data Flow**: Data flows through plugins without mutation
- **Result Sharing**: Plugins can depend on outputs from other plugins
- **Topological Execution**: Automatic ordering based on dependencies

## System Components

### Core System (`src/`)

```
src/
├── main.rs              # Application entry point
├── cli.rs               # Command-line interface definition
├── plugin_manager.rs    # Plugin loading and execution
└── commands/            # Command implementations
    ├── cfg.rs
    ├── branch_coverage.rs
    ├── statement_coverage.rs
    ├── cyclomatic_complexity.rs
    └── all.rs
```

#### Plugin Manager (`src/plugin_manager.rs`)
The core orchestrator responsible for:
- **Plugin Discovery**: Scanning plugin directories for shared libraries
- **Dynamic Loading**: Loading plugins using `abi_stable::library`
- **Dependency Resolution**: Building execution graphs from plugin dependencies
- **Parameter Passing**: Managing user parameters and inter-plugin data flow
- **Execution Orchestration**: Running plugins in correct dependency order

```rust
pub struct PluginManager {
    connectors: Vec<PFConnector>,           // All available plugin functions
    pf_to_idx: HashMap<QualPFID, usize>,   // Function lookup index
    plugins_dir: PathBuf,                   // Plugin directory path
}
```

#### CLI Interface (`src/cli.rs`)
Command-line interface using `clap` with:
- **Subcommands**: Different analysis types (cfg, branch-cov, etc.)
- **Common Options**: Project path, output format, include/exclude patterns
- **Type Safety**: Compile-time validation of arguments
- **Help Generation**: Automatic help text generation

### Plugin Interface (`crates/plugin_interface/`)

The ABI-stable interface layer that defines how plugins communicate:

```rust
#[repr(C)]
#[derive(StableAbi)]
pub struct Plugin {
    pub funcs: extern "C" fn() -> RVec<PFConnector>,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct PFConnector {
    pub pf: PluginFunction,           // The actual function pointer
    pub pf_type: PFType,              // Metadata about the function
    pub pf_id: QualPFID,              // Unique identifier
}
```

#### Key Types
- **PluginFunction**: ABI-stable function pointer for plugin operations
- **PFDependencies**: Map of dependency results passed to plugins
- **UserParameters**: Map of user-provided parameters
- **QualPFID**: Qualified plugin function identifier (plugin_id::function_id)

### Go Parser (`crates/go-parser/`)

Go source code parsing and AST manipulation:
- **AST Representation**: Rust types mirroring Go's AST structure
- **File System Traversal**: Walking Go project directories
- **Parse Options**: Configurable parsing (include tests, exclude patterns)
- **Error Handling**: Graceful handling of parse errors

### Analysis Plugins (`crates/plugins/`)

Each plugin is a separate Rust crate compiled as a dynamic library:

#### Plugin Structure
```
crates/plugins/example_plugin/
├── Cargo.toml          # Plugin configuration
├── src/
│   ├── lib.rs          # Plugin entry point and exports
│   ├── core.rs         # Analysis implementation
│   └── helpers.rs      # Utility functions
└── tests/              # Plugin-specific tests
```

#### CFG Plugin (`crates/plugins/cfg_plugin/`)
Control Flow Graph generation:
- **AST to CFG**: Converts Go AST to graph representation
- **Multiple Builders**: Different CFG building strategies
- **DOT Export**: Graphviz visualization format
- **Block Analysis**: Basic block identification and connection

```rust
pub struct ControlFlowGraph {
    pub blocks: HashMap<usize, BasicBlock>,
    pub entry: usize,
}

pub struct BasicBlock {
    pub id: usize,
    pub stmts: Vec<Statement>,
    pub succs: Vec<usize>,
    pub start_line: usize,
    pub end_line: usize,
}
```

#### Coverage Plugins
- **Branch Coverage**: Identifies decision points and branch paths
- **Statement Coverage**: Tracks individual statement execution
- **Instrumentation**: Generates instrumented Go code for coverage collection

#### Complexity Plugin
- **Cyclomatic Complexity**: McCabe complexity calculation
- **Decision Points**: Identification of complexity-contributing constructs
- **Cognitive Complexity**: Alternative complexity metrics

## Data Flow Architecture

### 1. Input Processing
```
Go Source Files → File System Scanner → AST Parser → ParsedProject
```

### 2. Plugin Execution Pipeline
```
ParsedProject → Plugin Dependencies Resolution → Topological Sort → Plugin Execution Chain → Results
```

### 3. Output Generation
```
Plugin Results → Format Selection → Output Formatter → File/Console Output
```

## Plugin System Details

### Plugin Loading Process

1. **Discovery**: Scan `PLUGINS_DIR` for dynamic libraries (`.dylib`, `.so`, `.dll`)
2. **Loading**: Load each library using `abi_stable::library::lib_header_from_path`
3. **Registration**: Call plugin's `get_library()` function to get plugin descriptors
4. **Indexing**: Build lookup tables for function resolution
5. **Validation**: Verify plugin interface compatibility

### Dependency Resolution

Plugins declare dependencies on other plugin functions:

```rust
PFConnector {
    pf_id: QualPFID {
        plugin_id: "my_plugin".into(),
        pf_id: "my_function".into(),
    },
    pf_type: PFType {
        pf_dependencies: rvec![
            QualPFID {
                plugin_id: "cfg_plugin".into(),
                pf_id: "build_cfg".into(),
            }
        ],
        user_params: rvec!["project_path".into()],
    },
    // ...
}
```

The plugin manager:
1. **Builds dependency graph** from plugin declarations
2. **Performs topological sort** to determine execution order
3. **Detects cycles** and reports dependency errors
4. **Executes in order** with result passing

### Inter-Plugin Communication

Plugins communicate through:
- **Shared Results**: Output from one plugin becomes input to dependents
- **User Parameters**: Configuration passed from CLI to plugins
- **Type-Safe Interfaces**: All communication through ABI-stable types

## Memory Management

### ABI-Stable Types
All plugin interfaces use types from `abi_stable`:
- **RVec**: ABI-stable vector
- **RString**: ABI-stable string
- **RHashMap**: ABI-stable hash map
- **DynTrait**: Type-erased trait objects

### Ownership Model
- **Plugin Results**: Owned by plugin manager, borrowed by dependents
- **User Parameters**: Shared immutable references
- **No Memory Sharing**: Plugins don't share mutable memory

## Error Handling Strategy

### Plugin Errors
- **Result Types**: All plugin functions return `Result<T, E>`
- **Error Propagation**: Errors bubble up through the execution chain
- **Graceful Degradation**: Failed plugins don't crash the entire system
- **User Feedback**: Clear error messages with context

### ABI Error Handling
- **Stable Error Types**: Errors must cross ABI boundary safely
- **Error Boxing**: Complex errors are boxed for ABI stability
- **Fallback Behavior**: System continues when non-critical plugins fail

## Performance Considerations

### Plugin Loading
- **Lazy Loading**: Plugins loaded only when needed
- **Caching**: Plugin metadata cached for subsequent runs
- **Parallel Loading**: Multiple plugins can be loaded concurrently

### Execution Optimization
- **Dependency Pruning**: Only execute plugins needed for target function
- **Result Caching**: Plugin results cached within single execution
- **Memory Efficiency**: Large data structures moved, not copied

### AST Processing
- **Single Parse**: Go files parsed once and shared between plugins
- **Incremental Processing**: Support for processing subsets of files
- **Memory Mapping**: Large projects can use memory-mapped files

## Security Considerations

### Plugin Sandboxing
- **No Sandbox**: Currently plugins run with full system privileges
- **Code Review**: Plugins should be reviewed before deployment
- **Trusted Sources**: Only load plugins from trusted locations

### Memory Safety
- **Rust Guarantees**: Memory safety enforced by Rust compiler
- **ABI Boundaries**: Careful handling of data across plugin boundaries
- **Resource Limits**: Plugins should implement resource limits

## Extension Points

### Adding New Analyses
1. **Create Plugin Crate**: New crate in `crates/plugins/`
2. **Implement Interface**: Follow plugin interface patterns
3. **Declare Dependencies**: Specify required plugin inputs
4. **Add Command**: Integrate with CLI system
5. **Test Integration**: Ensure plugin works with existing system

### Custom Output Formats
1. **Extend Format Enum**: Add new format to `OutputFormat`
2. **Implement Formatter**: Create format-specific output logic
3. **Update Commands**: Integrate with command system

### Advanced Integrations
- **IDE Plugins**: Build language server protocol support
- **Web Interface**: Create web-based analysis dashboard
- **API Server**: Expose analysis as REST API
- **Distributed Analysis**: Scale across multiple machines

## Debugging and Monitoring

### Debug Build
- **Verbose Logging**: Enable with `RUST_LOG=debug`
- **Plugin Introspection**: List loaded plugins and capabilities
- **Dependency Visualization**: Show plugin dependency graph

### Production Monitoring
- **Performance Metrics**: Track analysis execution times
- **Error Reporting**: Structured error logging
- **Resource Usage**: Monitor memory and CPU usage

---

This architecture provides a solid foundation for extensible static analysis while maintaining type safety, performance, and reliability. The plugin system allows the framework to grow with new analysis capabilities without requiring changes to the core system.
