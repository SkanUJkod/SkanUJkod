// Cyclomatic Complexity Module
// Provides an API for analyzing code complexity of Go programs

pub mod analyzer;
pub mod helpers;

pub use crate::analyzer::{
    analyze_cyclomatic_complexity,
    analyze_cyclomatic_complexity_with_options,
    FunctionComplexity,
    ProjectComplexity,
    ComplexityOptions,
    DecisionPoint,
};

pub use crate::helpers::ComplexityLevel;