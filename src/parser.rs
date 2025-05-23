
use std::fs;
use std::path::Path;



pub fn parse_file(path: &str) -> (go_parser::ast::File, go_parser::AstObjects) {
    let source = &read_file(&String::from(path));
    let mut fs = go_parser::FileSet::new();
    let mut o =   go_parser::AstObjects::new();
    let el = &mut go_parser::ErrorList::new();
	
	let mut pf : go_parser::ast::File;
    let (p, pf_maybe) = go_parser::parse_file(&mut o, &mut fs, el, path, source, false);

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