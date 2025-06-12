use clap::Parser;
use std::process;
use std::time::Instant;

mod cli;
mod plugin_manager;
mod commands;
mod ui;

use cli::{Cli, Commands};
use plugin_manager::PluginManager;
use ui::UI;

fn main() {
    let start_time = Instant::now();
    let cli = Cli::parse();

    // Show banner
    UI::print_banner();

    // Initialize plugin manager
    let plugins_dir = std::env::var("PLUGINS_DIR")
        .unwrap_or_else(|_| "./target/debug".to_string());
    
    UI::print_info(&format!("Loading plugins from: {}", plugins_dir));
    
    let mut plugin_manager = match PluginManager::new(&plugins_dir) {
        Ok(manager) => {
            UI::print_success("Plugin manager initialized");
            manager
        },
        Err(e) => {
            UI::print_error(&format!("Failed to initialize plugin manager: {}", e));
            UI::print_error("Make sure PLUGINS_DIR environment variable is set or plugins exist in ./target/debug");
            process::exit(1);
        }
    };

    // Execute the appropriate command
    let result = match &cli.command {
        Commands::Cfg { project_path, output, format, function, include_tests, exclude } => {
            commands::cfg::run_cfg_analysis(
                &mut plugin_manager,
                project_path,
                output.as_deref(),
                format,
                function.as_deref(),
                *include_tests,
                exclude,
            )
        }
        Commands::BranchCov { project_path, output, threshold, include_tests, exclude } => {
            commands::branch_coverage::run_branch_coverage_analysis(
                &mut plugin_manager,
                project_path,
                output.as_deref(),
                *threshold,
                *include_tests,
                exclude,
            )
        }
        Commands::StatementCov { project_path, output, threshold, include_tests, exclude } => {
            commands::statement_coverage::run_statement_coverage_analysis(
                &mut plugin_manager,
                project_path,
                output.as_deref(),
                *threshold,
                *include_tests,
                exclude,
            )
        }
        Commands::Complexity { project_path, output, max_complexity, include_tests, exclude } => {
            commands::cyclomatic_complexity::run_complexity_analysis(
                &mut plugin_manager,
                project_path,
                output.as_deref(),
                *max_complexity,
                *include_tests,
                exclude,
            )
        }
        Commands::All { project_path, output_dir, include_tests, exclude } => {
            commands::all::run_all_analyses(
                &mut plugin_manager,
                project_path,
                output_dir.as_deref(),
                *include_tests,
                exclude,
            )
        }
        Commands::ListPlugins => {
            commands::list::list_available_plugins(&plugin_manager)
        }
    };

    // Handle result and show completion
    match result {
        Ok(_) => {
            let duration = start_time.elapsed();
            UI::print_completion(duration);
        },
        Err(e) => {
            UI::print_error(&format!("Analysis failed: {}", e));
            process::exit(1);
        }
    }
}
