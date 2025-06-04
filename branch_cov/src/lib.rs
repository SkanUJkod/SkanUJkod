pub mod core;
pub mod helpers;
pub mod instrumentation;

pub use core::{
    analyze_branch_coverage, analyze_branch_coverage_with_options, BranchCoverageOptions,
    BranchInfo, FunctionBranchCoverage, ProjectBranchCoverage, UncoveredBranch,
};
