
use SkanUJkod::static_analysis::{fun_counter, typ_counter, fun_name, struct_name, variable_name};

use std::fs;
use std::path::Path;
use regex::Regex;
use SkanUJkod::parser::parse_file;


// fn main() {


//     test_fun_counter();
//     test_type_counter();
//     test_fun_name_length();
//     test_struct_name_length();
//     test_variable_name_length();
// }

#[test]
fn test_fun_counter() {
    let (f, o) = parse_file("./go-files/main.go");
    assert_eq!(fun_counter(&f), 5);
}

#[test]
fn test_type_counter() {
    let (f, o) = parse_file("./go-files/main.go");
    assert_eq!(typ_counter(&f, &o), 3);
}
#[test]
fn test_fun_name_length() {
    let (f, o) = parse_file("./go-files/main.go");
    let (funNames, informationFun) = fun_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 4);
}
#[test]
fn test_struct_name_length() {
    let (f, o) = parse_file("./go-files/main.go");;
    let (funNames, informationFun) = struct_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 3);
}

#[test]
fn test_variable_name_length() {
    let (f, o) = parse_file("./go-files/main.go");
    let (funNames, informationFun) = variable_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 11);
}


// Test for function names which include "_" in their name
#[test]
fn test_fun_name() {
    let (f, o) = parse_file("./go-files/main.go");;
    let (funNames, informationFun) = fun_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 4);
    for funName in funNames.iter() {
        assert!(!Regex::new(r"_").unwrap().is_match(&funName.name));
    }
}


//camelCase test for function names
#[test]
fn test_fun_name2(){
    let (f, o) = parse_file("./go-files/main.go");;
    let (funNames, informationFun) = fun_name(&f, &o, r"_");
    assert_eq!(funNames.len(), 4);
    for funName in funNames.iter() {
        if (funName.name == "main"){
            assert!(true);   
        }
        else{
            assert!(!Regex::new(r"^[a-z]+(?:[A-Z][a-z0-9]*)*$").unwrap().is_match(&funName.name));
        } 
    }
}



