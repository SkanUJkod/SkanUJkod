use std::{any::Any, ptr::eq};
use std::fs;
use std::path::Path;
use SkanUJkod::static_analysis::{fun_counter, typ_counter, fun_name, struct_name, variable_name};
use SkanUJkod::parser::parse_file;


fn main() {
	let (f,o) = parse_file("./main2.go");



	if let Some(rc_block) = &o.fdecls.vec()[0].body {
		println!("Block: {:?}", rc_block);
		// Assuming rc_block is a reference to a block of statements
		// You can iterate over the statements in the block
		for stmt in &rc_block.list {
			print!("----------------------------\n");
			println!("Statement: {:?}", stmt);
		}
	} else {
		println!("No block found");
	}

}






