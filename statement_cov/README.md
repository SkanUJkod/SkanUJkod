# Statement Coverage Module - Go Code Coverage Analysis

## Overview

The Statement Coverage module is a comprehensive Rust library that analyzes statement coverage in Go programs by instrumenting source code, running tests, and collecting execution data. It provides detailed coverage metrics at the statement level, identifying which statements were executed during testing.

## Architecture

The module is organized into three main components:

```
statement_cov/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── core.rs             # Main coverage analysis engine
│   ├── helpers.rs          # Utilities and exporters
│   └── instrumentation.rs  # Code instrumentation logic
```

## Core Data Structures

### FunctionCoverage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCoverage {
    pub total_statements: usize,
    pub covered_statements: usize,
    pub coverage_percentage: f64,
    pub uncovered_statements: Vec<usize>,
    pub uncovered_lines: Vec<usize>,
    pub uncovered_line_details: Vec<UncoveredLine>,
}
```

Represents coverage metrics for a single function including:

- **Total statements:** Count of all executable statements
- **Covered statements:** Count of statements executed during tests
- **Coverage percentage:** Calculated coverage ratio
- **Uncovered statements:** Statement IDs that were not executed
- **Uncovered lines:** Line numbers with uncovered statements
- **Uncovered line details:** Detailed information about uncovered lines

### UncoveredLine

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredLine {
    pub line: usize,
    pub stmt_type: String,
    pub stmt_ids: Vec<usize>,
}
```

Detailed information about a specific uncovered line:

- **Line:** Source code line number
- **Statement type:** Type of Go statement (if, for, assign, etc.)
- **Statement IDs:** All statement IDs on this line

### ProjectCoverage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCoverage {
    pub functions: HashMap<String, FunctionCoverage>,
    pub total_statements: usize,
    pub covered_statements: usize,
    pub overall_coverage: f64,
    pub files_analyzed: usize,
    pub test_output: Option<String>,
}
```

Complete coverage analysis results for an entire project:

- **Functions:** Per-function coverage data
- **Total statements:** Project-wide statement count
- **Covered statements:** Project-wide covered statement count
- **Overall coverage:** Project-wide coverage percentage
- **Files analyzed:** Number of Go files processed
- **Test output:** Optional test execution output

### CoverageOptions

```rust
#[derive(Debug, Clone)]
pub struct CoverageOptions {
    pub verbose: bool,
    pub fail_on_error: bool,
    pub timeout_seconds: u64,
    pub test_args: Vec<String>,
}
```

Configuration options for coverage analysis:

- **Verbose:** Enable detailed output during analysis
- **Fail on error:** Stop analysis if tests fail
- **Timeout seconds:** Maximum time to wait for test execution
- **Test args:** Additional arguments to pass to `go test`

## Core Module (core.rs)

### Main Analysis Functions

#### analyze_statement_coverage()

**Signature:** `analyze_statement_coverage(project_path: &Path) -> Result<ProjectCoverage>`

Entry point for statement coverage analysis using default options.

**Step-by-step process:**

1. **Path validation:** Ensures the project path exists
2. **Go installation check:** Verifies Go is installed and accessible
3. **Project parsing:** Uses CFG module to parse Go source files
4. **CFG construction:** Builds control flow graphs for all functions
5. **Instrumentation:** Instruments source code with coverage tracking
6. **Test execution:** Runs tests on instrumented code
7. **Coverage calculation:** Analyzes execution data to compute coverage

#### analyze_statement_coverage_with_options()

**Signature:** `analyze_statement_coverage_with_options(project_path: &Path, options: &CoverageOptions) -> Result<ProjectCoverage>`

Advanced analysis function with custom configuration options.

**Detailed workflow:**

##### Phase 1: Project Discovery and Validation

```rust
let (fset, objs, files) = parse_project(project_path).context("Failed to parse Go project")?;
```

- Recursively discovers all `.go` files in the project
- Parses each file into an Abstract Syntax Tree (AST)
- Builds control flow graphs for all functions
- Validates that functions exist for analysis

##### Phase 2: Code Instrumentation

```rust
let instrumentation_data = instrumentation::generate_instrumented_project(
    &cfgs, &files, &fset, &objs, project_path, temp_path,
).context("Failed to instrument project")?;
```

- Creates a temporary directory for instrumented code
- Copies project structure (go.mod, dependencies, etc.)
- Instruments each Go file with statement tracking calls
- Generates coverage tracking infrastructure
- Creates test runner that saves coverage data

##### Phase 3: Test Execution and Data Collection

```rust
let (coverage_data, test_output) = run_tests_and_collect_coverage(temp_path, options)?;
```

- Builds the instrumented project using `go build`
- Executes tests using `go test` with specified options
- Captures test output for debugging
- Collects coverage data from JSON output file
- Handles test failures based on configuration

##### Phase 4: Coverage Analysis

```rust
let mut coverage = calculate_statement_coverage(&instrumentation_data, &coverage_data)?;
```

- Maps executed statement IDs to source locations
- Calculates per-function coverage percentages
- Identifies uncovered statements and lines
- Computes project-wide coverage metrics
- Associates coverage data with source line numbers

### Helper Functions

#### run_tests_and_collect_coverage()

**Purpose:** Executes tests on instrumented code and collects coverage data

**Process:**

1. **Build phase:** Compiles instrumented Go code
2. **Test execution:** Runs `go test` with timeout and custom arguments
3. **Output capture:** Streams test output for verbose mode
4. **Error handling:** Manages test failures based on configuration
5. **Data collection:** Reads coverage data from generated JSON file
6. **Validation:** Ensures coverage data file exists and is valid

#### calculate_statement_coverage()

**Purpose:** Processes raw coverage data into structured coverage metrics

**Algorithm:**

1. **Data mapping:** Maps statement IDs to source locations using instrumentation data
2. **Coverage calculation:** Determines which statements were executed
3. **Line aggregation:** Groups statements by source line numbers
4. **Percentage computation:** Calculates coverage percentages per function
5. **Uncovered identification:** Identifies specific uncovered statements and lines
6. **Result compilation:** Assembles final coverage report structure

## Instrumentation Module (instrumentation.rs)

### Core Data Structures

#### StatementMapping

```rust
#[derive(Debug, Clone, Serialize)]
pub struct StatementMapping {
    pub stmt_id: usize,
    pub block_id: usize,
    pub stmt_index: usize,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub func_name: String,
    pub stmt_type: String,
}
```

Maps statement IDs to source code locations and metadata:

- **Statement ID:** Unique identifier for tracking
- **Block ID:** CFG basic block containing the statement
- **Statement index:** Position within the basic block
- **Location data:** Precise source position information
- **Function name:** Containing function for organization
- **Statement type:** Go statement classification

#### InstrumentationData

```rust
#[derive(Debug)]
pub struct InstrumentationData {
    pub statement_mappings: HashMap<String, Vec<StatementMapping>>,
    pub total_statements_per_function: HashMap<String, usize>,
}
```

Complete instrumentation metadata:

- **Statement mappings:** Per-function statement location data
- **Total statements:** Statement count per function for coverage calculation

### Key Functions

#### generate_instrumented_project()

**Signature:** `generate_instrumented_project(cfgs, parsed_files, fset, objs, source_path, output_path) -> Result<InstrumentationData>`

Main instrumentation orchestrator that transforms a Go project for coverage tracking.

**Detailed process:**

##### Phase 1: Project Structure Setup

```rust
copy_project_structure(source_path, output_path)?;
```

- Creates temporary directory structure
- Copies `go.mod` and `go.sum` files
- Copies non-Go files (configs, data files, etc.)
- Generates module file if none exists
- Preserves project dependencies and structure

##### Phase 2: Coverage Infrastructure Generation

```rust
let coverage_code = generate_coverage_tracker();
fs::write(&coverage_file, coverage_code)?;
```

- Generates `coverage_tracker.go` with tracking functions
- Implements thread-safe statement execution recording
- Provides JSON export functionality for coverage data
- Handles program termination and data persistence

##### Phase 3: File-by-File Instrumentation

```rust
let instrumented_content = instrument_file_advanced(
    &source_content, &parsed_file.path, cfgs, fset, objs,
    &functions_in_file, &mut instrumentation_data,
)?;
```

- Processes each Go source file individually
- Maps functions to their source files
- Instruments statements with tracking calls
- Preserves original code structure and formatting
- Maintains source location accuracy

##### Phase 4: Test Infrastructure

```rust
let test_code = generate_test_runner(&instrumentation_data);
fs::write(&test_file, test_code)?;
```

- Generates test runner that initializes coverage tracking
- Creates `TestMain` function for coverage data export
- Ensures coverage data is saved after test completion
- Provides fallback coverage test if no tests exist

#### instrument_file_advanced()

**Purpose:** Instruments individual Go source files with statement tracking

**Algorithm:**

1. **Statement discovery:** Identifies all executable statements in CFG blocks
2. **Location mapping:** Maps statements to precise source positions
3. **Instrumentation point calculation:** Determines optimal insertion points
4. **Code generation:** Inserts `stmt_hit()` calls at appropriate locations
5. **Metadata collection:** Records statement mappings for later analysis

**Statement filtering logic:**

- Skips entry/exit blocks (no executable code)
- Excludes implicit empty statements
- Filters statements not in current file
- Handles special statement types (labeled, if, block, etc.)

#### find_instrumentation_point()

**Purpose:** Determines optimal location for inserting coverage tracking calls

**Strategy by statement type:**

- **If statements:** Insert before condition evaluation
- **Labeled statements:** Insert after label declaration
- **Block statements:** Insert at block entry
- **Other statements:** Insert with proper indentation

#### apply_instrumentation()

**Purpose:** Inserts tracking calls into source code while preserving formatting

**Process:**

1. **Line-by-line processing:** Iterates through original source lines
2. **Instrumentation insertion:** Adds `stmt_hit()` calls at calculated points
3. **Indentation preservation:** Maintains original code formatting
4. **Comment annotation:** Adds line number comments for debugging

### Generated Code Components

#### Coverage Tracker (coverage_tracker.go)

Generated Go code that provides runtime coverage tracking:

**Key features:**

- **Thread-safe tracking:** Uses mutex for concurrent test execution
- **Statement recording:** Maps function names to executed statement IDs
- **Data persistence:** Exports coverage data to JSON file
- **Signal handling:** Ensures data is saved on program termination
- **Summary reporting:** Provides human-readable coverage summary

**Core functions:**

- `stmt_hit(funcName, stmtID)` - Records statement execution
- `saveCoverageData()` - Exports data to JSON file
- `printCoverage()` - Displays coverage summary
- `initCoverage()` - Initializes tracking system

#### Test Runner (coverage_test.go)

Generated test infrastructure:

**Components:**

- **TestMain function:** Orchestrates test execution and coverage export
- **Coverage verification test:** Ensures tracking system is functional
- **Data export timing:** Handles async operations and data persistence
- **Exit code handling:** Preserves original test results

## Helpers Module (helpers.go)

### Utility Functions

#### go_utils submodule

**check_go_installation()** - Verifies Go is installed and accessible
**find_go_files()** - Discovers Go source files while excluding test files and vendor directories

#### export submodule

**to_json()** - Serializes coverage data to JSON format
**to_csv()** - Exports coverage data as CSV for spreadsheet analysis
**ci_export()** - Generates CI/CD pipeline outputs and exit codes

### Configuration Structures

#### CoverageConfig

Provides advanced configuration options:

- **Minimum coverage thresholds**
- **Function/pattern exclusions**
- **HTML report generation**
- **Failure behavior configuration**

#### CoverageAnalyzer

High-level analyzer with configuration management:

- **Threshold enforcement**
- **Custom exclusion patterns**
- **Reporting options**

## Public API (lib.rs)

### Exported Functions

```rust
pub use core::{
    analyze_statement_coverage,
    analyze_statement_coverage_with_options,
    ProjectCoverage,
    FunctionCoverage,
    CoverageOptions,
    UncoveredLine,
};
```

## Usage Examples

### Basic Analysis

```rust
use statement_cov::analyze_statement_coverage;
use std::path::Path;

let coverage = analyze_statement_coverage(Path::new("./my-go-project"))?;
println!("Overall coverage: {:.1}%", coverage.overall_coverage);
```

### Advanced Analysis with Options

```rust
use statement_cov::{analyze_statement_coverage_with_options, CoverageOptions};

let options = CoverageOptions {
    verbose: true,
    fail_on_error: false,
    timeout_seconds: 600,
    test_args: vec!["-race".to_string(), "-count=3".to_string()],
};

let coverage = analyze_statement_coverage_with_options(
    Path::new("./my-go-project"),
    &options
)?;

for (func_name, func_cov) in &coverage.functions {
    println!("{}: {:.1}% ({}/{} statements)",
        func_name,
        func_cov.coverage_percentage,
        func_cov.covered_statements,
        func_cov.total_statements
    );
}
```

### Export Coverage Data

```rust
use statement_cov::helpers::export;

let json_output = export::to_json(&coverage)?;
let csv_output = export::to_csv(&coverage);
let exit_code = export::ci_export(&coverage, 80.0);
```

## Error Handling Strategy

### Instrumentation Errors

- **File access:** Graceful handling of permission and I/O errors
- **Parsing failures:** Continues analysis with available functions
- **Code generation:** Validates generated code syntax

### Test Execution Errors

- **Build failures:** Reports compilation errors with context
- **Test timeouts:** Configurable timeout with graceful termination
- **Coverage data:** Validates JSON format and completeness

### Analysis Errors

- **Missing data:** Handles incomplete coverage information
- **Mapping errors:** Validates statement ID mappings
- **Calculation errors:** Guards against division by zero and invalid percentages

## Performance Considerations

### Memory Usage

- **Streaming processing:** Processes files individually to limit memory usage
- **Temporary files:** Uses system temp directories with automatic cleanup
- **Data structures:** Efficient HashMap usage for O(1) lookups

### Time Complexity

- **Instrumentation:** O(n) where n is number of statements
- **Test execution:** Depends on test suite complexity
- **Analysis:** O(m) where m is number of executed statements

### Scalability

- **Large projects:** Handles projects with thousands of functions
- **Complex tests:** Supports long-running test suites
- **Concurrent execution:** Thread-safe coverage tracking

## Integration with CFG Module

The statement coverage module heavily depends on the CFG module for:

- **Project parsing:** Uses `parse_project()` for Go source analysis
- **CFG construction:** Uses `build_cfgs_for_file()` for control flow analysis
- **Statement identification:** Leverages basic blocks for statement enumeration
- **Location mapping:** Uses FileSet and AstObjects for source position mapping

## Dependencies

- **cfg:** Control flow graph construction and Go parsing
- **anyhow:** Error handling and context
- **serde:** JSON serialization for coverage data
- **tempfile:** Temporary directory management
- **std::process:** Go command execution
- **std::collections:** HashMap and HashSet for data management

## Future Enhancements

Potential improvements include:

- **Branch coverage integration:** Combine with branch coverage analysis
- **HTML report generation:** Rich visual coverage reports
- **IDE integration:** VS Code extension for live coverage display
- **Incremental analysis:** Only analyze changed functions
- **Performance optimization:** Parallel file processing and analysis
