# 🧠 CFG Library for Go

A Rust library for parsing Go source code, building control flow graphs (CFG), optimizing them, validating, and exporting to Graphviz DOT format for visualization.

---

## 📚 Table of Contents

- [Project Parsing (`parser.rs`)](#project-parsing-parserrs)
- [Control Flow Graph Construction (`builder.rs`)](#control-flow-graph-construction-builderrs)
- [Detailed Flow and Edge Cases](#detailed-flow-and-edge-cases)
- [CFG Optimization (`optimization.rs`)](#cfg-optimization-optimizationrs)
- [CFG Validation (`validation.rs`)](#cfg-validation-validationrs)
- [DOT Export (`dot.rs`)](#dot-export-dotrs)
- [Tutorial: Using the CFG Library](#tutorial-using-the-cfg-library)
- [Core Data Structures](#core-data-structures)

---

## 📦 Project Parsing (`parser.rs`)

This module is responsible for scanning a directory or file, and parsing all `.go` source files into Abstract Syntax Trees (ASTs).

### Function: `parse_project`

```rust
pub fn parse_project(root: &Path) -> Result<(FileSet, AstObjects, Vec<ParsedFile>)>
```

### Output

- `FileSet`: Source position info
- `AstObjects`: AST object registry
- `ParsedFile`: Each file's path and AST

---

## 🏗️ Control Flow Graph Construction (`builder.rs`)

Builds a Control Flow Graph (CFG) from a single Go function.

### Function: `ControlFlowGraph::build`

```rust
pub fn build(fset: &FileSet, func: &FuncDecl, objs: &AstObjects) -> Self
```

- Creates basic blocks
- Handles entry and exit paths
- Supports constructs like `if`, `for`, `switch`, and loops

---

## 🔍 Detailed Flow and Edge Cases

Describes how `build()` works in-depth and how edge cases are handled:

1. Initialization with entry and exit blocks
2. Handling empty function bodies
3. Statement traversal and block creation
4. Handling fallthroughs, early exits, loops
5. Structural integrity with macros and safety guards

---

## 🚀 CFG Optimization (`optimization.rs`)

Simplifies the CFG after initial construction.

### Functions

#### `clean_unreachable_blocks`

Removes dead/unreachable blocks.

#### `optimize_empty_blocks`

Removes trivial empty blocks not needed for control flow.

---

## ✅ CFG Validation (`validation.rs`)

Ensures the correctness of a CFG after transformation.

### Functions

#### `validate_cfg`

Checks for:

- Valid successors
- Entry existence
- Self-loops and malformed edges

#### `has_cycles`

Detects control flow cycles via DFS.

---

## 📊 DOT Export (`dot.rs`)

Exports a CFG to Graphviz DOT format.

### Function: `emit_dot`

```rust
pub fn emit_dot<W: Write>(...) -> std::io::Result<()>
```

- Outputs DOT format
- Blocks show IDs and line ranges
- Supports rendering with `dot` CLI

---

## 🧪 Tutorial: Using the CFG Library

### 1. Setup

```toml
go-parser = "..."
walkdir = "2.4"
anyhow = "1.0"
```

### 2. Parse

```rust
let (fset, objs, files) = parse_project(Path::new("my_project"))?;
```

### 3. Build CFG

```rust
let cfg = ControlFlowGraph::build(&fset, &func_decl, &objs);
```

### 4. Optimize

```rust
clean_unreachable_blocks(&mut cfg.blocks, cfg.entry);
optimize_empty_blocks(&mut cfg.blocks, cfg.entry, exit_id, &HashSet::new());
```

### 5. Validate

```rust
validate_cfg(&cfg.blocks, cfg.entry);
assert!(!has_cycles(&cfg.blocks, cfg.entry));
```

### 6. Export

```rust
let mut file = File::create("cfg.dot")?;
emit_dot(&mut file, &cfg.blocks, cfg.entry)?;
```

---

## 📐 Core Data Structures

### `BasicBlock`

```rust
pub struct BasicBlock {
    pub id: usize,
    pub stmts: Vec<Stmt>,
    pub succs: Vec<usize>,
    pub start_line: usize,
    pub end_line: usize,
}
```

### `ControlFlowGraph`

```rust
pub struct ControlFlowGraph {
    pub blocks: HashMap<usize, BasicBlock>,
    pub entry: usize,
}
```

# CFG Module - Control Flow Graph Builder for Go Code Analysis

## Overview

The CFG (Control Flow Graph) module is a comprehensive Rust library designed to parse Go source code and construct detailed control flow graphs. This module is part of a larger static analysis toolkit and provides essential functionality for code coverage analysis, complexity measurement, and program flow understanding.

## Architecture

The module is organized into four main submodules:

```
cfg/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── ast/                # Abstract Syntax Tree parsing
│   │   ├── mod.rs         # Module exports
│   │   └── parser.rs      # Go code parsing logic
│   ├── cfg/               # Control Flow Graph construction
│   │   ├── mod.rs         # Module exports
│   │   ├── types.rs       # Core data structures
│   │   ├── builder.rs     # CFG construction algorithm
│   │   ├── optimization.rs # Graph optimization passes
│   │   └── validation.rs  # Graph integrity validation
│   └── export/            # Output formatting
│       ├── mod.rs         # Module exports
│       └── dot.rs         # Graphviz DOT format export
```

## Core Data Structures

### Statement

```rust
#[derive(Debug, Clone)]
pub struct Statement {
    pub text: String,              // Human-readable representation
    pub stmt: go_parser::ast::Stmt, // Original AST statement
}
```

Represents a single Go statement with both its textual representation and the original AST node for detailed analysis.

### BasicBlock

```rust
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,           // Unique block identifier
    pub stmts: Vec<Statement>, // Statements in this block
    pub succs: Vec<usize>,   // Successor block IDs
    pub start_line: usize,   // Starting line number
    pub end_line: usize,     // Ending line number
}
```

A basic block is a maximal sequence of statements with:

- Single entry point (first statement)
- Single exit point (last statement)
- No branching within the block
- Linear execution flow

### ControlFlowGraph

```rust
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub blocks: HashMap<usize, BasicBlock>, // All blocks in the graph
    pub entry: usize,                       // Entry block ID
}
```

The complete control flow graph containing all basic blocks and their relationships, with a designated entry point.

### LoopContext

```rust
#[derive(Debug, Clone)]
pub struct LoopContext {
    pub label_id: usize,   // Block containing the loop label
    pub cond_id: usize,    // Block containing loop condition
    pub after_id: usize,   // Block executed after loop completion
}
```

Tracks loop structure for proper handling of `break` and `continue` statements with labels.

## AST Module (ast/)

### Purpose

Handles parsing of Go source code into Abstract Syntax Trees using the `go-parser` crate.

### Key Components

#### ParsedFile Structure

```rust
pub struct ParsedFile {
    pub path: PathBuf,           // File system path
    pub ast: go_parser::ast::File, // Parsed AST representation
}
```

#### parse_project Function

**Signature:** `parse_project(root: &Path) -> Result<(FileSet, AstObjects, Vec<ParsedFile>)>`

**Step-by-step operation:**

1. **Input Validation:** Determines if the root path is a file or directory
2. **File Discovery:**
   - If directory: Recursively walks the directory tree using `WalkDir`
   - If file: Processes the single file
   - Filters for `.go` files only
3. **Parsing Process:** For each Go file:
   - Reads file content into memory
   - Calls `parse_file` from go-parser crate
   - Handles parsing errors gracefully
   - Collects successful parses into `ParsedFile` structures
4. **Error Handling:** Accumulates parsing errors and reports them
5. **Return:** Triple containing:
   - `FileSet`: Position information for error reporting
   - `AstObjects`: Parsed AST node storage
   - `Vec<ParsedFile>`: Successfully parsed files

**Error Handling Strategy:**

- Non-fatal: Parsing errors are collected and reported but don't stop processing
- Fatal: File system errors (permissions, missing files) cause immediate failure

## CFG Module (cfg/)

### Types Module (types.rs)

Defines the core data structures described above. Each structure is designed for:

- **Clonability:** All structures implement `Clone` for flexible ownership
- **Debuggability:** `Debug` trait for development and troubleshooting
- **Efficiency:** Uses `HashMap` for O(1) block lookup by ID

### Builder Module (builder.rs)

The heart of the CFG construction process.

#### Main Function: ControlFlowGraph::build()

**Signature:** `build(fset: &FileSet, func: &FuncDecl, objs: &AstObjects) -> Self`

**High-level Algorithm:**

1. Initialize graph with entry and exit blocks
2. Process function body statements sequentially
3. Handle control flow constructs (if, for, switch, etc.)
4. Resolve labels and gotos
5. Apply optimizations
6. Validate graph integrity

**Detailed Step-by-step Process:**

#### Phase 1: Initialization

```rust
let mut blocks = HashMap::<usize, BasicBlock>::new();
let mut next_id = 0;
let entry = next_id; // Always 0
next_id += 1;
let exit_id = next_id; // Always 1
```

Creates the fundamental graph structure with:

- **Entry block (ID: 0):** Function entry point
- **Exit block (ID: 1):** Function exit point
- **ID counter:** Ensures unique block identification

#### Phase 2: Statement Processing

The builder processes each statement in the function body:

##### If Statements

```go
if condition {
    // then block
} else {
    // else block
}
```

**Processing steps:**

1. **Condition Block:** Creates block for condition evaluation
2. **Then Branch:** Recursively processes then statements via `build_chain()`
3. **Else Branch:** Processes else statements (if present)
4. **Merge Block:** Creates convergence point after if-else
5. **Edge Creation:**
   - Condition → Then entry
   - Condition → Else entry (or merge if no else)
   - Then exit → Merge
   - Else exit → Merge

##### For Loops

```go
for init; condition; post {
    // loop body
}
```

**Processing steps:**

1. **Init Block:** Initialization statement (if present)
2. **Condition Block:** Loop condition evaluation
3. **Body Processing:** Recursive `build_chain()` with loop context
4. **Post Block:** Post-iteration statement (if present)
5. **After Block:** Post-loop execution
6. **Edge Creation:**
   - Init → Condition
   - Condition → Body entry (true branch)
   - Condition → After (false branch)
   - Body exit → Post (or Condition if no post)
   - Post → Condition

##### Range Loops

```go
for key, value := range collection {
    // loop body
}
```

Similar to for loops but with simplified structure (no explicit init/post).

##### Switch Statements

```go
switch expression {
case value1:
    // case 1 body
    fallthrough
case value2:
    // case 2 body
default:
    // default body
}
```

**Processing steps:**

1. **Switch Block:** Expression evaluation
2. **Case Processing:** Each case becomes a separate control flow path
3. **Fallthrough Handling:** Special logic for `fallthrough` statements
4. **Default Handling:** Default case (if present)
5. **Edge Creation:**
   - Switch → Each case entry
   - Case exit → Next case (if fallthrough) or merge block

##### Labeled Statements

```go
OuterLoop:
for {
    for {
        break OuterLoop  // Breaks to after OuterLoop
    }
}
```

**Processing steps:**

1. **Label Registration:** Records label name and associated block ID
2. **Context Creation:** For loops, creates `LoopContext` with label
3. **Nested Processing:** Handles labeled statement recursively

##### Branch Statements

- **break:** Jumps to loop exit or labeled statement
- **continue:** Jumps to loop condition or labeled statement
- **goto:** Direct jump to labeled statement
- **return:** Jumps to function exit
- **fallthrough:** Falls through to next switch case

#### Phase 3: Label Resolution

```rust
// Resolve pending gotos
for (blk_id, label_name) in pending_gotos {
    if let Some(&target) = label_map.get(&label_name) {
        if let Some(goto_block) = blocks.get_mut(&blk_id) {
            goto_block.succs = vec![target];
        }
    }
}
```

Resolves forward references where goto statements reference labels defined later in the code.

#### Phase 4: Graph Finalization

1. **Final Connections:** Ensures all blocks connect to appropriate successors
2. **Optimization:** Removes unreachable blocks and optimizes empty blocks
3. **Validation:** Checks graph integrity
4. **Exit Cleanup:** Ensures exit block has no successors

#### Helper Function: build_chain()

**Purpose:** Processes a sequence of statements within a specific context (loop, if-branch, etc.)

**Parameters:**

- `stmts`: Statement sequence to process
- `exit_id`: Function exit block for returns
- `loop_cond`: Current loop condition block (for continue)
- `loop_after`: Current loop after block (for break)
- `label_map`: Label-to-block mapping

**Returns:** `(entry_id, exit_id, pending_gotos)`

### Optimization Module (optimization.rs)

#### Unreachable Block Elimination

**Function:** `clean_unreachable_blocks()`

**Algorithm:**

1. **Reachability Analysis:** DFS from entry block
2. **Marking:** Records all reachable blocks
3. **Removal:** Deletes unreachable blocks from graph

**Benefits:**

- Reduces graph size
- Eliminates dead code blocks
- Simplifies analysis

#### Empty Block Optimization

**Function:** `optimize_empty_blocks()`

**Algorithm:**

1. **Identification:** Finds blocks with only empty statements
2. **Bypass Creation:** Redirects predecessors to successors
3. **Removal:** Eliminates the empty block
4. **Iteration:** Repeats until no more optimizations possible

**Constraints:**

- Never removes entry or exit blocks
- Preserves blocks that are label targets
- Limits iterations to prevent infinite loops

**Benefits:**

- Cleaner graph visualization
- Faster graph traversal
- Reduced memory usage

### Validation Module (validation.rs)

#### Graph Integrity Checking

**Function:** `validate_graph()`

**Checks performed:**

1. **Successor Existence:** All referenced successor blocks exist
2. **Exit Block Validation:** Only exit blocks should have no successors
3. **Connectivity:** Warns about potential graph issues

**Error Reporting:**

- Non-fatal warnings for suspicious conditions
- Detailed error messages with block IDs
- Helps identify CFG construction bugs

## Export Module (export/)

### DOT Format Export (dot.rs)

#### Purpose

Generates Graphviz DOT format for graph visualization.

#### Function: to_dot()

**Signature:** `to_dot(cfg: &ControlFlowGraph, name: &str) -> String`

**Process:**

1. **Sanitization:** Cleans function name for valid DOT identifiers
2. **Graph Declaration:** Creates directed graph with styling
3. **Node Generation:**
   - Creates node for each basic block
   - Generates descriptive labels based on statement types
   - Applies consistent styling
4. **Edge Generation:** Creates directed edges between blocks
5. **Formatting:** Produces valid DOT syntax

**Styling Features:**

- Rounded rectangular nodes
- Top-to-bottom layout
- Optimized spacing and layout
- Statement-type-based labeling

**Label Types:**

- Entry/Exit for function boundaries
- Statement type names (If, For, Return, etc.)
- Sanitized content for safety

## Public API (lib.rs)

### Exported Functions

```rust
pub use ast::{ParsedFile, parse_project};
pub use cfg::{BasicBlock, ControlFlowGraph, build_cfgs_for_file};
pub use export::to_dot;
```

### Main Entry Point: build_cfgs_for_file()

**Signature:** `build_cfgs_for_file(fset: &FileSet, objs: &AstObjects, file: &go_parser::ast::File) -> HashMap<String, ControlFlowGraph>`

**Process:**

1. **Function Discovery:** Iterates through file declarations
2. **Function Filtering:** Processes only function declarations
3. **CFG Construction:** Builds CFG for each function
4. **Collection:** Returns map of function name to CFG

## Usage Example

```rust
use cfg::{parse_project, build_cfgs_for_file, to_dot};
use std::path::Path;

// Parse Go project
let (fset, objs, files) = parse_project(Path::new("./go-project"))?;

// Build CFGs for each file
for file in files {
    let cfgs = build_cfgs_for_file(&fset, &objs, &file.ast);

    // Export each function's CFG
    for (func_name, cfg) in cfgs {
        let dot_output = to_dot(&cfg, &func_name);
        println!("Function {}: {}", func_name, dot_output);
    }
}
```

## Error Handling Strategy

### Parsing Errors

- **Collection:** Accumulates errors without stopping processing
- **Reporting:** Displays warnings for failed parses
- **Continuation:** Processes successfully parsed files

### CFG Construction Errors

- **Validation:** Post-construction integrity checks
- **Warnings:** Non-fatal issues reported to stderr
- **Graceful Degradation:** Continues with best-effort CFG

### Label Resolution Errors

- **Forward References:** Maintains pending goto list
- **Missing Labels:** Routes to exit block with warnings
- **Loop Context:** Tracks nested loop structures

## Performance Considerations

### Memory Usage

- **HashMap Storage:** O(1) block access
- **Cloning:** Minimal cloning of large structures
- **String Interning:** Efficient statement text storage

### Time Complexity

- **CFG Construction:** O(n) where n is number of statements
- **Optimization:** O(n²) worst case, typically much better
- **Validation:** O(n + e) where e is number of edges

### Scalability

- **Large Functions:** Handles complex control flow efficiently
- **Many Functions:** Processes files independently
- **Memory Bounds:** Optimization prevents excessive memory use

## Dependencies

- **go-parser (0.1.5):** Go AST parsing
- **walkdir (2):** Recursive directory traversal
- **anyhow (1):** Error handling
- **tempfile (3):** Testing utilities

## Testing Strategy

The module can be tested using the test file in `cfg/tests/cfg_tests.rs` which should include:

- Basic statement processing
- Control flow construct handling
- Label and goto resolution
- Optimization effectiveness
- Error condition handling

## Future Enhancements

Potential improvements include:

- **More Go Constructs:** Select statements, type switches
- **Advanced Optimizations:** Dead code elimination, constant propagation
- **Additional Exports:** JSON, XML, custom formats
- **Metrics Integration:** Complexity calculation, coverage analysis
- **Performance Tuning:** Parallel processing, memory optimization
