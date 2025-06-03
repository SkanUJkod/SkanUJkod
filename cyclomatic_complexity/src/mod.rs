// Cyclomatic Complexity Module Entry Point
//
// This module provides a clean API for analyzing cyclomatic complexity of Go code.

use anyhow::Result;
use std::path::Path;

mod analyzer;
mod helpers;

pub use analyzer::{
    analyze_cyclomatic_complexity,
    analyze_cyclomatic_complexity_with_options,
    FunctionComplexity,
    ProjectComplexity,
    ComplexityOptions,
    DecisionPoint,
};

pub use helpers::ComplexityLevel;

/// Analyze a Go project for cyclomatic complexity
///
/// # Arguments
///
/// * `project_path` - Path to the Go project to analyze
///
/// # Returns
///
/// * `Result<ProjectComplexity>` - Analysis results or error
pub fn analyze_project(project_path: &Path) -> Result<ProjectComplexity> {
    analyze_cyclomatic_complexity(project_path)
}

/// Analyze a Go project with custom options
///
/// # Arguments
///
/// * `project_path` - Path to the Go project to analyze
/// * `options` - Custom analysis options
///
/// # Returns
///
/// * `Result<ProjectComplexity>` - Analysis results or error
pub fn analyze_project_with_options(
    project_path: &Path,
    options: &ComplexityOptions,
) -> Result<ProjectComplexity> {
    analyze_cyclomatic_complexity_with_options(project_path, options)
}