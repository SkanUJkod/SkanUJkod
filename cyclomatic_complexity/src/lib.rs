pub mod analyzer;
pub mod helpers;

pub use analyzer::{
    analyze_cyclomatic_complexity,
    analyze_cyclomatic_complexity_with_options,
    print_complexity_report,
    export_complexity_report,
    ProjectComplexity,
    FunctionComplexity,
    ComplexityOptions,
    ExportFormat,
    DecisionPoint,
};

pub use helpers::{
    ComplexityAnalyzer,
    ComplexityConfig,
    ComplexityLevel,
    ComplexityThresholds,
    export::to_json as export_to_json,
    export::markdown_report,
    export::to_sonarqube_format,
    refactoring::generate_refactoring_hints,
};