pub mod core;
pub mod helpers;
pub mod instrumentation;

pub use core::{
    analyze_statement_coverage, analyze_statement_coverage_with_options, CoverageOptions,
    FunctionCoverage, ProjectCoverage, UncoveredLine,
};
