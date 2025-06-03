use crate::analyzer::{analyze_cyclomatic_complexity_with_options, ComplexityOptions};
use crate::ProjectComplexity;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Complexity level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    Low,      // 1-5
    Moderate, // 6-10
    High,     // 11-20
    VeryHigh, // >20
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityLevel::Low => write!(f, "Low"),
            ComplexityLevel::Moderate => write!(f, "Moderate"),
            ComplexityLevel::High => write!(f, "High"),
            ComplexityLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}

/// Configuration for complexity analysis
#[derive(Debug, Clone)]
pub struct ComplexityConfig {
    pub max_complexity: usize,
    pub max_cognitive_complexity: usize,
    pub exclude_functions: HashSet<String>,
    pub exclude_patterns: Vec<String>,
    pub include_tests: bool,
    pub fail_on_high_complexity: bool,
    pub verbose: bool,
    pub output_format: OutputFormat,
    pub thresholds: ComplexityThresholds,
}

#[derive(Debug, Clone)]
pub struct ComplexityThresholds {
    pub low_upper: usize,
    pub moderate_upper: usize,
    pub high_upper: usize,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            low_upper: 5,
            moderate_upper: 10,
            high_upper: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Plain,
    Json,
    Html,
    Csv,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            max_complexity: 10,
            max_cognitive_complexity: 15,
            exclude_functions: HashSet::from(["init".to_string(), "main".to_string()]),
            exclude_patterns: vec!["*_test.go".to_string(), "vendor/*".to_string()],
            include_tests: false,
            fail_on_high_complexity: true,
            verbose: false,
            output_format: OutputFormat::Plain,
            thresholds: ComplexityThresholds::default(),
        }
    }
}

/// High-level analyzer for cyclomatic complexity
pub struct ComplexityAnalyzer {
    config: ComplexityConfig,
}

impl ComplexityAnalyzer {
    pub fn new(config: ComplexityConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(ComplexityConfig::default())
    }

    pub fn analyze(&self, project_path: &Path) -> Result<ProjectComplexity> {
        if self.config.verbose {
            println!("Analyzing complexity at: {}", project_path.display());
        }

        let options = ComplexityOptions {
            verbose: self.config.verbose,
            include_cognitive: true,
            max_allowed_complexity: self.config.max_complexity,
            fail_on_high_complexity: self.config.fail_on_high_complexity,
        };

        let mut complexity = analyze_cyclomatic_complexity_with_options(project_path, &options)?;

        // Filter out excluded functions
        complexity.functions.retain(|name, _| {
            !self.config.exclude_functions.contains(name)
        });

        // Check thresholds
        let violations = self.check_violations(&complexity);
        if !violations.is_empty() && self.config.fail_on_high_complexity {
            let msg = violations.join("\n");
            anyhow::bail!("Complexity violations found:\n{}", msg);
        }

        Ok(complexity)
    }

    fn check_violations(&self, complexity: &ProjectComplexity) -> Vec<String> {
        let mut violations = Vec::new();

        for (name, func) in &complexity.functions {
            if func.cyclomatic_complexity > self.config.max_complexity {
                violations.push(format!(
                    "Function '{}' has cyclomatic complexity {} (max allowed: {})",
                    name, func.cyclomatic_complexity, self.config.max_complexity
                ));
            }

            if func.cognitive_complexity > self.config.max_cognitive_complexity {
                violations.push(format!(
                    "Function '{}' has cognitive complexity {} (max allowed: {})",
                    name, func.cognitive_complexity, self.config.max_cognitive_complexity
                ));
            }
        }

        violations
    }

    pub fn generate_badge(&self, complexity: &ProjectComplexity) -> String {
        let avg = complexity.average_complexity;
        let (color, status) = if avg <= 5.0 {
            ("brightgreen", "excellent")
        } else if avg <= 10.0 {
            ("green", "good")
        } else if avg <= 15.0 {
            ("yellow", "moderate")
        } else if avg <= 20.0 {
            ("orange", "high")
        } else {
            ("red", "very high")
        };

        format!(
            "![Complexity](https://img.shields.io/badge/complexity-{:.1}_{}-{}.svg)",
            avg, status, color
        )
    }
}

pub mod export {
    use super::*;
    use crate::{};

    pub fn to_json(complexity: &ProjectComplexity) -> Result<String> {
        serde_json::to_string_pretty(complexity).context("Failed to serialize complexity data")
    }

    pub fn to_sonarqube_format(complexity: &ProjectComplexity) -> Result<String> {
        #[derive(Serialize)]
        struct SonarQubeIssue {
            #[serde(rename = "engineId")]
            engine_id: String,
            #[serde(rename = "ruleId")]
            rule_id: String,
            #[serde(rename = "primaryLocation")]
            primary_location: SonarQubeLocation,
            #[serde(rename = "type")]
            issue_type: String,
            severity: String,
            effort: String,
        }

        #[derive(Serialize)]
        struct SonarQubeLocation {
            message: String,
            #[serde(rename = "filePath")]
            file_path: String,
            #[serde(rename = "textRange")]
            text_range: SonarQubeTextRange,
        }

        #[derive(Serialize)]
        struct SonarQubeTextRange {
            #[serde(rename = "startLine")]
            start_line: usize,
            #[serde(rename = "endLine")]
            end_line: usize,
        }

        let mut issues = Vec::new();

        for (func_name, func_complexity) in &complexity.functions {
            if func_complexity.cyclomatic_complexity > 10 {
                let severity = match func_complexity.complexity_level {
                    ComplexityLevel::High => "MAJOR",
                    ComplexityLevel::VeryHigh => "CRITICAL",
                    _ => "MINOR",
                };

                issues.push(SonarQubeIssue {
                    engine_id: "cyclomatic-complexity".to_string(),
                    rule_id: "high-complexity".to_string(),
                    primary_location: SonarQubeLocation {
                        message: format!(
                            "Function '{}' has cyclomatic complexity of {}",
                            func_name, func_complexity.cyclomatic_complexity
                        ),
                        file_path: "unknown".to_string(), // Would need file path tracking
                        text_range: SonarQubeTextRange {
                            start_line: 1,
                            end_line: 1,
                        },
                    },
                    issue_type: "CODE_SMELL".to_string(),
                    severity: severity.to_string(),
                    effort: format!("{}min", func_complexity.cyclomatic_complexity * 5),
                });
            }
        }

        serde_json::to_string_pretty(&issues).context("Failed to generate SonarQube format")
    }

    pub fn ci_export(complexity: &ProjectComplexity, threshold: f64) -> i32 {
        println!(
            "::set-output name=average_complexity::{:.1}",
            complexity.average_complexity
        );
        println!(
            "::set-output name=max_complexity::{}",
            complexity.max_complexity
        );
        println!(
            "::set-output name=total_functions::{}",
            complexity.total_functions
        );

        if complexity.average_complexity > threshold {
            eprintln!(
                "Average complexity {:.1} exceeds threshold {:.1}",
                complexity.average_complexity, threshold
            );
            1
        } else {
            0
        }
    }

    pub fn markdown_report(complexity: &ProjectComplexity) -> String {
        let mut md = String::new();
        
        md.push_str("# Cyclomatic Complexity Report\n\n");
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Total Functions**: {}\n", complexity.total_functions));
        md.push_str(&format!("- **Average Complexity**: {:.2}\n", complexity.average_complexity));
        md.push_str(&format!("- **Maximum Complexity**: {} ({})\n", 
            complexity.max_complexity, complexity.max_complexity_function));
        md.push_str("\n## Complexity Distribution\n\n");
        md.push_str("| Level | Count | Percentage |\n");
        md.push_str("|-------|-------|------------|\n");
        
        let total = complexity.total_functions as f64;
        for (level, count) in &complexity.complexity_distribution {
            let percentage = (*count as f64 / total) * 100.0;
            md.push_str(&format!("| {} | {} | {:.1}% |\n", level, count, percentage));
        }
        
        md.push_str("\n## Most Complex Functions\n\n");
        md.push_str("| Function | Cyclomatic | Cognitive | LOC | Level |\n");
        md.push_str("|----------|------------|-----------|-----|-------|\n");
        
        let mut sorted: Vec<_> = complexity.functions.iter().collect();
        sorted.sort_by(|a, b| b.1.cyclomatic_complexity.cmp(&a.1.cyclomatic_complexity));
        
        for (name, func) in sorted.iter().take(10) {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                name, 
                func.cyclomatic_complexity,
                func.cognitive_complexity,
                func.lines_of_code,
                func.complexity_level
            ));
        }
        
        md
    }
}

pub mod go_utils {
    use super::*;

    pub fn check_go_installation() -> Result<String> {
        let output = Command::new("go")
            .arg("version")
            .output()
            .context("Failed to run 'go version'. Is Go installed?")?;

        if !output.status.success() {
            anyhow::bail!("Go is not properly installed");
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    }

    pub fn find_go_files(project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut go_files = Vec::new();

        fn visit_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if !name_str.starts_with('.')
                            && name_str != "vendor"
                            && name_str != "testdata"
                        {
                            visit_dir(&path, files)?;
                        }
                    }
                } else if path.extension().and_then(|s| s.to_str()) == Some("go") {
                    files.push(path);
                }
            }
            Ok(())
        }

        visit_dir(project_path, &mut go_files)?;
        Ok(go_files)
    }

    pub fn is_test_file(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with("_test.go"))
            .unwrap_or(false)
    }
}

pub mod refactoring {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct RefactoringHint {
        pub function: String,
        pub severity: RefactoringSeverity,
        pub hints: Vec<String>,
    }

    #[derive(Debug, Serialize)]
    pub enum RefactoringSeverity {
        Low,
        Medium,
        High,
        Critical,
    }

    pub fn generate_refactoring_hints(complexity: &ProjectComplexity) -> Vec<RefactoringHint> {
        let mut hints = Vec::new();

        for (name, func) in &complexity.functions {
            if func.cyclomatic_complexity <= 10 {
                continue;
            }

            let severity = match func.cyclomatic_complexity {
                11..=15 => RefactoringSeverity::Low,
                16..=20 => RefactoringSeverity::Medium,
                21..=30 => RefactoringSeverity::High,
                _ => RefactoringSeverity::Critical,
            };

            let mut suggestions = Vec::new();

            // Basic suggestions based on complexity level
            if func.cyclomatic_complexity > 10 {
                suggestions.push("Consider breaking this function into smaller functions".to_string());
            }

            if func.nesting_depth_max > 3 {
                suggestions.push("Reduce nesting by using early returns or guard clauses".to_string());
            }

            // Analyze decision points
            let switch_count = func.decision_points.iter()
                .filter(|dp| dp.stmt_type == "switch" || dp.stmt_type == "typeswitch")
                .count();
            
            if switch_count > 1 {
                suggestions.push("Consider using polymorphism or strategy pattern instead of multiple switches".to_string());
            }

            let if_count = func.decision_points.iter()
                .filter(|dp| dp.stmt_type == "if")
                .count();
            
            if if_count > 5 {
                suggestions.push("Too many if statements - consider using a lookup table or map".to_string());
            }

            // Cognitive complexity specific hints
            if func.cognitive_complexity > func.cyclomatic_complexity * 2 {
                suggestions.push("High cognitive complexity indicates confusing logic - simplify control flow".to_string());
            }

            hints.push(RefactoringHint {
                function: name.clone(),
                severity,
                hints: suggestions,
            });
        }

        hints.sort_by_key(|h| match h.severity {
            RefactoringSeverity::Critical => 0,
            RefactoringSeverity::High => 1,
            RefactoringSeverity::Medium => 2,
            RefactoringSeverity::Low => 3,
        });

        hints
    }
}