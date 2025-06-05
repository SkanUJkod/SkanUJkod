
use SkanUJkod::static_analysis::{fun_counter, typ_counter, fun_name, struct_name, variable_name};

use std::fs;
use std::path::Path;
use regex::Regex;
use SkanUJkod::parser::parse_file;

#[test] 
fn test_fun_name2() {
    let (f, o) = parse_file("./go-files/main.go");
    let (funNames, informationFun) = fun_name(&f, &o, r"([a-z]+[A-Z]+)");
    assert_eq!(funNames.len(), 5);
    for funName in funNames.iter() {
        assert!(!Regex::new(r"([a-z]+[A-Z]+)").unwrap().is_match(&funName.name));
    }
}
#[test]
fn test_fun_counter() {
    let (f, o) = parse_file("./go-files/main2.go");
    assert_eq!(fun_counter(&f), 1);
}


#[test]
fn test_fun_counter2(){
    let (f, o) = parse_file("./go-files/main3.go");
    assert_eq!(fun_counter(&f), 0);
}