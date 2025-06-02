use anyhow::Result;
use go_parser::{AstObjects, ErrorList, FileSet, parse_file};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub struct ParsedFile {
    pub path: PathBuf,
    pub ast: go_parser::ast::File,
}

pub fn parse_project(root: &Path) -> Result<(FileSet, AstObjects, Vec<ParsedFile>)> {
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
