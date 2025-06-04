use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub coverage: CoverageConfig,
    pub complexity: ComplexityConfig,
    pub output: OutputConfig,
    pub analysis: AnalysisConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CoverageConfig {
    pub threshold: f64,
    pub timeout_seconds: u64,
    pub exclude_patterns: Vec<String>,
    pub include_test_files: bool,
    pub fail_on_low_coverage: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComplexityConfig {
    pub max_complexity: usize,
    pub cognitive_threshold: usize,
    pub lines_of_code_threshold: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    pub format: String, // "json", "text", "html"
    pub output_dir: PathBuf,
    pub verbose: bool,
    pub colored: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisConfig {
    pub parallel_processing: bool,
    pub cache_results: bool,
    pub incremental_analysis: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            coverage: CoverageConfig {
                threshold: 80.0,
                timeout_seconds: 30,
                exclude_patterns: vec![
                    "*_test.go".to_string(),
                    "vendor/*".to_string(),
                    "testdata/*".to_string(),
                ],
                include_test_files: false,
                fail_on_low_coverage: false,
            },
            complexity: ComplexityConfig {
                max_complexity: 10,
                cognitive_threshold: 15,
                lines_of_code_threshold: 50,
            },
            output: OutputConfig {
                format: "text".to_string(),
                output_dir: PathBuf::from("./reports"),
                verbose: false,
                colored: true,
            },
            analysis: AnalysisConfig {
                parallel_processing: true,
                cache_results: true,
                incremental_analysis: false,
            },
        }
    }
}

impl Config {
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load_or_default(path: Option<&std::path::Path>) -> Self {
        match path {
            Some(p) if p.exists() => Self::load_from_file(p).unwrap_or_else(|_| {
                eprintln!("Warning: Failed to load config file, using defaults");
                Self::default()
            }),
            _ => Self::default(),
        }
    }
}
