
use SkanUJkod::static_analysis::{fun_counter, typ_counter, fun_name, struct_name, variable_name};

use std::fs;
use std::path::Path;
use regex::Regex;

fn main() {


    test_fun_counter();
    test_type_counter();
    test_fun_name_length();
    test_struct_name_length();
    test_variable_name_length();
}

#[test]
fn test_fun_counter() {
    let (f, o) = parse_file();
    assert_eq!(fun_counter(&f), 6);
}

#[test]
fn test_type_counter() {
    let (f, o) = parse_file();
    assert_eq!(typ_counter(&f, &o), 3);
}
#[test]
fn test_fun_name_length() {
    let (f, o) = parse_file();
    let (funNames, informationFun) = fun_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 5);
}
#[test]
fn test_struct_name_length() {
    let (f, o) = parse_file();
    let (funNames, informationFun) = struct_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 3);
}

#[test]
fn test_variable_name_length() {
    let (f, o) = parse_file();
    let (funNames, informationFun) = variable_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 11);
}

fn parse_file() -> (go_parser::ast::File, go_parser::AstObjects) {
    let source = &read_file(&String::from("main.go"));
    let mut fs = go_parser::FileSet::new();
    let mut o =   go_parser::AstObjects::new();
    let el = &mut go_parser::ErrorList::new();
	
	let mut pf : go_parser::ast::File;
    let (p, pf_maybe) = go_parser::parse_file(&mut o, &mut fs, el, "./main.go", source, false);

	 let pf = match pf_maybe{
 		Some(pf_maybe) => pf_maybe,
	 	None => {
			
			panic!("Error parsing file: {:?}", el);
		},	
	}; 	


		
	
	return (pf, o);
}

fn read_file(file_path: &String) -> String{
	
    
    let contents = fs::read_to_string(file_path)
    .expect("Should have been able to read the file");
    contents
}