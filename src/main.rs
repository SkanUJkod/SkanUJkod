use std::{any::Any, ptr::eq};
use std::fs;
use std::path::Path;
use SkanUJkod::static_analysis::{fun_counter, typ_counter, fun_name, struct_name, variable_name};

fn main() {



	let (f,o) = parse_file();

	//println!("File: {:?}", f);
	//println!("Objects: {:?}", o.fdecls);
	
	//println!("{:?}", o.entities.vec());
	//println!("{:?}", o.fdecls);
	//println!("{:?}", o.fields);
	//println!("{:?}", o.ftypes);
	//println!("{:?}", o.idents);
	println!("Number of functions: {:?}", fun_counter(&f));
	let (funNames, informationFun ) =  fun_name(&f, &o, r"_");//human readble output for problem insted (kind of check, regex or predicate , type of message )
	println!("{}", informationFun);
	for funName in funNames.iter(){
		println!("{:?}", funName.name);
	}
	//stmt type definition for struct counter

	let (typNames, informationTyp) =  struct_name(&f, &o, r"_");
	println!("{}", informationTyp);
	for structName in typNames.iter(){
		println!("Struct name violation: {:?}", structName.name);
	}
	let (varNames, informationVar) =  variable_name(&f, &o, r"_");
	println!("{}", informationVar);
	for varName in varNames.iter(){
		println!("Variable name violation: {:?}", varName.name);
	}
	println!("Number of types: {:?}", typ_counter(&f, &o));
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

