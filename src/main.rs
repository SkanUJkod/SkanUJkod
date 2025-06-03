use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;

use cfg::{parse_project, build_cfgs_for_file, to_dot};
use statement_cov::{
    analyze_statement_coverage_with_options,
    CoverageOptions,
    ProjectCoverage as StmtCoverage
};
use cyclomatic_complexity::{
    analyze_cyclomatic_complexity_with_options,
    ComplexityOptions,
    ProjectComplexity
};
use branch_cov::{
    analyze_branch_coverage_with_options,
    BranchCoverageOptions,
    ProjectBranchCoverage
};

mod cli;
mod config;
mod reports;

use config::Config;

#[derive(Parser)]
#[command(name = "skan-uj-kod")]
#[command(about = "Static analysis tool for Go code")]
#[command(version = "1.0")]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output in JSON format
    #[arg(short, long, global = true)]
    json: bool,

    /// Output file path
    #[arg(short, long, global = true)]
    output: Option<PathBuf>,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Display Control Flow Graph in debug format
    Cfg {
        /// Path to Go project or file
        path: PathBuf,
    },
    /// Generate DOT format for a specific function
    Dot {
        /// Path to Go project or file
        path: PathBuf,
        /// Function name to generate DOT for
        function: String,
        /// Image format to generate
        #[arg(long, value_enum)]
        image: Option<ImageFormat>,
    },
    /// Generate DOT files for all functions
    DotAll {
        /// Path to Go project or file
        path: PathBuf,
        /// Image format to generate
        #[arg(long, value_enum)]
        image: Option<ImageFormat>,
    },
    /// Analyze statement coverage
    StmtCov {
        /// Path to Go project
        path: PathBuf,
        /// Coverage threshold for CI
        #[arg(long, default_value = "80")]
        threshold: f64,
        /// Test timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
    /// Analyze branch coverage
    BranchCov {
        /// Path to Go project
        path: PathBuf,
        /// Coverage threshold for CI
        #[arg(long, default_value = "80")]
        threshold: f64,
        /// Include test files in analysis
        #[arg(long)]
        include_tests: bool,
        /// Test timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
    /// Analyze cyclomatic complexity
    Complexity {
        /// Path to Go project
        path: PathBuf,
        /// Maximum complexity threshold
        #[arg(long, default_value = "10")]
        max_complexity: usize,
    },
    /// Run comprehensive analysis (all metrics)
    Full {
        /// Path to Go project
        path: PathBuf,
        /// Generate HTML report
        #[arg(long)]
        html: bool,
        /// Coverage threshold
        #[arg(long, default_value = "80")]
        threshold: f64,
    },
    /// Generate default configuration file
    InitConfig {
        /// Output path for config file
        #[arg(default_value = "skan-uj-kod.toml")]
        output: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
enum ImageFormat {
    Png,
    Svg,
    Pdf,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Pdf => "pdf",
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Load configuration
    let config = Config::load_or_default(args.config.as_deref());
    
    match args.command {
        Commands::InitConfig { output } => {
            config.save_to_file(&output)?;
            cli::print_success(&format!("Configuration file created: {}", output.display()));
            return Ok(());
        },
        _ => {}
    }
    
    let json_output = args.json;
    let verbose = args.verbose || config.output.verbose;
    
    // Extract project path without borrowing the whole command
    let project_path = match &args.command {
        Commands::Cfg { path } => path.clone(),
        Commands::Dot { path, .. } => path.clone(),
        Commands::DotAll { path, .. } => path.clone(),
        Commands::StmtCov { path, .. } => path.clone(),
        Commands::BranchCov { path, .. } => path.clone(),
        Commands::Complexity { path, .. } => path.clone(),
        Commands::Full { path, .. } => path.clone(),
        Commands::InitConfig { .. } => return Ok(()),
    };
    
    if !project_path.exists() {
        cli::print_error(&format!("Project path does not exist: {}", project_path.display()));
        std::process::exit(1);
    }

    match args.command {
        Commands::Cfg { .. } => {
            let spinner = cli::create_spinner("Parsing Go project...");
            let (fset, objs, files) = parse_project(&project_path)?;
            spinner.finish_with_message("✅ Project parsed successfully");
            
            let pb = cli::create_progress_bar(files.len() as u64, "Building CFGs");
            let mut cfgs_map = HashMap::new();
            
            for (i, pf) in files.iter().enumerate() {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
                pb.set_position(i as u64 + 1);
            }
            pb.finish_with_message("✅ CFGs built successfully");
            
            if json_output {
                let mut cfg_data = HashMap::new();
                for (name, cfg) in &cfgs_map {
                    cfg_data.insert(name.clone(), format!("{:#?}", cfg));
                }
                cli::export_to_json(&cfg_data, args.output.as_deref())?;
            } else {
                println!("\n{}", "=== Control Flow Graphs ===".bright_blue().bold());
                for (fname, graph) in &cfgs_map {
                    println!("--- func {} ---\n{:#?}", fname.white().bold(), graph);
                }
            }
        },
        
        Commands::Full { html, threshold, .. } => {
            cli::print_info("Running comprehensive analysis...");
            
            // Run all analyses
            let stmt_spinner = cli::create_spinner("Analyzing statement coverage...");
            let stmt_options = CoverageOptions {
                verbose,
                min_coverage_threshold: threshold,
                timeout_seconds: config.coverage.timeout_seconds,
                ..CoverageOptions::default()
            };
            let stmt_coverage = analyze_statement_coverage_with_options(&project_path, &stmt_options)?;
            stmt_spinner.finish_with_message("✅ Statement coverage analysis complete");
            
            let branch_spinner = cli::create_spinner("Analyzing branch coverage...");
            let branch_options = BranchCoverageOptions {
                verbose,
                min_coverage_threshold: threshold,
                timeout_seconds: config.coverage.timeout_seconds,
                include_test_files: config.coverage.include_test_files,
                exclude_patterns: config.coverage.exclude_patterns.clone(),
                ..BranchCoverageOptions::default()
            };
            let branch_coverage = analyze_branch_coverage_with_options(&project_path, &branch_options)?;
            branch_spinner.finish_with_message("✅ Branch coverage analysis complete");
            
            let complexity_spinner = cli::create_spinner("Analyzing complexity...");
            let complexity_options = ComplexityOptions {
                verbose,
                max_allowed_complexity: config.complexity.max_complexity,
                ..ComplexityOptions::default()
            };
            let complexity = analyze_cyclomatic_complexity_with_options(&project_path, &complexity_options)?;
            complexity_spinner.finish_with_message("✅ Complexity analysis complete");
            
            if html {
                let output_path = args.output.unwrap_or_else(|| {
                    config.output.output_dir.join("report.html")
                });
                
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                reports::generate_html_report(
                    Some(&stmt_coverage),
                    Some(&branch_coverage),
                    Some(&complexity),
                    &output_path,
                )?;
                cli::print_success(&format!("HTML report generated: {}", output_path.display()));
            } else if json_output {
                let combined_data = serde_json::json!({
                    "statement_coverage": stmt_coverage,
                    "branch_coverage": branch_coverage,
                    "complexity": complexity,
                    "summary": {
                        "overall_health": calculate_health_score(&stmt_coverage, &branch_coverage, &complexity),
                        "recommendations": generate_recommendations(&stmt_coverage, &branch_coverage, &complexity)
                    }
                });
                cli::export_to_json(&combined_data, args.output.as_deref())?;
            } else {
                // Print all reports
                cli::print_coverage_report(&stmt_coverage, verbose);
                cli::print_branch_coverage_report(&branch_coverage, verbose);
                cli::print_complexity_report(&complexity, verbose);
                
                // Print summary
                print_summary_report(&stmt_coverage, &branch_coverage, &complexity);
            }
        },
        
        Commands::Dot { function, image, .. } => {
            let (fset, objs, files) = parse_project(&project_path)?;
            
            let mut cfgs_map = HashMap::new();
            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }
            
            if let Some(graph) = cfgs_map.get(&function) {
                let dot = to_dot(graph, &function);
                
                if let Some(format) = image {
                    let output_path = args.output.unwrap_or_else(|| 
                        PathBuf::from(format!("{}.{}", function, format.extension()))
                    );
                    
                    cli::generate_image_from_dot(&dot, &output_path, format)?;
                    println!("Graph image for '{}' generated: {}", function, output_path.display());
                } else if let Some(path) = args.output {
                    fs::write(&path, dot)?;
                    println!("DOT graph for '{}' written to: {}", function, path.display());
                } else {
                    println!("{}", dot);
                }
            } else {
                eprintln!("Error: Function '{}' not found", function);
                std::process::exit(1);
            }
        },
        
        Commands::DotAll { image, .. } => {
            let (fset, objs, files) = parse_project(&project_path)?;
            
            let mut cfgs_map = HashMap::new();
            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }
            
            let out_dir = args.output.unwrap_or(PathBuf::from("dot_output"));
            if !out_dir.exists() {
                fs::create_dir_all(&out_dir)?;
            }
            
            for (fname, graph) in cfgs_map {
                let dot = to_dot(&graph, &fname);
                
                if let Some(format) = &image {
                    let filepath = out_dir.join(format!("{}.{}", fname, format.extension()));
                    cli::generate_image_from_dot(&dot, &filepath, format.clone())?;
                    if verbose {
                        println!("Generated image: {}", filepath.display());
                    }
                } else {
                    let filepath = out_dir.join(format!("{}.dot", fname));
                    fs::write(&filepath, dot)?;
                    if verbose {
                        println!("Generated DOT: {}", filepath.display());
                    }
                }
            }
            
            if let Some(format) = &image {
                println!("Graph images generated in directory: {} (format: {})", out_dir.display(), format.extension());
            } else {
                println!("DOT graphs generated in directory: {}", out_dir.display());
            }
        },
        
        Commands::StmtCov { threshold, timeout, .. } => {
            let options = CoverageOptions {
                verbose,
                min_coverage_threshold: threshold,
                timeout_seconds: timeout,
                ..CoverageOptions::default()
            };
            
            let coverage = analyze_statement_coverage_with_options(&project_path, &options)?;
            
            if json_output {
                cli::export_to_json(&coverage, args.output.as_deref())?;
            } else {
                cli::print_coverage_report(&coverage, verbose);
            }
        },
        
        Commands::Complexity { max_complexity, .. } => {
            let options = ComplexityOptions {
                verbose,
                max_allowed_complexity: max_complexity,
                ..ComplexityOptions::default()
            };
            
            let complexity = analyze_cyclomatic_complexity_with_options(&project_path, &options)?;
            
            if json_output {
                cli::export_to_json(&complexity, args.output.as_deref())?;
            } else {
                cli::print_complexity_report(&complexity, verbose);
            }
        },
        
        Commands::BranchCov { threshold, include_tests, timeout, .. } => {
            let options = BranchCoverageOptions {
                verbose,
                include_test_files: include_tests,
                min_coverage_threshold: threshold,
                fail_on_low_coverage: false,
                exclude_patterns: vec!["*_test.go".to_string(), "vendor/*".to_string()],
                simulate_coverage: false,
                test_args: Vec::new(),
                fail_on_error: false,
                timeout_seconds: timeout,
            };
            
            let coverage = analyze_branch_coverage_with_options(&project_path, &options)?;
            
            if json_output {
                cli::export_to_json(&coverage, args.output.as_deref())?;
            } else {
                cli::print_branch_coverage_report(&coverage, verbose);
            }
        },
        
        Commands::InitConfig { .. } => {
            // Already handled above
        }
    }
    
    Ok(())
}

fn calculate_health_score(
    stmt_coverage: &StmtCoverage,
    branch_coverage: &ProjectBranchCoverage,
    complexity: &ProjectComplexity,
) -> f64 {
    let coverage_score = (stmt_coverage.overall_coverage + branch_coverage.overall_coverage_percentage) / 2.0;
    let complexity_score = (20.0 - complexity.average_complexity.min(20.0)) / 20.0 * 100.0;
    
    (coverage_score * 0.7 + complexity_score * 0.3).min(100.0)
}

fn generate_recommendations(
    stmt_coverage: &StmtCoverage,
    branch_coverage: &ProjectBranchCoverage,
    complexity: &ProjectComplexity,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if stmt_coverage.overall_coverage < 80.0 {
        recommendations.push("Consider adding more unit tests to improve statement coverage".to_string());
    }
    
    if branch_coverage.overall_coverage_percentage < 75.0 {
        recommendations.push("Add tests for edge cases to improve branch coverage".to_string());
    }
    
    if complexity.average_complexity > 10.0 {
        recommendations.push("Consider refactoring complex functions to improve maintainability".to_string());
    }
    
    let high_complexity_count = complexity.functions.values()
        .filter(|f| f.cyclomatic_complexity > 15)
        .count();
    
    if high_complexity_count > 0 {
        recommendations.push(format!("Review {} functions with very high complexity (>15)", high_complexity_count));
    }
    
    recommendations
}

fn print_summary_report(
    stmt_coverage: &StmtCoverage,
    branch_coverage: &ProjectBranchCoverage,
    complexity: &ProjectComplexity,
) {
    let health_score = calculate_health_score(stmt_coverage, branch_coverage, complexity);
    let recommendations = generate_recommendations(stmt_coverage, branch_coverage, complexity);
    
    println!("\n{}", "=== Project Health Summary ===".bright_blue().bold());
    
    let health_color = if health_score >= 90.0 { health_score.to_string().green().bold() }
                      else if health_score >= 70.0 { health_score.to_string().yellow().bold() }
                      else { health_score.to_string().red().bold() };
    
    println!("{}: {}", "Overall Health Score".cyan(), health_color);
    
    if !recommendations.is_empty() {
        println!("\n{}:", "Recommendations".yellow().bold());
        for (i, rec) in recommendations.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().cyan(), rec);
        }
    }
}