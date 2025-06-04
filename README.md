# SkanUJkod

A code analysis tool for Go programs. SkanUJkod analyzes Go code to extract control flow graphs, calculate cyclomatic complexity, and measure statement coverage.

## Features

- **Control Flow Graph (CFG) Generation**: Build visual representations of program control flow
- **Cyclomatic Complexity Analysis**: Calculate McCabe's cyclomatic complexity metric
- **Statement Coverage Analysis**: Measure code coverage from test execution
- **Branch Coverage Analysis**: Analyze branch coverage and identify uncovered paths
- **Graph Visualization**: Generate DOT files and convert them to images (PNG, SVG, PDF)
- **📊 HTML Reports**: Generate professional interactive HTML reports
- **⚙️ Configuration Files**: Use TOML configuration files for consistent settings
- **🎯 Comprehensive Analysis**: Run all analyses at once with health scoring
- **🎨 Colorful CLI**: Modern command-line interface with progress indicators

## Installation and Setup

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd SkanUJkod

# Build the project
cargo build --release

# The binary will be available at ./target/release/skan-uj-kod
```

### Dependencies

- Rust 1.70 or later
- Go 1.19 or later
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

## Usage

The tool provides a modern command-line interface with several subcommands:

```
Usage: skan-uj-kod <COMMAND>

Commands:
  cfg          Display Control Flow Graph in debug format
  dot          Generate DOT format for a specific function
  dot-all      Generate DOT files for all functions
  stmt-cov     Analyze statement coverage
  branch-cov   Analyze branch coverage
  complexity   Analyze cyclomatic complexity
  full         Run comprehensive analysis (all metrics)
  init-config  Generate default configuration file
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose        Enable verbose output
  -j, --json          Output in JSON format
  -o, --output <OUTPUT>    Output file path
  -c, --config <CONFIG>    Configuration file path
  -h, --help          Print help
  -V, --version       Print version
```

## 📊 Report Generation

### HTML Reports

Generate beautiful, interactive HTML reports with all analysis results:

```bash
# Generate comprehensive HTML report
./target/release/skan-uj-kod full go-code/example-go --html

# Generate HTML report with custom output path
./target/release/skan-uj-kod full go-code/example-go --html --output=./reports/analysis.html

# Generate HTML report with specific coverage threshold
./target/release/skan-uj-kod full go-code/example-go --html --threshold=85
```

**HTML Report Features:**

- 📈 Interactive charts and progress bars
- 📋 Sortable tables with function details
- 🎯 Coverage and complexity metrics visualization
- 📱 Responsive design for mobile and desktop
- 🔍 Detailed breakdowns by function
- 💡 Automatic recommendations based on analysis

### JSON Reports

Export structured data for integration with other tools:

```bash
# Export comprehensive analysis to JSON
./target/release/skan-uj-kod full go-code/example-go --json --output=analysis.json

# Individual analysis exports
./target/release/skan-uj-kod complexity go-code/example-go --json --output=complexity.json
./target/release/skan-uj-kod stmt-cov go-code/example-go --json --output=coverage.json
./target/release/skan-uj-kod branch-cov go-code/example-go --json --output=branches.json
```

**JSON Structure Example:**

```json
{
  "statement_coverage": {
    "overall_coverage": 78.5,
    "functions": { ... }
  },
  "branch_coverage": {
    "overall_coverage_percentage": 65.2,
    "functions": { ... }
  },
  "complexity": {
    "average_complexity": 4.2,
    "max_complexity": 15,
    "functions": { ... }
  },
  "summary": {
    "overall_health": 82.3,
    "recommendations": [
      "Consider adding more unit tests to improve statement coverage",
      "Review 2 functions with very high complexity (>15)"
    ]
  }
}
```

## ⚙️ Configuration Management

### Generate Configuration File

Create a default configuration file for consistent analysis settings:

```bash
# Generate default config file
./target/release/skan-uj-kod init-config

# Generate config with custom name
./target/release/skan-uj-kod init-config my-config.toml
```

### Configuration File Example

`skan-uj-kod.toml`:

```toml
[coverage]
threshold = 80.0
timeout_seconds = 30
exclude_patterns = ["*_test.go", "vendor/*", "testdata/*"]
include_test_files = false
fail_on_low_coverage = false

[complexity]
max_complexity = 10
cognitive_threshold = 15
lines_of_code_threshold = 50

[output]
format = "text"
output_dir = "./reports"
verbose = false
colored = true

[analysis]
parallel_processing = true
cache_results = true
incremental_analysis = false
```

### Using Configuration Files

```bash
# Use custom config file
./target/release/skan-uj-kod full go-code/example-go --config=my-config.toml --html

# Config settings are applied automatically
./target/release/skan-uj-kod complexity go-code/example-go --config=my-config.toml
```

## 🎯 Comprehensive Analysis

Run all analyses at once with intelligent health scoring:

```bash
# Basic comprehensive analysis
./target/release/skan-uj-kod full go-code/example-go

# Comprehensive analysis with HTML report
./target/release/skan-uj-kod full go-code/example-go --html

# Comprehensive analysis with custom thresholds
./target/release/skan-uj-kod full go-code/example-go --threshold=90 --html

# Verbose comprehensive analysis
./target/release/skan-uj-kod full go-code/example-go --verbose
```

**Comprehensive Analysis Output:**

```
🔍 SkanUJkod Analysis Report

=== Project Health Summary ===
Overall Health Score: 85.2

Statement Coverage: 78.5% (157/200 statements covered)
Branch Coverage: 65.2% (15/23 branches covered)
Avg Complexity: 4.2 (Max: 15 - complexFlow)

Recommendations:
  1. Consider adding more unit tests to improve statement coverage
  2. Add tests for edge cases to improve branch coverage
  3. Review 2 functions with very high complexity (>15)
```

## 🔀 Control Flow Graph (CFG) Analysis

### Basic Usage

```bash
# Show CFG for all functions
./target/release/skan-uj-kod cfg go-code/example-go

# Show CFG with verbose output
./target/release/skan-uj-kod cfg go-code/example-go --verbose

# Generate DOT format for a specific function
./target/release/skan-uj-kod dot getDayName go-code/example-go

# Generate DOT with verbose output
./target/release/skan-uj-kod dot getDayName go-code/example-go --verbose

# Generate all functions as DOT files
./target/release/skan-uj-kod dot-all go-code/example-go

# Export CFG data to JSON
./target/release/skan-uj-kod cfg go-code/example-go --json --output=cfg-data.json
```

### Image Generation Examples

```bash
# Generate PNG for specific function
./target/release/skan-uj-kod dot getDayName go-code/example-go --image=png --output=getDayName.png

# Generate SVG for documentation
./target/release/skan-uj-kod dot complexFlow go-code/example-go --image=svg --output=complexFlow.svg

# Generate PDF for printing
./target/release/skan-uj-kod dot calculateGrade go-code/example-go --image=pdf --output=calculateGrade.pdf

# Generate all functions as SVG images
./target/release/skan-uj-kod dot-all go-code/example-go --image=svg

# Generate all functions with custom output directory
mkdir graphs && ./target/release/skan-uj-kod dot-all go-code/example-go --image=svg --output=graphs/
```

### Testing CFG Module

```bash
# Test different functions from go-code/example-go
./target/release/skan-uj-kod cfg go-code/example-go --verbose
./target/release/skan-uj-kod dot findMax go-code/example-go --verbose
./target/release/skan-uj-kod dot minMax go-code/example-go --image=svg --output=minMax.svg
./target/release/skan-uj-kod dot sumArray go-code/example-go --image=png --output=sumArray.png

# Export and view CFG data
./target/release/skan-uj-kod cfg go-code/example-go --json --output=cfg-test.json
cat cfg-test.json
```

### CFG HTML Gallery

Generate beautiful interactive HTML galleries with all function graphs:

```bash
# Generate HTML gallery with SVG images
./target/release/skan-uj-kod cfg-gallery go-code/example-go --images

# Generate HTML gallery with custom output path
./target/release/skan-uj-kod cfg-gallery go-code/example-go --images --output=reports/cfg_gallery.html

# Generate gallery with DOT files only (no images)
./target/release/skan-uj-kod cfg-gallery go-code/example-go --output=cfg_gallery.html

# Open the generated gallery in browser (macOS)
open reports/cfg_gallery.html
```

**CFG Gallery Features:**

- 🎨 **Interactive Design**: Modern responsive layout with gradient backgrounds
- 📊 **Function Cards**: Each function displayed with complexity metrics
- 🔍 **Complexity Filtering**: Filter functions by low/medium/high complexity
- 🖼️ **Image Modal**: Click on graphs to view them full-screen
- 📈 **Statistics Dashboard**: Overall project complexity overview
- 🎯 **Color-coded Complexity**: Visual indicators for complexity levels

**Gallery Structure:**

- `cfg_gallery.html` - Main interactive gallery page
- `graphs/` - Directory containing SVG images for each function
- Automatic sorting by complexity (most complex functions first)

## Quick Test Script

Create a test script to run all modules:

```bash
#!/bin/bash
# test-all-modules.sh

echo "🔍 Testing SkanUJkod modules with go-code/example-go"
echo "=================================================="

# Build first
cargo build --release

echo "📊 Testing Statement Coverage..."
./target/release/skan-uj-kod stmt-cov go-code/example-go --threshold=65 --verbose

echo "🌿 Testing Branch Coverage..."
./target/release/skan-uj-kod branch-cov go-code/example-go --threshold=20 --verbose

echo "🔄 Testing Complexity Analysis..."
./target/release/skan-uj-kod complexity go-code/example-go --threshold=8 --verbose

echo "🔀 Testing CFG Generation..."
./target/release/skan-uj-kod cfg go-code/example-go --verbose
./target/release/skan-uj-kod dot getDayName go-code/example-go --image=svg --output=test-getDayName.svg

echo "🎨 Testing CFG HTML Gallery..."
./target/release/skan-uj-kod cfg-gallery go-code/example-go --images --output=reports/test-gallery.html

echo "🎯 Testing Full Analysis..."
./target/release/skan-uj-kod full go-code/example-go --html --output=reports/test-full.html

echo "✅ All tests completed!"
echo "📄 Check reports/test-full.html for comprehensive analysis"
echo "🎨 Check reports/test-gallery.html for CFG gallery"
echo "🌐 Open gallery: open reports/test-gallery.html"
```
