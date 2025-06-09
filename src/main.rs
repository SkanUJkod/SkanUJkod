use clap::Parser;
use std::process;

mod cli;
mod plugin_manager;
mod commands;

use cli::{Cli, Commands};
use plugin_manager::PluginManager;

fn main() {
    let cli = Cli::parse();

    // Initialize plugin manager
    let plugins_dir = std::env::var("PLUGINS_DIR")
        .unwrap_or_else(|_| "./target/debug".to_string());
    
    let mut plugin_manager = match PluginManager::new(&plugins_dir) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Error: Failed to initialize plugin manager: {}", e);
            eprintln!("Make sure PLUGINS_DIR environment variable is set or plugins exist in ./target/debug");
            process::exit(1);
        }
    };

    // Execute the appropriate command
    let result = match &cli.command {
        Commands::Cfg { project_path, output, format, include_tests, exclude } => {
            commands::cfg::run_cfg_analysis(
                &mut plugin_manager,
                project_path,
                output.as_deref(),
                format,
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

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
