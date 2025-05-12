use std::{
    collections::HashMap,
    fs,
    path::Path,
    process::Command,
};

use anyhow::{bail, Context, Result};
use go_parser::{
    ast::{Decl, Node},
    parse_file, AstObjects, ErrorList, FileSet,
};
use tempfile::TempDir;
use walkdir::{DirEntry, WalkDir};

pub fn run_statement_coverage(project_root: &Path) -> Result<f64> {

    let tmp_dir = TempDir::new().context("creating temp workspace")?;
    let tmp_path = tmp_dir.path();


    let module_path = detect_module_path(project_root)?;
    let pkg_name = detect_primary_package(project_root).unwrap_or_else(|| "main".into());


    mirror_project_sources(project_root, tmp_path)?;

    if !project_root.join("go.mod").exists() {
        init_go_mod(tmp_path, &module_path)?;
    }


    write_coverage_pkg(tmp_path)?;
    write_test_main(tmp_path, &module_path, &pkg_name)?;


    let mut total_stmts = 0usize;
    for entry in WalkDir::new(tmp_path)
        .into_iter()
        .filter_entry(|e| !is_cov_dir(e))
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("go") {
            continue;
        }
        if p.file_name()
            .and_then(|s| s.to_str())
            .map_or(false, |n| n == "cov_test_main_test.go")
        {
            continue;
        }
        instrument_file(p, project_root, &module_path, &mut total_stmts)
            .with_context(|| format!("instrumenting {}", p.display()))?;
    }

    let output = Command::new("go")
        .current_dir(tmp_path)
        .args(["test", "./...", "-v"])
        .output()
        .context("running `go test`")?;

    if !output.status.success() {
        bail!(
            "go test failed (exit {}):\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut counts = HashMap::<String, usize>::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some((id, cnt)) = line.rsplit_once(": ") {
            if let Ok(n) = cnt.parse::<usize>() {
                counts.insert(id.to_string(), n);
            }
        }
    }
    let executed = counts.values().filter(|&&c| c > 0).count();

    Ok(if total_stmts == 0 {
        0.0
    } else {
        executed as f64 / total_stmts as f64 * 100.0
    })
}


fn detect_module_path(project_root: &Path) -> Result<String> {
    let go_mod = project_root.join("go.mod");
    if go_mod.exists() {
        let content = fs::read_to_string(&go_mod).context("reading go.mod")?;
        if let Some(line) = content
            .lines()
            .find(|l| l.trim_start().starts_with("module "))
        {
            return Ok(line
                .trim_start_matches("module")
                .trim()
                .to_string());
        }
    }
    Ok(project_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("covmod")
        .to_string())
}

fn detect_primary_package(project_root: &Path) -> Option<String> {
    fs::read_dir(project_root)
        .ok()?
        .filter_map(Result::ok)
        .find_map(|entry| {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("go") {
                return None;
            }
            if p.file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |n| n.ends_with("_test.go"))
            {
                return None;
            }
            fs::read_to_string(&p)
                .ok()?
                .lines()
                .find(|l| l.trim_start().starts_with("package "))
                .map(|l| {
                    l.trim_start()
                        .trim_start_matches("package ")
                        .trim()
                        .to_string()
                })
        })
}

fn mirror_project_sources(src_root: &Path, dst_root: &Path) -> Result<()> {
    for entry in WalkDir::new(src_root).into_iter().filter_map(Result::ok) {
        let rel = entry.path().strip_prefix(src_root)?;
        if entry.file_name() == "cov.go" {
            continue;
        }
        let dest = dst_root.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest)?;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

fn init_go_mod(work_dir: &Path, module_path: &str) -> Result<()> {
    let status = Command::new("go")
        .current_dir(work_dir)
        .args(["mod", "init", module_path])
        .status()
        .context("running go mod init")?;
    if !status.success() {
        bail!("`go mod init {}` failed", module_path);
    }
    Ok(())
}

fn write_coverage_pkg(tmp_path: &Path) -> Result<()> {
    let pkg = r#"package cov
import "fmt"
var C map[string]int
func init() { C = make(map[string]int) }
func Init() { C = make(map[string]int) }
func Inc(id string) { C[id]++ }
func Report() { for k,v := range C { fmt.Printf("%s: %d\n", k, v) } }
"#;
    let cov_dir = tmp_path.join("cov");
    fs::create_dir_all(&cov_dir)?;
    fs::write(cov_dir.join("cov.go"), pkg)?;
    Ok(())
}

fn write_test_main(tmp_path: &Path, module_name: &str, pkg_name: &str) -> Result<()> {
    let body = format!(
        r#"package {pkg}

import (
    "os"
    "testing"
    "{module}/cov"
)

func TestMain(m *testing.M) {{
    cov.Init()
    code := m.Run()
    cov.Report()
    os.Exit(code)
}}
"#,
        pkg = pkg_name,
        module = module_name
    );
    fs::write(tmp_path.join("cov_test_main_test.go"), body)?;
    Ok(())
}

fn is_cov_dir(entry: &DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|c| c.as_os_str() == "cov")
}

fn instrument_file(
    path: &Path,
    project_root: &Path,
    module_name: &str,
    total: &mut usize,
) -> Result<()> {
    let src = fs::read_to_string(path).context("reading file")?;

    let mut fileset = FileSet::new();
    let mut objs = AstObjects::new();
    let mut errs = ErrorList::new();
    let (_, ast) = parse_file(
        &mut objs,
        &mut fileset,
        &mut errs,
        path.to_str().unwrap(),
        &src,
        false,
    );
    let ast = ast.ok_or_else(|| anyhow::anyhow!("go_parser returned no AST"))?;

    let mut lines: Vec<String> = src.lines().map(|l| l.to_string()).collect();
    let mut offset = 0usize;

    let cov_import = format!("\"{}/cov\"", module_name);

    for decl in &ast.decls {
        if let Decl::Func(fkey) = decl {
            let func = &objs.fdecls[*fkey];
            if let Some(body) = &func.body {
                for stmt in &body.list {
                    if let Some(pos) = fileset.position(stmt.pos(&objs)) {
                        let id = format!(
                            "{}:{}:stmt{}",
                            path.strip_prefix(project_root)
                                .unwrap_or(path)
                                .display(),
                            pos.line,
                            *total
                        );
                        let insert_at = pos.line - 1 + offset;
                        lines.insert(insert_at, format!("    cov.Inc({:?})", id));
                        offset += 1;
                        *total += 1;
                    }
                }
            }
        }
    }

    ensure_cov_import(&mut lines, &cov_import)?;

    fs::write(path, lines.join("\n")).context("writing instrumented file")?;
    Ok(())
}

fn ensure_cov_import(lines: &mut Vec<String>, cov_import: &str) -> Result<()> {
    let pkg_idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with("package "))
        .unwrap_or(0);

    if let Some(imp_idx) = lines.iter().position(|l| l.trim_start().starts_with("import")) {
        let original_line = lines[imp_idx].clone();
        let original_trim = original_line.trim_start();

        if original_trim.starts_with("import \"") {
            let existing = original_trim.trim_start_matches("import").trim().to_string();
            if !existing.contains(cov_import) {
                lines[imp_idx] = "import (".into();
                lines.insert(imp_idx + 1, format!("    {}", existing));
                lines.insert(imp_idx + 2, format!("    {}", cov_import));
                lines.insert(imp_idx + 3, ")".into());
            }
        } else {
            if let Some(end_rel) = lines[imp_idx..].iter().position(|l| l.trim() == ")") {
                let block = &lines[imp_idx + 1..imp_idx + end_rel];
                if !block.iter().any(|l| l.contains(cov_import)) {
                    lines.insert(imp_idx + end_rel, format!("    {}", cov_import));
                }
            }
        }
    } else {
        lines.insert(pkg_idx + 1, "import (".into());
        lines.insert(pkg_idx + 2, format!("    {}", cov_import));
        lines.insert(pkg_idx + 3, ")".into());
    }
    Ok(())
}