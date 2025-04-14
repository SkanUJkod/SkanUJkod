use std::any::Any;
use std::fs;
use std::path::Path;

use go_parser::ast::Node;

fn main() {
     //parse_file();
	let file_path = String::from("main.go");
	let pf = parse_file();
	let fun_count = fun_counter(pf);
	println!("Number of functions in file: {}", fun_count);
}


fn parse_file() -> go_parser::ast::File{
    let source = &read_file(&String::from("main.go"));
    let mut fs = go_parser::FileSet::new();
    let o = &mut go_parser::AstObjects::new();
    let el = &mut go_parser::ErrorList::new();
    let (p, pf_maybe) = go_parser::parse_file(o, &mut fs, el, "./main.go", source, false);
    
	 let pf = match pf_maybe{
 		Some(pf_maybe) => pf_maybe,
	 	None => {
			
			panic!("Error parsing file: {:?}", el);
		},	
	}; 	
	return pf;
}

fn read_file(file_path: &String) -> String{
	
    
    let contents = fs::read_to_string(file_path)
    .expect("Should have been able to read the file");
    contents
}

fn fun_counter(file: go_parser::ast::File) -> u32 { 
	let mut counter: u32 = 0;
	for decl in file.decls{
		let decl_str = format!("{:?}", decl);
		if decl_str.contains("FuncDecl") {	
			counter += 1;
		}
	}
	return counter;
}