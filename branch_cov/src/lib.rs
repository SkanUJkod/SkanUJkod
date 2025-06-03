// Branch Coverage Module
// Provides an API for analyzing branch coverage in Go programs

pub mod core;
pub mod helpers;
pub mod instrumentation;

pub use core::{
    analyze_branch_coverage,
    analyze_branch_coverage_with_options,
    ProjectBranchCoverage,
    FunctionBranchCoverage,
    BranchCoverageOptions,
    UncoveredBranch,
    BranchInfo,
};