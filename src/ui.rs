use colored::Colorize;
use std::time::Duration;

/// Beautiful console output utilities for SkanUJkod
pub struct UI;

impl UI {
    /// Print a welcome banner with project info
    pub fn print_banner() {
        println!("{}", "╔══════════════════════════════════════════════════════════════════════╗".bright_blue());
        println!("{}", "║                              SkanUJkod                              ║".bright_blue());
        println!("{}", "║                   Static Analysis Framework for Go                  ║".bright_blue());
        println!("{}", "║                            Version 0.1.0                           ║".bright_blue());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════╝".bright_blue());
        println!();
    }

    /// Print section header
    pub fn print_section(title: &str) {
        println!("{} {}", "▶".bright_green().bold(), title.bright_white().bold());
        println!("{}", "─".repeat(50).bright_black());
    }

    /// Print subsection header
    pub fn print_subsection(title: &str) {
        println!("  {} {}", "•".bright_cyan(), title.white());
    }

    /// Print success message
    pub fn print_success(message: &str) {
        println!("  {} {}", "✓".bright_green().bold(), message.green());
    }

    /// Print warning message
    pub fn print_warning(message: &str) {
        println!("  {} {}", "⚠".bright_yellow().bold(), message.yellow());
    }

    /// Print error message
    pub fn print_error(message: &str) {
        eprintln!("  {} {}", "✗".bright_red().bold(), message.red());
    }

    /// Print info message
    pub fn print_info(message: &str) {
        println!("  {} {}", "ℹ".bright_blue().bold(), message.white());
    }

    /// Print key-value pair
    pub fn print_kv(key: &str, value: &str) {
        println!("  {} {}", 
                format!("{}:", key).bright_cyan(), 
                value.white());
    }

    /// Print analysis result summary
    pub fn print_analysis_summary(analysis_type: &str, project_path: &str, function_filter: Option<&str>) {
        Self::print_section(&format!("{} Analysis", analysis_type));
        Self::print_kv("Project", project_path);
        
        if let Some(filter) = function_filter {
            Self::print_kv("Function Filter", filter);
        } else {
            Self::print_kv("Scope", "All functions");
        }
        println!();
    }

    /// Create and return a progress bar
    /// Show plugin execution progress
    pub fn print_plugin_execution(plugin_name: &str, function_name: &str) {
        println!("  {} Executing: {}", 
                "⚙".bright_yellow(), 
                format!("{}::{}", plugin_name, function_name).bright_white());
    }

    /// Print results header
    pub fn print_results_header(format: &str) {
        println!();
        Self::print_section(&format!("Analysis Results ({})", format));
    }

    /// Print file output info
    pub fn print_file_output(path: &str) {
        Self::print_success(&format!("Results written to: {}", path.bright_white()));
    }

    /// Print completion message
    pub fn print_completion(duration: Duration) {
        println!();
        Self::print_success(&format!("Analysis completed in {:.2}s", duration.as_secs_f64()));
        println!();
    }

    /// Print available plugins in a nice format
    pub fn print_plugins_header() {
        Self::print_section("Available Plugins");
    }

    /// Print plugin information
    pub fn print_plugin_info(plugin_name: &str, function_count: usize) {
        Self::print_subsection(&format!("{} ({} functions)", plugin_name, function_count));
    }

    /// Print plugin function details
    pub fn print_function_details(function_name: &str, dependencies: &[String], params: &[String]) {
        println!("    {} {}", "⚡".bright_blue(), function_name.bright_white());
        
        if !dependencies.is_empty() {
            println!("      {} {}", "Dependencies:".bright_black(), dependencies.join(", ").bright_black());
        }
        
        if !params.is_empty() {
            println!("      {} {}", "Parameters:".bright_black(), params.join(", ").bright_black());
        }
        println!();
    }

    /// Print summary statistics for all analyses
    pub fn print_all_analysis_summary(results: &[(&str, String)]) {
        println!();
        Self::print_section("Analysis Summary");
        
        let mut success_count = 0;
        let mut total_count = 0;
        
        for (analysis, status) in results {
            total_count += 1;
            if status.contains("Success") {
                success_count += 1;
                Self::print_success(&format!("{}: {}", analysis, status));
            } else {
                Self::print_error(&format!("{}: {}", analysis, status));
            }
        }
        
        println!();
        Self::print_kv("Total Analyses", &total_count.to_string());
        Self::print_kv("Successful", &success_count.to_string());
        Self::print_kv("Failed", &(total_count - success_count).to_string());
        
        if success_count == total_count {
            Self::print_success("All analyses completed successfully! 🎉");
        } else if success_count > 0 {
            Self::print_warning("Some analyses completed with errors");
        } else {
            Self::print_error("All analyses failed");
        }
        
        println!();
    }
}
