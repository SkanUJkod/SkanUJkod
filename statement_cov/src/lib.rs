// Statement Coverage Module
// Provides an API for analyzing statement coverage in Go programs

pub mod core;
pub mod helpers;
pub mod instrumentation;

pub use core::{
    analyze_statement_coverage,
    analyze_statement_coverage_with_options,
    ProjectCoverage,
    FunctionCoverage,
    CoverageOptions,
    UncoveredLine,
};