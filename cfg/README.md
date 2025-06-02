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
