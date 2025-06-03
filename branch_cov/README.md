# Branch Coverage Module - Go Code Branch Coverage Analysis

## Overview

The Branch Coverage module is a comprehensive Rust library that analyzes branch coverage in Go programs by instrumenting source code, running tests, and tracking which branches (decision paths) were executed. Unlike statement coverage which tracks individual statements, branch coverage ensures that all possible execution paths through conditional statements, loops, and switch constructs are tested. This provides deeper insight into test completeness and helps identify untested code paths.

## Architecture

The module is organized into three main components:

```
branch_cov/
├── src/
│   ├── lib.rs            # Public API exports
│   ├── core.rs           # Main branch coverage analysis engine
│   ├── helpers.rs        # Utilities and configuration management
│   └── instrumentation.rs # Code instrumentation and branch tracking
```

## Core Data Structures

### BranchInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub branch_id: String,
    pub line: usize,
    pub branch_type: String,
    pub condition: String,
    pub is_covered: bool,
}
```

Detailed information about a specific branch in the code:

- **Branch ID:** Unique identifier in format `function:block:successor`
- **Line:** Source code line number where the branch occurs
- **Branch type:** Type of construct (if, switch, for, range, etc.)
- **Condition:** Description of the branching condition
- **Is covered:** Whether this branch was executed during testing

### UncoveredBranch

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredBranch {
    pub branch_id: String,
    pub line: usize,
    pub branch_type: String,
    pub condition: String,
    pub file_path: String,
}
```

Detailed information about branches that were not executed:

- **File path:** Source file containing the uncovered branch
- **Complete location context:** Line, type, and condition information
- **Traceability:** Links back to specific source code locations

### FunctionBranchCoverage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBranchCoverage {
    pub total_branches: usize,
    pub covered_branches: usize,
    pub coverage_percentage: f64,
    pub branches: Vec<BranchInfo>,
    pub uncovered_branches: Vec<UncoveredBranch>,
    pub function_name: String,
    pub file_path: String,
}
```

Comprehensive branch coverage metrics for a single function:

- **Total branches:** Count of all decision paths in the function
- **Covered branches:** Count of branches executed during testing
- **Coverage percentage:** Calculated coverage ratio
- **Branches:** Detailed information about all branches
- **Uncovered branches:** Specific branches that need additional testing
- **Function context:** Name and file location

### ProjectBranchCoverage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBranchCoverage {
    pub functions: HashMap<String, FunctionBranchCoverage>,
    pub total_branches: usize,
    pub covered_branches: usize,
    pub overall_coverage_percentage: f64,
    pub files_analyzed: Vec<String>,
    pub uncovered_branches: Vec<UncoveredBranch>,
    pub test_output: Option<String>,
}
```

Project-wide branch coverage analysis results:

- **Functions:** Per-function branch coverage data
- **Total branches:** Project-wide branch count
- **Covered branches:** Project-wide covered branch count
- **Overall coverage percentage:** Project-wide coverage ratio
- **Files analyzed:** List of source files processed
- **Uncovered branches:** All uncovered branches across the project
- **Test output:** Optional test execution output for debugging

### BranchCoverageOptions

```rust
#[derive(Debug, Clone)]
pub struct BranchCoverageOptions {
    pub verbose: bool,
    pub include_test_files: bool,
    pub min_coverage_threshold: f64,
    pub fail_on_low_coverage: bool,
    pub exclude_patterns: Vec<String>,
    pub simulate_coverage: bool,
    pub test_args: Vec<String>,
    pub fail_on_error: bool,
    pub timeout_seconds: u64,
}
```

Configuration options for branch coverage analysis:

- **Verbose:** Enable detailed output during analysis
- **Include test files:** Whether to analyze test files for coverage
- **Min coverage threshold:** Minimum acceptable coverage percentage
- **Fail on low coverage:** Return error if coverage is below threshold
- **Exclude patterns:** File patterns to exclude from analysis
- **Simulate coverage:** Mark all branches as covered (testing mode)
- **Test args:** Additional arguments to pass to `go test`
- **Fail on error:** Stop analysis if tests fail
- **Timeout seconds:** Maximum time to wait for test execution

## Core Module (core.rs)

### Main Analysis Functions

#### analyze_branch_coverage()

**Signature:** `analyze_branch_coverage(project_path: &Path) -> Result<ProjectBranchCoverage>`

Entry point for branch coverage analysis using default options.

**Step-by-step process:**

1. **Go installation validation:** Ensures Go toolchain is available
2. **Project parsing:** Uses CFG module to parse Go source files
3. **CFG construction:** Builds control flow graphs for all functions
4. **Branch identification:** Identifies all decision points in the code
5. **Code instrumentation:** Modifies source code to track branch execution
6. **Test execution:** Runs tests on instrumented code
7. **Coverage calculation:** Analyzes execution data to compute coverage

#### analyze_branch_coverage_with_options()

**Signature:** `analyze_branch_coverage_with_options(project_path: &Path, options: &BranchCoverageOptions) -> Result<ProjectBranchCoverage>`

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

##### Phase 2: Branch Identification and Instrumentation

```rust
let inst_data = instrument_for_branch_coverage(
    project_path, temp_path, &cfgs, &fset, &objs, options,
)?;
```

- Creates temporary directory for instrumented code
- Copies project structure and dependencies
- Identifies all branch points using control flow analysis
- Inserts tracking code at each branch location
- Generates unique branch IDs for tracking

##### Phase 3: Test Execution and Data Collection

```rust
let (covered_branches, test_output) = run_tests_and_collect_coverage(
    temp_path, options, &inst_data
)?;
```

- Builds instrumented project using `go build`
- Executes tests using `go test` with specified options
- Captures branch coverage data from test output
- Collects test execution logs for debugging

##### Phase 4: Coverage Analysis and Reporting

```rust
let result = calculate_branch_coverage(&inst_data, &covered_branches, test_output)?;
```

- Maps executed branch IDs to source locations
- Calculates per-function and project-wide coverage percentages
- Identifies specific uncovered branches
- Generates comprehensive coverage report

### Helper Functions

#### run_tests_and_collect_coverage()

**Purpose:** Executes tests on instrumented code and collects branch execution data

**Process:**

1. **Build verification:** Ensures instrumented code compiles successfully
2. **Test execution:** Runs `go test` with timeout and custom arguments
3. **Output parsing:** Extracts branch coverage markers from test output
4. **Data collection:** Builds set of executed branch IDs
5. **Error handling:** Manages test failures based on configuration

**Branch tracking mechanism:**

- Instrumented code prints `BRANCH_COV:branch_id` markers
- Test runner captures and parses these markers
- Creates set of covered branch IDs for analysis

#### calculate_branch_coverage()

**Purpose:** Processes raw branch execution data into structured coverage metrics

**Algorithm:**

1. **Data correlation:** Maps branch IDs to instrumentation metadata
2. **Coverage calculation:** Determines which branches were executed
3. **Function aggregation:** Groups branches by function for analysis
4. **Percentage computation:** Calculates coverage percentages
5. **Uncovered identification:** Identifies specific branches needing coverage
6. **Report generation:** Assembles comprehensive coverage report

## Instrumentation Module (instrumentation.rs)

### Core Data Structures

#### InstrumentationData

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationData {
    pub instrumented_functions: HashMap<String, FunctionInstrumentation>,
    pub branch_mapping: HashMap<String, BranchMapping>,
    pub total_branches: usize,
}
```

Complete instrumentation metadata for the project:

- **Instrumented functions:** Per-function instrumentation details
- **Branch mapping:** Maps branch IDs to source locations
- **Total branches:** Overall branch count for coverage calculation

#### FunctionInstrumentation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInstrumentation {
    pub function_name: String,
    pub file_path: String,
    pub branches: Vec<InstrumentedBranch>,
    pub instrumentation_points: Vec<InstrumentationPoint>,
}
```

Instrumentation details for a single function:

- **Function identification:** Name and file location
- **Branches:** All instrumented branches in the function
- **Instrumentation points:** Exact locations where tracking code was inserted

#### InstrumentedBranch

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentedBranch {
    pub branch_id: String,
    pub line: usize,
    pub branch_type: String,
    pub condition: String,
    pub instrumentation_code: String,
}
```

Details about a specific instrumented branch:

- **Unique identification:** Branch ID and source location
- **Branch classification:** Type and condition description
- **Instrumentation code:** Actual tracking code inserted

### Key Functions

#### instrument_for_branch_coverage()

**Signature:** `instrument_for_branch_coverage(project_path, output_path, cfgs, fset, objs, options) -> Result<InstrumentationData>`

Main instrumentation orchestrator that prepares Go code for branch coverage tracking.

**Detailed process:**

##### Phase 1: Project Preparation

```rust
copy_project(project_path, output_path)?;
```

- Creates complete copy of project in temporary directory
- Preserves project structure, dependencies, and configuration
- Ensures build environment integrity

##### Phase 2: Branch Discovery

For each function's control flow graph:

- **Multi-successor blocks:** Identifies blocks with multiple successors
- **Branch classification:** Categorizes branches by type (if, switch, loop, etc.)
- **Location mapping:** Records precise source code positions
- **Unique identification:** Generates branch IDs in format `function:block:successor`

##### Phase 3: Instrumentation Generation

```rust
let function_instrumentation = instrument_function(func_name, cfg, fset, objs, output_path, options)?;
```

- **Branch analysis:** Examines each basic block for branching behavior
- **Code generation:** Creates tracking calls for each branch
- **Position calculation:** Determines optimal insertion points
- **Metadata collection:** Records instrumentation details for analysis

#### instrument_function()

**Purpose:** Instruments a single function for branch coverage tracking

**Algorithm:**

1. **Block analysis:** Examines each basic block in the control flow graph
2. **Branch identification:** Finds blocks with multiple successors (branch points)
3. **Type classification:** Determines branch type (if, switch, loop, etc.)
4. **Successor mapping:** Creates separate tracking for each possible execution path
5. **Code generation:** Generates `fmt.Printf("BRANCH_COV:branch_id\n")` calls
6. **Position optimization:** Places tracking code at optimal locations

**Branch type identification:**

- **If statements:** Two branches (true/false paths)
- **Switch statements:** Multiple branches (one per case)
- **Loops:** Two branches (enter loop/skip loop)
- **Select statements:** Multiple branches (one per channel operation)

#### modify_source_files()

**Purpose:** Applies instrumentation to actual Go source files

**Process:**

1. **File grouping:** Groups instrumentation by source file
2. **Content modification:** Inserts tracking code at calculated positions
3. **Import management:** Ensures `fmt` package is imported for tracking
4. **Strategic placement:** Adds tracking calls at branch decision points
5. **File writing:** Saves instrumented code to temporary directory

**Instrumentation placement strategy:**

- **If statements:** Tracks both true and false branches
- **Switch cases:** Tracks entry to each case clause
- **Loop constructs:** Tracks loop entry and exit paths
- **Function boundaries:** Avoids duplicate instrumentation

### Branch Analysis Functions

#### get_precise_branch_line()

**Purpose:** Determines exact source line number for branch instrumentation

**Algorithm:**

1. **Statement analysis:** Examines all statements in the basic block
2. **Branching detection:** Identifies statements that create control flow branches
3. **Position extraction:** Uses AST position information for accuracy
4. **Fallback mechanisms:** Handles cases with missing position data

#### analyze_block_branch_info()

**Purpose:** Determines branch type and condition from control flow analysis

**Classification system:**

- **Conditional branches:** if statements with true/false paths
- **Multi-way branches:** switch statements with multiple cases
- **Loop branches:** for/range statements with iteration control
- **Channel branches:** select statements with channel operations

#### create_specific_condition()

**Purpose:** Creates descriptive condition text for each branch

**Strategy:**

- **If branches:** Distinguishes between true and false paths
- **Switch branches:** Identifies specific case being executed
- **Loop branches:** Describes iteration vs. completion paths
- **Generic branches:** Provides meaningful path descriptions

## Helpers Module (helpers.rs)

### Utility Functions

#### go_utils submodule

**check_go_installation()** - Validates Go toolchain availability
**is_test_file()** - Identifies Go test files for filtering
**should_exclude_file()** - Applies exclusion patterns to file paths
**find_go_files()** - Recursively discovers Go source files
**is_go_module()** - Checks for Go module structure
**ensure_go_module()** - Initializes Go module if needed

#### export submodule

**to_json()** - Serializes coverage data to JSON format
**to_csv()** - Exports coverage data as CSV for spreadsheet analysis
**ci_export()** - Generates CI/CD pipeline exit codes
**save_report()** - Saves coverage reports in various formats

### Configuration Management

#### BranchCoverageConfig

Advanced configuration options:

- **Coverage thresholds:** Minimum acceptable coverage levels
- **File exclusions:** Patterns and specific files to exclude
- **Analysis modes:** Include/exclude test files from analysis
- **Failure behavior:** Configure when analysis should fail

#### BranchCoverageAnalyzer

High-level analyzer with fluent configuration:

- **Builder pattern:** Chainable configuration methods
- **Default settings:** Sensible defaults for common use cases
- **Flexible analysis:** Supports various analysis scenarios

## Public API (lib.rs)

### Exported Functions

```rust
pub use core::{
    analyze_branch_coverage,
    analyze_branch_coverage_with_options,
    ProjectBranchCoverage,
    FunctionBranchCoverage,
    BranchCoverageOptions,
    UncoveredBranch,
    BranchInfo,
};
```

## Usage Examples

### Basic Branch Coverage Analysis

```rust
use branch_cov::analyze_branch_coverage;
use std::path::Path;

let coverage = analyze_branch_coverage(Path::new("./my-go-project"))?;
println!("Overall branch coverage: {:.1}%", coverage.overall_coverage_percentage);

for uncovered in &coverage.uncovered_branches {
    println!("Uncovered: {} at line {} in {}",
        uncovered.branch_type,
        uncovered.line,
        uncovered.file_path
    );
}
```

### Advanced Analysis with Options

```rust
use branch_cov::{analyze_branch_coverage_with_options, BranchCoverageOptions};

let options = BranchCoverageOptions {
    verbose: true,
    min_coverage_threshold: 90.0,
    fail_on_low_coverage: true,
    exclude_patterns: vec!["*_test.go".to_string(), "vendor/*".to_string()],
    test_args: vec!["-race".to_string(), "-count=3".to_string()],
    timeout_seconds: 120,
    ..Default::default()
};

let coverage = analyze_branch_coverage_with_options(
    Path::new("./my-go-project"),
    &options
)?;

for (func_name, func_coverage) in &coverage.functions {
    println!("{}: {:.1}% ({}/{} branches)",
        func_name,
        func_coverage.coverage_percentage,
        func_coverage.covered_branches,
        func_coverage.total_branches
    );

    for branch in &func_coverage.branches {
        let status = if branch.is_covered { "✓" } else { "✗" };
        println!("  {} Line {}: {} ({})",
            status,
            branch.line,
            branch.branch_type,
            branch.condition
        );
    }
}
```

### Export Coverage Data

```rust
use branch_cov::helpers::export;

let json_output = export::to_json(&coverage)?;
let csv_output = export::to_csv(&coverage);
export::save_report(&coverage, Path::new("branch_coverage.json"), "json")?;

let exit_code = export::ci_export(&coverage, 85.0);
std::process::exit(exit_code);
```

## Branch Coverage vs Statement Coverage

### Complementary Analysis

**Branch coverage** and **statement coverage** provide different insights:

**Statement Coverage:**

- **Individual statements:** Tracks execution of each code statement
- **Line-based:** Focuses on which lines were executed
- **Basic completeness:** Ensures code is reached during testing

**Branch Coverage:**

- **Decision paths:** Tracks all possible execution paths
- **Condition-based:** Focuses on decision outcomes (true/false, case selection)
- **Logical completeness:** Ensures all decision scenarios are tested

### Combined Benefits

Using both metrics together provides:

- **Statement coverage:** Identifies unreachable code
- **Branch coverage:** Identifies untested decision paths
- **Complete picture:** Comprehensive understanding of test completeness

## Error Handling Strategy

### Instrumentation Errors

- **File access:** Graceful handling of permission and I/O errors
- **Code generation:** Validates generated instrumentation syntax
- **Build failures:** Reports compilation errors with context

### Test Execution Errors

- **Build failures:** Reports compilation errors from instrumented code
- **Test timeouts:** Configurable timeout with graceful termination
- **Coverage data parsing:** Handles malformed or missing coverage markers

### Analysis Errors

- **Missing branches:** Handles incomplete instrumentation data
- **Data correlation:** Validates branch ID mappings
- **Calculation errors:** Guards against division by zero and invalid percentages

## Performance Considerations

### Memory Usage

- **Streaming processing:** Processes files individually to limit memory usage
- **Temporary files:** Uses system temp directories with automatic cleanup
- **Efficient data structures:** HashMap usage for O(1) branch lookups

### Time Complexity

- **Instrumentation:** O(n) where n is number of branches
- **Test execution:** Depends on test suite complexity and instrumentation overhead
- **Analysis:** O(m) where m is number of executed branches

### Scalability

- **Large projects:** Handles projects with thousands of branches
- **Complex control flow:** Efficiently processes deeply nested code structures
- **Test suite impact:** Minimal overhead from branch tracking code

## Integration with CFG Module

The branch coverage module depends heavily on the CFG module for:

- **Project parsing:** Uses `parse_project()` for Go source analysis
- **CFG construction:** Uses `build_cfgs_for_file()` for control flow analysis
- **Branch identification:** Leverages basic blocks and successors for branch detection
- **Position mapping:** Uses FileSet and AstObjects for source location accuracy

## Dependencies

- **cfg:** Control flow graph construction and Go parsing
- **anyhow:** Error handling and context
- **serde:** JSON serialization for coverage data
- **tempfile:** Temporary directory management
- **std::process:** Go command execution
- **std::collections:** HashMap and HashSet for data management

## Future Enhancements

Potential improvements include:

- **Path coverage integration:** Combine with path coverage analysis
- **Condition coverage:** Track individual conditions in complex boolean expressions
- **Modified condition/decision coverage (MC/DC):** Aviation-grade coverage analysis
- **Visual reporting:** Graphical representation of branch coverage
- **IDE integration:** Real-time branch coverage feedback during development
- **Performance optimization:** Reduce instrumentation overhead
- **Smart instrumentation:** Selective instrumentation based on change analysis
