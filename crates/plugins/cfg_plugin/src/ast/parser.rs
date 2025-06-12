use anyhow::Result;
use go_parser::{AstObjects, ErrorList, FileSet, parse_file};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub ast: go_parser::ast::File,
}

/// Options for filtering files during project parsing
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub exclude_patterns: Vec<String>,
    pub include_test_files: bool,
}

/// Check if a file should be excluded based on patterns
fn should_exclude_file(path: &Path, exclude_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    for pattern in exclude_patterns {
        if path_str.contains(pattern.trim_matches('*')) {
            return true;
        }
    }
    false
}

/// Check if a path is a Go test file
fn is_test_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with("_test.go"))
        .unwrap_or(false)
}

pub fn parse_project(root: &Path) -> Result<(FileSet, AstObjects, Vec<ParsedFile>)> {
    parse_project_with_options(root, &ParseOptions::default())
}

pub fn parse_project_with_options(
    root: &Path,
    options: &ParseOptions,
) -> Result<(FileSet, AstObjects, Vec<ParsedFile>)> {
    let mut fset = FileSet::new();
    let mut objs = AstObjects::new();
    let mut errs = ErrorList::new();
    let mut files = Vec::new();

    let meta = fs::metadata(root)?;
    let walker = if meta.is_dir() {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect()
    } else {
        vec![
            WalkDir::new(root)
                .min_depth(0)
                .max_depth(0)
                .into_iter()
                .next()
                .unwrap()?,
        ]
    };

    for entry in walker {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("go") {
            let is_test = is_test_file(p);

            // Skip test files if not included
            if !options.include_test_files && is_test {
                continue;
            }

            // Apply filtering logic, but don't exclude test files if they should be included
            if !is_test && should_exclude_file(p, &options.exclude_patterns) {
                continue;
            }

            let src = fs::read_to_string(p)?;
            let (_, ast_opt) = parse_file(
                &mut objs,
                &mut fset,
                &mut errs,
                p.to_str().unwrap(),
                &src,
                false,
            );
            if let Some(ast) = ast_opt {
                files.push(ParsedFile {
                    path: p.to_path_buf(),
                    ast,
                });
            }
        }
    }

    if errs.len() > 0 {
        eprintln!("⚠️ parser errors:\n{:#?}", errs);
    }

    Ok((fset, objs, files))
}
