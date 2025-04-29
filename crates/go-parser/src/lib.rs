mod errors;
mod map;
mod objects;
mod parser;
mod position;
mod scanner;
mod token;

pub mod ast;
pub mod scope;
pub mod visitor;

pub use errors::*;
pub use map::{Map, MapIter};
pub use objects::*;
pub use parser::Parser;
pub use position::*;
use std::{collections::HashMap, fs};
pub use token::*;

/// Copyright 2022 The Goscript Authors. All rights reserved.
/// Use of this source code is governed by a BSD-style
/// license that can be found in the LICENSE file.

/// This crate is part of the Goscript project. Please refer to <https://goscript.dev> for more information.
//
/// It's a port of the the parser from the Go standard library <https://github.com/golang/go/tree/release-branch.go1.12/src/go/parser>
//
/// # Usage:
/// ```
/// fn parse_file() {
///     let source = "package main ...";
///     let mut fs = go_parser::FileSet::new();
///     let o = &mut go_parser::AstObjects::new();
///     let el = &mut go_parser::ErrorList::new();
///     let (p, _) = go_parser::parse_file(o, &mut fs, el, "./main.go", source, false);
///     print!("{}", p.get_errors());
/// }
/// ```
//
/// # Feature
/// - `btree_map`: Make it use BTreeMap instead of HashMap
///
pub fn parse_file<'a>(
    o: &'a mut AstObjects,
    fs: &'a mut FileSet,
    el: &'a ErrorList,
    name: &str,
    src: &'a str,
    trace: bool,
) -> (parser::Parser<'a>, Option<ast::File>) {
    let f = fs.add_file(name.to_string(), None, src.chars().count());
    let mut p = parser::Parser::new(o, f, el, src, trace);
    let file = p.parse_file();
    (p, file)
}

/// Parses all `.go` source files in the given directory and returns a map
/// of package name to `ast::Package` containing all the parsed files.
///
/// If a `filter` is provided, only files for which the filter returns true
/// (and which end in `.go`) will be parsed.
///
/// Position information is recorded in the provided `FileSet`, and parsed
/// AST nodes are built using the provided `AstObjects`. Any syntax errors
/// encountered during parsing will be recorded in the given `ErrorList`.
///
/// If the directory cannot be read, an error is returned. If parsing errors
/// occur, an incomplete map is returned with syntax errors stored in the
/// `ErrorList`.
///
/// # Arguments
///
/// * `o`     - Mutable reference to `AstObjects`, used to allocate AST nodes.
/// * `fs`    - Mutable reference to `FileSet`, used for position tracking.
/// * `el`    - Mutable reference to `ErrorList`, for collecting syntax errors.
/// * `path`  - Path to the directory to parse.
/// * `trace` - If `true`, enables debug tracing during parsing.
/// * `filter` - Optional filter function to include only matching files.
///
/// # Usage
/// ```
/// let mut fs = go_parser::FileSet::new();
/// let mut o = go_parser::AstObjects::new();
/// let mut el = go_parser::ErrorList::new();
/// let path = std::path::Path::new("./examples");
/// let pkgs = go_parser::parse_dir(&mut o, &mut fs, &mut el, path, false, None).unwrap();
/// for (pkg_name, pkg) in pkgs {
///     println!("Parsed package: {}", pkg_name);
///     for (file_name, _) in pkg.files {
///         println!(" - {}", file_name);
///     }
/// }
/// if !el.is_empty() {
///     println!("Errors:\n{}", el);
/// }
/// ```
///
pub fn parse_dir<'a>(
    o: &'a mut AstObjects,
    fs: &'a mut FileSet,
    el: &'a ErrorList,
    path: &str,
    src: &'a str,
    trace: bool,
    filter: Option<&dyn Fn(&fs::DirEntry) -> bool>,
) -> Result<HashMap<String, ast::Package>, std::io::Error> {
    let mut pkgs: HashMap<String, ast::Package> = HashMap::new();
    let entries = fs::read_dir(path)?;

    for entry_result in entries {
        let entry = entry_result?;
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("go") {
            continue;
        }

        if let Some(filter_fn) = filter {
            if !filter_fn(&entry) {
                continue;
            }
        }

        let src = std::fs::read_to_string(&path)?;
        let name = path.to_string_lossy();

        let (_parser, file_opt) = crate::parse_file(o, fs, el, &name, &src, trace);
        if let Some(file) = file_opt {
            let pkg_ident_key: IdentKey = file.name.clone();
            let ident = &o.idents[pkg_ident_key];
            let pkg_name: String = ident.name.clone();

            let mut pkg = ast::Package::new(pkg_name);

            pkg.files_mut().insert(name.to_string(), Box::new(file));
            pkgs.insert(pkg.name().to_string(), pkg);
        }
    }

    Ok(pkgs)
}
