# Cyclomatic Complexity Module - Go Code Complexity Analysis

## Overview

The Cyclomatic Complexity module is a comprehensive Rust library that analyzes both cyclomatic and cognitive complexity in Go programs. It uses control flow graphs to calculate McCabe's cyclomatic complexity metric and also provides cognitive complexity analysis, which considers nesting levels and decision point complexity. This module helps identify overly complex functions that may be difficult to test, maintain, or understand.

## Architecture

The module is organized into three main components:

```
cyclomatic_complexity/
├── src/
│   ├── lib.rs       # Public API exports
│   ├── mod.rs       # Alternative entry point
│   ├── analyzer.rs  # Core complexity analysis engine
│   └── helpers.rs   # Utilities and complexity classifications
```

## Core Data Structures

### FunctionComplexity

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub lines_of_code: usize,
    pub decision_points: Vec<DecisionPoint>,
    pub complexity_level: ComplexityLevel,
    pub nesting_depth_max: usize,
}
```

Comprehensive complexity metrics for a single function:

- **Cyclomatic complexity:** McCabe's metric (E - N + 2P)
- **Cognitive complexity:** Considers nesting and decision point types
- **Lines of code:** Count of executable statements
- **Decision points:** Detailed breakdown of complexity contributors
- **Complexity level:** Classification (Low/Moderate/High/VeryHigh)
- **Maximum nesting depth:** Deepest level of nested control structures

### DecisionPoint

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    pub line: usize,
    pub stmt_type: String,
    pub nesting_level: usize,
}
```

Detailed information about a complexity-contributing statement:

- **Line:** Source code line number
- **Statement type:** Type of Go construct (if, for, switch, etc.)
- **Nesting level:** Depth within nested control structures

### ProjectComplexity

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectComplexity {
    pub functions: HashMap<String, FunctionComplexity>,
    pub total_functions: usize,
    pub average_complexity: f64,
    pub max_complexity: usize,
    pub max_complexity_function: String,
    pub complexity_distribution: HashMap<String, usize>,
    pub files_analyzed: usize,
}
```

Project-wide complexity analysis results:

- **Functions:** Per-function complexity data
- **Total functions:** Count of analyzed functions
- **Average complexity:** Mean cyclomatic complexity across all functions
- **Max complexity:** Highest cyclomatic complexity found
- **Max complexity function:** Name of the most complex function
- **Complexity distribution:** Count of functions in each complexity category
- **Files analyzed:** Number of Go files processed

### ComplexityOptions

```rust
#[derive(Debug, Clone)]
pub struct ComplexityOptions {
    pub verbose: bool,
    pub include_cognitive: bool,
    pub max_allowed_complexity: usize,
    pub fail_on_high_complexity: bool,
}
```

Configuration options for complexity analysis:

- **Verbose:** Enable detailed output during analysis
- **Include cognitive:** Calculate cognitive complexity in addition to cyclomatic
- **Max allowed complexity:** Threshold for acceptable complexity
- **Fail on high complexity:** Return error if threshold is exceeded

### ComplexityLevel

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    Low,      // 1-5
    Moderate, // 6-10
    High,     // 11-20
    VeryHigh, // >20
}
```

Classification of complexity levels based on industry standards and maintainability research.

## Analyzer Module (analyzer.rs)

### Main Analysis Functions

#### analyze_cyclomatic_complexity()

**Signature:** `analyze_cyclomatic_complexity(project_path: &Path) -> Result<ProjectComplexity>`

Entry point for complexity analysis using default options.

**Step-by-step process:**

1. **Path validation:** Ensures the project path exists
2. **Go installation check:** Verifies Go toolchain availability
3. **Project parsing:** Uses CFG module to parse Go source files
4. **CFG construction:** Builds control flow graphs for all functions
5. **Complexity calculation:** Applies McCabe's formula and cognitive analysis
6. **Statistical analysis:** Computes project-wide metrics and distributions

#### analyze_cyclomatic_complexity_with_options()

**Signature:** `analyze_cyclomatic_complexity_with_options(project_path: &Path, options: &ComplexityOptions) -> Result<ProjectComplexity>`

Advanced analysis function with custom configuration.

**Detailed workflow:**

##### Phase 1: Project Discovery and Validation

```rust
let (fset, objs, files) = parse_project(project_path).context("Failed to parse Go project")?;
```

- Recursively discovers all `.go` files in the project
- Parses each file into Abstract Syntax Trees (ASTs)
- Builds control flow graphs for all functions
- Validates that functions exist for analysis

##### Phase 2: Per-Function Complexity Analysis

```rust
let func_complexity = analyze_function_complexity(cfg, &objs, &fset, options)?;
```

For each function:

- Calculates cyclomatic complexity using control flow graph structure
- Identifies and catalogs all decision points
- Computes cognitive complexity if enabled
- Determines complexity classification level

##### Phase 3: Project-Wide Statistical Analysis

- Identifies the most complex function
- Calculates average complexity across all functions
- Creates complexity distribution histogram
- Validates against configured thresholds

### Core Algorithm: analyze_function_complexity()

#### Cyclomatic Complexity Calculation

**Formula:** `CC = E - N + 2P`

- **E:** Number of edges in the control flow graph
- **N:** Number of nodes (basic blocks) in the graph
- **P:** Number of connected components (always 1 for a single function)

**Implementation details:**

```rust
let num_nodes = cfg.blocks.len();
let num_edges: usize = cfg.blocks.values()
    .map(|block| block.succs.len())
    .sum();
let cyclomatic_complexity = num_edges.saturating_add(2).saturating_sub(num_nodes).max(1);
```

**Special cases handled:**

- **Empty functions:** Guaranteed minimum complexity of 1
- **Entry/exit blocks:** Properly excluded from statement analysis
- **Overflow protection:** Uses saturating arithmetic to prevent panics

#### Decision Point Analysis

**Purpose:** Identifies all control flow decision points for detailed complexity breakdown

**Supported Go constructs:**

- **if statements:** Conditional branching
- **for loops:** Loop iteration control
- **switch statements:** Multi-way branching
- **range loops:** Collection iteration
- **type switches:** Type-based branching
- **case clauses:** Individual switch cases
- **select statements:** Channel operation selection

**Algorithm:**

1. **Recursive traversal:** Processes nested control structures
2. **Position mapping:** Records exact source code locations
3. **Nesting tracking:** Maintains current nesting depth
4. **Statement classification:** Categorizes each decision point type

#### Cognitive Complexity Calculation

**Purpose:** Measures code complexity from a human cognitive perspective

**Key principles:**

- **Base complexity:** Each decision point adds to complexity
- **Nesting penalty:** Deeper nesting increases cognitive load exponentially
- **Statement type weighting:** Different constructs have different cognitive costs

**Algorithm:**

```rust
fn calculate_cognitive_complexity(stmt: &Stmt, objs: &AstObjects, nesting_level: usize) -> (usize, usize) {
    match stmt {
        Stmt::If(_) => {
            let complexity = 1 + nesting_level; // Base + nesting penalty
            // Recursively analyze nested statements
        }
        // Similar logic for other decision-making constructs
    }
}
```

**Nesting penalty calculation:**

- **Level 0:** No penalty
- **Level 1:** +1 penalty per decision point
- **Level 2:** +2 penalty per decision point
- **Level N:** +N penalty per decision point

### Statement Processing Functions

#### process_statement_for_decision_points()

**Purpose:** Recursively identifies all decision points in a statement and its children

**Algorithm:**

1. **Primary analysis:** Checks if current statement is a decision point
2. **Recursive descent:** Processes all nested statements
3. **Nesting tracking:** Increments nesting level for child statements
4. **Comprehensive coverage:** Handles all Go control flow constructs

**Nested structure handling:**

- **If statements:** Processes both then and else branches
- **Loops:** Analyzes loop bodies with incremented nesting
- **Switch statements:** Processes all case clauses
- **Block statements:** Analyzes all contained statements

#### analyze_statement()

**Purpose:** Determines if a specific statement is a decision point

**Decision point criteria:**

- **Control flow impact:** Statement affects program execution flow
- **Branching behavior:** Creates multiple possible execution paths
- **Complexity contribution:** Adds to cognitive load for understanding

## Helpers Module (helpers.rs)

### Utility Functions

#### go_utils submodule

**check_go_installation()** - Validates Go toolchain availability
**is_test_file()** - Identifies Go test files for potential filtering

### Complexity Classification

#### ComplexityLevel Enumeration

Based on software engineering research and industry best practices:

- **Low (1-5):** Simple functions, easy to test and maintain
- **Moderate (6-10):** Acceptable complexity, may benefit from refactoring
- **High (11-20):** Complex functions, difficult to test thoroughly
- **Very High (>20):** Overly complex, strong refactoring candidate

**Research basis:**

- McCabe's original research suggesting 10 as a practical upper limit
- Industry experience with maintenance costs
- Testing complexity correlation with cyclomatic complexity

## Public API (lib.rs and mod.rs)

### Exported Functions

```rust
pub use analyzer::{
    analyze_cyclomatic_complexity,
    analyze_cyclomatic_complexity_with_options,
    FunctionComplexity,
    ProjectComplexity,
    ComplexityOptions,
    DecisionPoint,
};
pub use helpers::ComplexityLevel;
```

### Alternative Entry Points

The module provides two equivalent APIs:

- **Direct analyzer functions:** `analyze_cyclomatic_complexity()`
- **Wrapper functions:** `analyze_project()` and `analyze_project_with_options()`

## Usage Examples

### Basic Complexity Analysis

```rust
use cyclomatic_complexity::analyze_cyclomatic_complexity;
use std::path::Path;

let complexity = analyze_cyclomatic_complexity(Path::new("./my-go-project"))?;
println!("Average complexity: {:.1}", complexity.average_complexity);
println!("Most complex function: {} ({})",
    complexity.max_complexity_function,
    complexity.max_complexity
);
```

### Advanced Analysis with Options

```rust
use cyclomatic_complexity::{analyze_cyclomatic_complexity_with_options, ComplexityOptions};

let options = ComplexityOptions {
    verbose: true,
    include_cognitive: true,
    max_allowed_complexity: 15,
    fail_on_high_complexity: true,
};

let complexity = analyze_cyclomatic_complexity_with_options(
    Path::new("./my-go-project"),
    &options
)?;

for (func_name, func_complexity) in &complexity.functions {
    println!("{}: CC={}, Cognitive={}, Level={}",
        func_name,
        func_complexity.cyclomatic_complexity,
        func_complexity.cognitive_complexity,
        func_complexity.complexity_level
    );

    for decision_point in &func_complexity.decision_points {
        println!("  Line {}: {} (nesting: {})",
            decision_point.line,
            decision_point.stmt_type,
            decision_point.nesting_level
        );
    }
}
```

### Complexity Distribution Analysis

```rust
println!("Complexity Distribution:");
for (level, count) in &complexity.complexity_distribution {
    println!("  {}: {} functions", level, count);
}
```

## Mathematical Foundation

### McCabe's Cyclomatic Complexity

**Formula:** `CC = E - N + 2P`

**Theoretical basis:**

- **Graph theory:** Based on control flow graph structure
- **Branching paths:** Measures independent execution paths
- **Testing correlation:** Higher complexity requires more test cases

**Practical interpretation:**

- **Minimum value:** 1 (linear code with no branching)
- **Each decision point:** Typically adds 1 to complexity
- **Nested decisions:** Can add complexity multiplicatively

### Cognitive Complexity

**Enhancement over cyclomatic complexity:**

- **Human factors:** Considers cognitive load of understanding code
- **Nesting penalty:** Recognizes increased difficulty of nested structures
- **Statement weighting:** Different constructs have different cognitive costs

**Calculation approach:**

- **Base score:** Each decision point contributes base complexity
- **Nesting multiplier:** Penalty increases with nesting depth
- **Accumulative:** Complexity accumulates through nested structures

## Error Handling Strategy

### Analysis Errors

- **Path validation:** Ensures project directory exists
- **Go toolchain:** Validates Go installation availability
- **Parsing failures:** Graceful handling of unparseable files
- **Empty projects:** Appropriate handling of projects with no functions

### Threshold Enforcement

- **Configurable limits:** User-defined complexity thresholds
- **Failure modes:** Optional hard failures for CI/CD integration
- **Detailed reporting:** Specific function and complexity information

## Performance Considerations

### Time Complexity

- **CFG analysis:** O(n) where n is number of statements
- **Recursive traversal:** O(d) where d is maximum nesting depth
- **Project analysis:** O(f) where f is number of functions

### Memory Usage

- **CFG storage:** Efficient representation of control flow graphs
- **Decision point tracking:** Minimal overhead for complexity metrics
- **Batch processing:** Processes functions independently

### Scalability

- **Large projects:** Handles projects with thousands of functions
- **Deep nesting:** Efficiently processes deeply nested code structures
- **Memory bounds:** Controlled memory usage through streaming analysis

## Integration with CFG Module

The complexity module depends heavily on the CFG module for:

- **Project parsing:** Uses `parse_project()` for Go source analysis
- **CFG construction:** Uses `build_cfgs_for_file()` for control flow analysis
- **Graph structure:** Leverages basic blocks and edges for complexity calculation
- **Statement analysis:** Uses AST nodes for decision point identification

## Dependencies

- **cfg:** Control flow graph construction and Go parsing
- **anyhow:** Error handling and context
- **serde:** JSON serialization for analysis results
- **go_parser:** AST analysis and source position mapping
- **std::collections:** HashMap for efficient data management

## Complexity Metrics Comparison

### Cyclomatic vs Cognitive Complexity

**Cyclomatic Complexity:**

- **Objective measurement:** Based purely on control flow graph structure
- **Testing focus:** Correlates with number of test cases needed
- **Industry standard:** Widely used and understood metric

**Cognitive Complexity:**

- **Human-centered:** Focuses on code understandability
- **Nesting awareness:** Recognizes increased difficulty of nested code
- **Maintenance focus:** Better predictor of maintenance difficulty

**Use cases:**

- **Testing strategy:** Use cyclomatic complexity for test case planning
- **Code review:** Use cognitive complexity for refactoring decisions
- **Quality gates:** Combine both metrics for comprehensive assessment

## Future Enhancements

Potential improvements include:

- **Essential complexity:** Measure of irreducible complexity
- **Halstead metrics:** Integration with code vocabulary analysis
- **Maintainability index:** Composite metric combining multiple factors
- **Visual reporting:** Graphical representation of complexity trends
- **IDE integration:** Real-time complexity feedback during development
- **Historical analysis:** Tracking complexity changes over time
