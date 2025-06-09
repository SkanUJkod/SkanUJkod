use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
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
    pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  ")
        );
        pb.set_message(message.to_string());
        pb
    }

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

    /// Print statistics
    pub fn print_stats(stats: &[(&str, String)]) {
        println!();
        Self::print_subsection("Statistics");
        for (key, value) in stats {
            Self::print_kv(key, value);
        }
    }

    /// Print table header
    pub fn print_table_header(headers: &[&str]) {
        let separator = "─".repeat(80).bright_black();
        println!("  {}", separator);
        
        let header_str = headers.iter()
            .map(|h| format!("{:20}", h.bright_cyan().bold()))
            .collect::<Vec<_>>()
            .join("");
        println!("  {}", header_str);
        
        println!("  {}", separator);
    }

    /// Print table row
    pub fn print_table_row(cells: &[&str]) {
        let row_str = cells.iter()
            .map(|c| format!("{:20}", c.white()))
            .collect::<Vec<_>>()
            .join("");
        println!("  {}", row_str);
    }

    /// Print completion message
    pub fn print_completion(duration: Duration) {
        println!();
        Self::print_success(&format!("Analysis completed in {:.2}s", duration.as_secs_f64()));
        println!();
    }

    /// Print CFG specific information
    pub fn print_cfg_info(total_functions: usize, filtered_functions: usize, filter: Option<&str>) {
        Self::print_subsection("CFG Analysis Details");
        
        if let Some(f) = filter {
            Self::print_kv("Filter Applied", f);
            Self::print_kv("Functions Found", &total_functions.to_string());
            Self::print_kv("Functions Matching", &filtered_functions.to_string());
            
            if filtered_functions == 0 {
                Self::print_warning(&format!("No functions found matching filter '{}'", f));
            } else if filtered_functions == 1 {
                Self::print_success("Found matching function");
            } else {
                Self::print_info(&format!("Found {} matching functions", filtered_functions));
            }
        } else {
            Self::print_kv("Total Functions", &total_functions.to_string());
            Self::print_info("No filter applied - analyzing all functions");
        }
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

/// Helper trait for colorizing analysis results
pub trait StatusColorize {
    fn colorize_status(&self) -> String;
}

impl StatusColorize for str {
    fn colorize_status(&self) -> String {
        use colored::Colorize;
        match self {
            s if s.contains("Success") || s.contains("✓") => s.green().to_string(),
            s if s.contains("Warning") || s.contains("⚠") => s.yellow().to_string(),
            s if s.contains("Error") || s.contains("Failed") || s.contains("✗") => s.red().to_string(),
            s => s.white().to_string(),
        }
    }
}
