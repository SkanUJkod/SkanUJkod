# SkanUJkod

A code analysis tool for Go programs. SkanUJkod analyzes Go code to extract control flow graphs, calculate cyclomatic complexity, and measure statement coverage.

## Features

- **Control Flow Graph (CFG) Generation**: Build visual representations of program control flow
- **Cyclomatic Complexity Analysis**: Calculate McCabe's cyclomatic complexity metric
- **Statement Coverage Analysis**: Measure code coverage from test execution
- **Graph Visualization**: Generate DOT files and convert them to images

## Usage

The tool provides a command-line interface with several subcommands:

```
Usage:
  skan-uj-kod <command> [options] <path>

Commands:
  cfg                   - Display CFG in debug format
  dot <FuncName>        - Print DOT format for selected function
  dot-all               - Generate DOT files for all functions
  stmt-cov              - Display statement coverage
  branch-cov            - Display branch coverage analysis
  complexity            - Display cyclomatic complexity analysis
  help                  - Show this help message

Options:
  --json                - Output in JSON format
  --output=<file>       - Write output to file
  --image=<format>      - Generate image (png, svg, pdf) from DOT
  --verbose             - Show detailed information
```

## Examples

### Control Flow Graph Analysis

Display CFGs in debug format:

```bash
cargo run -- cfg /path/to/go-project
```

Generate DOT representation for a specific function:

```bash
cargo run -- dot getFibonacci /path/to/go-project
```

Generate DOT files for all functions:

```bash
cargo run -- dot-all --output=./dot_files /path/to/go-project
```

### Graph Visualization

Generate PNG image for a specific function:

```bash
cargo run -- dot getDayName --image=png /path/to/go-project
```

Generate SVG images for all functions:

```bash
cargo run -- dot-all --image=svg --output=./graph_images /path/to/go-project
```

Generate PDF images for all functions:

```bash
cargo run -- dot-all --image=pdf --output=./pdf_graphs /path/to/go-project
```

### Cyclomatic Complexity Analysis

Display cyclomatic complexity metrics:

```bash
cargo run -- complexity /path/to/go-project
```

Display detailed complexity information:

```bash
cargo run -- complexity --verbose /path/to/go-project
```

Export complexity data to JSON:

```bash
cargo run -- complexity --json --output=complexity_report.json /path/to/go-project
```

### Statement Coverage Analysis

Display statement coverage:

```bash
cargo run -- stmt-cov /path/to/go-project
```

Display detailed coverage information:

```bash
cargo run -- stmt-cov --verbose /path/to/go-project
```

Export coverage data to JSON:

```bash
cargo run -- stmt-cov --json --output=coverage_report.json /path/to/go-project
```

### Branch Coverage Analysis

Display branch coverage metrics:

```bash
cargo run -- branch-cov /path/to/go-project
```

Display detailed branch coverage information with uncovered branches:

```bash
cargo run -- branch-cov --verbose /path/to/go-project
```

Export branch coverage data to JSON:

```bash
cargo run -- branch-cov --json --output=branch_coverage_report.json /path/to/go-project
```

Example output:

```
=== Branch Coverage Report ===

Files analyzed: 3
Functions found: 5
Total branches: 12
Covered branches: 8
Overall coverage: 66.7%

Function Details:
calculateGrade
  Coverage: 50.0% (2/4)
  Uncovered branches:
    Line 15: else (grade < 60)
    Line 18: else if (grade < 80)

processUser
  Coverage: 100.0% (3/3)

validateInput
  Coverage: 75.0% (3/4)
  Uncovered branches:
    Line 42: else (input == nil)
```

## Requirements

- Rust 1.56 or later
- Go 1.17 or later
- Graphviz (for image generation from DOT files)

To install Graphviz:

```bash
# macOS
brew install graphviz

# Ubuntu/Debian
sudo apt install graphviz

# Windows
choco install graphviz
```
