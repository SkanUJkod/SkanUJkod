use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skan-uj-kod")]
#[command(about = "A static analysis tool for Go code with plugin architecture")]
#[command(version = "0.1.0")]
#[command(long_about = "
SkanUJkod is a modular static analysis tool for Go projects.
It provides various analyses through a plugin system including:
- Control Flow Graph (CFG) generation
- Branch coverage analysis  
- Statement coverage analysis
- Cyclomatic complexity analysis

Set PLUGINS_DIR environment variable to specify plugin location.
")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate Control Flow Graph for Go project
    Cfg {
        /// Path to Go project directory
        #[arg(short, long)]
        project_path: PathBuf,
        
        /// Output file path (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Output format
        #[arg(short, long, default_value = "dot")]
        format: OutputFormat,
        
        /// Include test files in analysis
        #[arg(long, default_value = "false")]
        include_tests: bool,
        
        /// File patterns to exclude (glob patterns)
        #[arg(long)]
        exclude: Vec<String>,
    },
    
    /// Analyze branch coverage
    BranchCov {
        /// Path to Go project directory
        #[arg(short, long)]
        project_path: PathBuf,
        
        /// Output file path (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Minimum coverage threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.8")]
        threshold: f64,
        
        /// Include test files in analysis
        #[arg(long, default_value = "false")]
        include_tests: bool,
        
        /// File patterns to exclude (glob patterns)
        #[arg(long)]
        exclude: Vec<String>,
    },
    
    /// Analyze statement coverage
    StatementCov {
        /// Path to Go project directory
        #[arg(short, long)]
        project_path: PathBuf,
        
        /// Output file path (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Minimum coverage threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.8")]
        threshold: f64,
        
        /// Include test files in analysis
        #[arg(long, default_value = "false")]
        include_tests: bool,
        
        /// File patterns to exclude (glob patterns)
        #[arg(long)]
        exclude: Vec<String>,
    },
    
    /// Analyze cyclomatic complexity
    Complexity {
        /// Path to Go project directory
        #[arg(short, long)]
        project_path: PathBuf,
        
        /// Output file path (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Maximum allowed complexity
        #[arg(short, long, default_value = "10")]
        max_complexity: i32,
        
        /// Include test files in analysis
        #[arg(long, default_value = "false")]
        include_tests: bool,
        
        /// File patterns to exclude (glob patterns)
        #[arg(long)]
        exclude: Vec<String>,
    },
    
    /// Run all available analyses
    All {
        /// Path to Go project directory  
        #[arg(short, long)]
        project_path: PathBuf,
        
        /// Output directory for results (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        
        /// Include test files in analysis
        #[arg(long, default_value = "false")]
        include_tests: bool,
        
        /// File patterns to exclude (glob patterns)
        #[arg(long)]
        exclude: Vec<String>,
    },
    
    /// List available plugins and their capabilities
    ListPlugins,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// DOT format for graphviz
    Dot,
    /// JSON format
    Json,
    /// Human-readable text
    Text,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Dot => write!(f, "dot"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Text => write!(f, "text"),
        }
    }
}
