extern crate go_parser as fe;
use go_parser::{parse_dir, AstObjects, ErrorList, FileSet};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

fn load_parse(path: &str, trace: bool) -> usize {
    let mut fs = fe::FileSet::new();
    let src = fs::read_to_string(path).expect("read file err: ");
    let o = &mut fe::AstObjects::new();
    let el = &mut fe::ErrorList::new();
    let (p, _) = fe::parse_file(o, &mut fs, el, path, &src, trace);

    print!("{}", p.get_errors());

    let l = p.get_errors().len();
    l
}

#[test]
fn test_parser_case0() {
    load_parse("./../../go/src/archive/tar/strconv_test.go", true);
}

#[test]
fn test_parser_case1() {
    let err_cnt = load_parse("./tests/data/case1.gos", true);
    dbg!(err_cnt);
}

#[test]
fn test_issue3() {
    let mut fs = fe::FileSet::new();
    let o = &mut fe::AstObjects::new();
    let el = &mut fe::ErrorList::new();
    let (p, _) = fe::parse_file(o, &mut fs, el, "/a", "`", false);
    print!("{}", p.get_errors());
}

#[test]
fn test_basic_parse_dir() {
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("main.go");
    let go_source = r#"
        package main
        func main() {
            println("Hello, world")
        }
    "#;

    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(go_source.as_bytes())
        .expect("Failed to write to file");

    let mut ast_objects = AstObjects::new();
    let mut file_set = FileSet::new();
    let error_list = ErrorList::new();

    let result = parse_dir(
        &mut ast_objects,
        &mut file_set,
        &error_list,
        dir.path().to_str().unwrap(),
        "",
        false,
        None,
    );

    assert!(result.is_ok(), "Parsing failed when it should succeed");
    let pkgs: HashMap<String, _> = result.unwrap();
    assert!(!pkgs.is_empty(), "Packages should not be empty");
    assert!(pkgs.contains_key("main"), "Package 'main' not found");
}

#[test]
fn test_parse_dir_with_no_go_files() {
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("not_a_go_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(b"This is not a Go file.")
        .expect("Failed to write to file");

    let mut ast_objects = AstObjects::new();
    let mut file_set = FileSet::new();
    let error_list = ErrorList::new();

    let result = parse_dir(
        &mut ast_objects,
        &mut file_set,
        &error_list,
        dir.path().to_str().unwrap(),
        "",
        false,
        None,
    );

    assert!(result.is_ok(), "Parsing failed when it should succeed");
    let pkgs: HashMap<String, _> = result.unwrap();
    assert!(
        pkgs.is_empty(),
        "Packages should be empty as there are no Go files"
    );
}

#[test]
fn test_parse_dir_with_invalid_path() {
    let mut ast_objects = AstObjects::new();
    let mut file_set = FileSet::new();
    let error_list = ErrorList::new();

    let result = parse_dir(
        &mut ast_objects,
        &mut file_set,
        &error_list,
        "invalid_path",
        "",
        false,
        None,
    );

    assert!(result.is_err(), "Parsing did not fail on invalid path");
}

#[test]
fn test_parse_dir_with_filter() {
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path1 = dir.path().join("main.go");
    let file_path2 = dir.path().join("skip.go");

    let go_source = r#"
        package main
        func main() {
            println("Hello, world")
        }
    "#;
    File::create(&file_path1)
        .expect("Failed to create file")
        .write_all(go_source.as_bytes())
        .expect("Failed to write to file");

    File::create(&file_path2)
        .expect("Failed to create file")
        .write_all(go_source.as_bytes())
        .expect("Failed to write to file");

    let mut ast_objects = AstObjects::new();
    let mut file_set = FileSet::new();
    let error_list = ErrorList::new();

    let filter = |entry: &fs::DirEntry| entry.file_name() != "skip.go";

    let result = parse_dir(
        &mut ast_objects,
        &mut file_set,
        &error_list,
        dir.path().to_str().unwrap(),
        "",
        false,
        Some(&filter),
    );

    assert!(
        result.is_ok(),
        "Parsing failed when it should succeed with a filter"
    );
    let pkgs: HashMap<String, _> = result.unwrap();
    assert!(!pkgs.is_empty(), "Packages should not be empty");
    assert!(pkgs.contains_key("main"), "Package 'main' not found");
    assert!(
        !pkgs.contains_key("skip"),
        "Package 'skip' should not be in packages"
    );
}

#[test]
fn test_parse_dir_with_imports() {
    let dir = tempdir().expect("Failed to create temp dir");

    let file_a_path = dir.path().join("a.go");
    let file_b_path = dir.path().join("b.go");

    let file_a_src = r#"
        package main
        import "b"
        func main() {
            b.CallB()
        }
    "#;

    let file_b_src = r#"
        package b
        func CallB() {
            println("Hello from B")
        }
    "#;

    let mut file_a = File::create(&file_a_path).expect("Failed to create file a.go");
    file_a
        .write_all(file_a_src.as_bytes())
        .expect("Failed to write to a.go");

    let mut file_b = File::create(&file_b_path).expect("Failed to create file b.go");
    file_b
        .write_all(file_b_src.as_bytes())
        .expect("Failed to write to b.go");

    let mut ast_objects = AstObjects::new();
    let mut file_set = FileSet::new();
    let error_list = ErrorList::new();

    let result = parse_dir(
        &mut ast_objects,
        &mut file_set,
        &error_list,
        dir.path().to_str().unwrap(),
        "",
        false,
        None,
    );

    assert!(
        result.is_ok(),
        "Parsing failed even though the setup was correct"
    );
    let pkgs: HashMap<String, _> = result.unwrap();
    assert!(pkgs.contains_key("main"), "Package 'main' not found");
    assert!(pkgs.contains_key("b"), "Package 'b' not found");
}
