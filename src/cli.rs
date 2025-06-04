use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use std::process::Command;

use branch_cov::ProjectBranchCoverage;
use cyclomatic_complexity::ProjectComplexity;
use statement_cov::ProjectCoverage as StmtCoverage;

pub fn check_graphviz_installation() -> Result<bool> {
    let output = Command::new("dot").arg("-V").output();

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

pub fn generate_image_from_dot(
    dot_content: &str,
    output_path: &Path,
    format: crate::ImageFormat,
) -> Result<()> {
    if !check_graphviz_installation()? {
        anyhow::bail!(
            "Graphviz is not installed. Please install 'graphviz' package to generate images."
        );
    }

    let temp_dot_path = output_path.with_extension("dot");
    fs::write(&temp_dot_path, dot_content)?;

    let status = Command::new("dot")
        .arg("-T")
        .arg(format.extension())
        .arg("-o")
        .arg(output_path)
        .arg(&temp_dot_path)
        .status()
        .context("Failed to run 'dot' command")?;

    if !status.success() {
        anyhow::bail!("The 'dot' command failed to execute");
    }

    if temp_dot_path != output_path {
        let _ = fs::remove_file(temp_dot_path);
    }

    Ok(())
}

/// Print usage information (kept for potential future use)
#[allow(dead_code)]
pub fn print_usage() {
    eprintln!(
        "Usage:\n  \
        skan-uj-kod <command> [options] <path>\n\n\
        Commands:\n  \
        cfg                   - Display CFG in debug format\n  \
        dot <FuncName>        - Print DOT format for selected function\n  \
        dot-all               - Generate DOT files for all functions\n  \
        stmt-cov              - Display statement coverage\n  \
        branch-cov            - Display branch coverage analysis\n  \
        complexity            - Display cyclomatic complexity analysis\n  \
        help                  - Show this help message\n\n\
        Options:\n  \
        --json                - Output in JSON format\n  \
        --output=<file>       - Write output to file\n  \
        --image=<format>      - Generate image (png, svg, pdf) from DOT\n  \
        --verbose             - Show detailed information"
    );
}

pub fn print_complexity_report(complexity: &ProjectComplexity, detailed: bool) {
    println!(
        "\n{}\n",
        "=== Cyclomatic Complexity Report ===".bright_blue().bold()
    );

    println!(
        "{}: {}",
        "Files analyzed".cyan(),
        complexity.files_analyzed.to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Functions found".cyan(),
        complexity.total_functions.to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Average complexity".cyan(),
        format!("{:.2}", complexity.average_complexity)
            .white()
            .bold()
    );
    println!(
        "{}: {} ({})\n",
        "Maximum complexity".cyan(),
        complexity.max_complexity.to_string().red().bold(),
        complexity.max_complexity_function.yellow()
    );

    println!("{}:", "Complexity Distribution".green().bold());
    println!(
        "  {}: {} functions",
        "Low (1-5)".green(),
        complexity.complexity_distribution.get("low").unwrap_or(&0)
    );
    println!(
        "  {}: {} functions",
        "Moderate (6-10)".yellow(),
        complexity
            .complexity_distribution
            .get("moderate")
            .unwrap_or(&0)
    );
    println!(
        "  {}: {} functions",
        "High (11-20)".red(),
        complexity.complexity_distribution.get("high").unwrap_or(&0)
    );
    println!(
        "  {}: {} functions\n",
        "Very High (>20)".bright_red().bold(),
        complexity
            .complexity_distribution
            .get("very_high")
            .unwrap_or(&0)
    );

    if detailed {
        println!("{}:", "Function Details".blue().bold());
        let mut sorted_functions: Vec<_> = complexity.functions.iter().collect();
        sorted_functions.sort_by(|a, b| b.1.cyclomatic_complexity.cmp(&a.1.cyclomatic_complexity));

        for (name, func) in sorted_functions.iter().take(10) {
            let complexity_color = match func.cyclomatic_complexity {
                1..=5 => func.cyclomatic_complexity.to_string().green(),
                6..=10 => func.cyclomatic_complexity.to_string().yellow(),
                11..=20 => func.cyclomatic_complexity.to_string().red(),
                _ => func.cyclomatic_complexity.to_string().bright_red().bold(),
            };

            println!(
                "{} - CC: {}, Cognitive: {}, LOC: {}",
                name.white().bold(),
                complexity_color,
                func.cognitive_complexity.to_string().cyan(),
                func.lines_of_code.to_string().blue()
            );
        }
    }
}

pub fn print_coverage_report(coverage: &StmtCoverage, detailed: bool) {
    println!(
        "\n{}\n",
        "=== Statement Coverage Report ===".bright_blue().bold()
    );

    println!(
        "{}: {}",
        "Files analyzed".cyan(),
        coverage.files_analyzed.to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Functions found".cyan(),
        coverage.functions.len().to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Total statements".cyan(),
        coverage.total_statements.to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Covered statements".cyan(),
        coverage.covered_statements.to_string().green().bold()
    );

    let coverage_color = if coverage.overall_coverage >= 90.0 {
        format!("{:.1}%", coverage.overall_coverage).green().bold()
    } else if coverage.overall_coverage >= 70.0 {
        format!("{:.1}%", coverage.overall_coverage).yellow().bold()
    } else {
        format!("{:.1}%", coverage.overall_coverage).red().bold()
    };
    println!("{}: {}\n", "Overall coverage".cyan(), coverage_color);

    if detailed {
        println!("{}:", "Function Details".blue().bold());
        let mut functions: Vec<_> = coverage.functions.iter().collect();
        functions.sort_by(|a, b| {
            a.1.coverage_percentage
                .partial_cmp(&b.1.coverage_percentage)
                .unwrap()
        });

        for (func_name, func_coverage) in functions {
            let func_coverage_color = if func_coverage.coverage_percentage >= 90.0 {
                format!("{:.1}%", func_coverage.coverage_percentage).green()
            } else if func_coverage.coverage_percentage >= 70.0 {
                format!("{:.1}%", func_coverage.coverage_percentage).yellow()
            } else {
                format!("{:.1}%", func_coverage.coverage_percentage).red()
            };

            println!("{}", func_name.white().bold());
            println!(
                "  Coverage: {} ({}/{})",
                func_coverage_color,
                func_coverage.covered_statements.to_string().green(),
                func_coverage.total_statements.to_string().blue()
            );

            if !func_coverage.uncovered_lines.is_empty() && func_coverage.uncovered_lines.len() <= 5
            {
                println!(
                    "  {}: {:?}",
                    "Uncovered lines".red(),
                    func_coverage.uncovered_lines
                );
            } else if !func_coverage.uncovered_lines.is_empty() {
                println!(
                    "  {}: {} (showing first 5: {:?}...)",
                    "Uncovered lines".red(),
                    func_coverage.uncovered_lines.len(),
                    &func_coverage.uncovered_lines[..5.min(func_coverage.uncovered_lines.len())]
                );
            }
            println!();
        }
    }
}

pub fn print_branch_coverage_report(coverage: &ProjectBranchCoverage, detailed: bool) {
    println!(
        "\n{}\n",
        "=== Branch Coverage Report ===".bright_blue().bold()
    );

    println!(
        "{}: {}",
        "Files analyzed".cyan(),
        coverage.files_analyzed.len().to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Functions found".cyan(),
        coverage.functions.len().to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Total branches".cyan(),
        coverage.total_branches.to_string().white().bold()
    );
    println!(
        "{}: {}",
        "Covered branches".cyan(),
        coverage.covered_branches.to_string().green().bold()
    );

    let coverage_color = if coverage.overall_coverage_percentage >= 90.0 {
        format!("{:.1}%", coverage.overall_coverage_percentage)
            .green()
            .bold()
    } else if coverage.overall_coverage_percentage >= 70.0 {
        format!("{:.1}%", coverage.overall_coverage_percentage)
            .yellow()
            .bold()
    } else {
        format!("{:.1}%", coverage.overall_coverage_percentage)
            .red()
            .bold()
    };
    println!("{}: {}\n", "Overall coverage".cyan(), coverage_color);

    if detailed {
        println!("\n{}:", "Function Details".blue().bold());
        let mut functions: Vec<_> = coverage.functions.iter().collect();
        functions.sort_by(|a, b| {
            a.1.coverage_percentage
                .partial_cmp(&b.1.coverage_percentage)
                .unwrap()
        });

        for (func_name, func_coverage) in functions {
            let func_coverage_color = if func_coverage.coverage_percentage >= 90.0 {
                format!("{:.1}%", func_coverage.coverage_percentage).green()
            } else if func_coverage.coverage_percentage >= 70.0 {
                format!("{:.1}%", func_coverage.coverage_percentage).yellow()
            } else {
                format!("{:.1}%", func_coverage.coverage_percentage).red()
            };

            println!("{}", func_name.white().bold());
            println!(
                "  Coverage: {} ({}/{})",
                func_coverage_color,
                func_coverage.covered_branches.to_string().green(),
                func_coverage.total_branches.to_string().blue()
            );

            if !func_coverage.uncovered_branches.is_empty()
                && func_coverage.uncovered_branches.len() <= 5
            {
                println!("  {}:", "Uncovered branches".red());
                for uncovered in &func_coverage.uncovered_branches
                    [..5.min(func_coverage.uncovered_branches.len())]
                {
                    println!(
                        "    Line {}: {} ({})",
                        uncovered.line.to_string().yellow(),
                        uncovered.branch_type.red(),
                        uncovered.condition.dimmed()
                    );
                }
            } else if !func_coverage.uncovered_branches.is_empty() {
                println!(
                    "  {}: {} (showing first 5)",
                    "Uncovered branches".red(),
                    func_coverage.uncovered_branches.len()
                );
                for uncovered in &func_coverage.uncovered_branches[..5] {
                    println!(
                        "    Line {}: {} ({})",
                        uncovered.line.to_string().yellow(),
                        uncovered.branch_type.red(),
                        uncovered.condition.dimmed()
                    );
                }
                println!("    ...");
            }
            println!();
        }

        if !coverage.uncovered_branches.is_empty() {
            println!("{}:", "Overall Uncovered Branches Summary".red().bold());
            println!(
                "Total uncovered: {}",
                coverage.uncovered_branches.len().to_string().red().bold()
            );
        }
    }
}

pub fn export_to_json<T: serde::Serialize>(data: &T, output: Option<&Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;

    if let Some(path) = output {
        fs::write(path, json).context("Failed to write JSON to file")?;
        println!(
            "{}: {}",
            "Output written to".green(),
            path.display().to_string().white().bold()
        );
    } else {
        println!("{}", json);
    }

    Ok(())
}

/// Print success message with icon
pub fn print_success(msg: &str) {
    println!("{} {}", "✅".green(), msg.green());
}

/// Print warning message with icon  
#[allow(dead_code)]
pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠️".yellow(), msg.yellow());
}

/// Print error message with icon
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "❌".red(), msg.red());
}

/// Print info message with icon
pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ️".blue(), msg.blue());
}

/// Create a progress bar for long-running operations
pub fn create_progress_bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb.set_message(msg.to_string());
    pb
}

/// Create a spinner for indeterminate operations
pub fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb
}
