use std::any::Any;
use std::fs;
use std::path::Path;
use regex::Regex;
use go_parser::ast::{Node, TypeSpec};

fn main() {



	let (f,o) = parse_file();

	//println!("File: {:?}", f);
	println!("Objects: {:?}", o.fdecls);
	
	let funNames =  fun_name(&f, &o);//human readble output for problem insted (kind of check, regex or predicate , type of message )
	for funName in funNames.iter(){
		println!("Function name violation: {:?}", funName.name);
	}
	//stmt type definition for struct counter
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

fn fun_counter(file: &go_parser::ast::File) -> u32 { 
	let mut counter: u32 = 0;
	for decl in file.decls.iter(){
		match decl{
			go_parser::ast::Decl::Func(_) => {
				counter += 1;
				
			},
			_ => {}
		}
	}
	return counter;
}

fn name_vaiolation<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, kind_of_check: go_parser::scope::EntityKind) -> Vec<&'a go_parser::scope::Entity> {
	let re = Regex::new(r"_").unwrap();
	return o.entities.vec().iter().filter(|entity| matches!(&entity.kind, kind_of_check)).filter(|entity| !re.is_match(&entity.name)).collect();	
}

fn fun_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects) -> Vec<&'a go_parser::scope::Entity> {
	return name_vaiolation(file, o, go_parser::scope::EntityKind::Fun);
}

fn struct_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects) -> Vec<&'a go_parser::scope::Entity> {
	return name_vaiolation(file, o, go_parser::scope::EntityKind::Struct);
}

