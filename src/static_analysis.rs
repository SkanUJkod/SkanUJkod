use std::{any::Any, ptr::eq};
use std::fs;
use std::path::Path;
use regex::{Matches, Regex};
use go_parser::ast::{Node, TypeSpec};
use std::mem::{self, discriminant};


//counter number of functions in the file
pub fn fun_counter(file: &go_parser::ast::File) -> u32 { 
    return file.decls.iter().filter(|decl| matches!(decl, go_parser::ast::Decl::Func(_))).count() as u32;
}
//counter number of types in the file (by type I mean class but go does not have classes)
pub fn typ_counter<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects) -> u32 {
    return o.entities.vec().iter().
    filter(|entity| matches!(&entity.kind, go_parser::scope::EntityKind::Typ)).count() as u32;	
}

//matches the name of the entity with the regex
pub fn name_vaiolation<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, kind_of_check: go_parser::scope::EntityKind, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    let re = Regex::new(&regex).unwrap();
    let information = String::from(format!("Name violation. Name do not match regex: {}", regex));
    return (o.entities.vec().iter().filter(|entity| (discriminant(&entity.kind)  == discriminant(&kind_of_check) )).
                filter(|entity| !re.is_match(&entity.name) && entity.name != "main").collect(),
            information);	
}
//name_violation for function names
pub fn fun_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    return name_vaiolation(file, o, go_parser::scope::EntityKind::Fun, regex);
}
//name violation for struct names
pub fn struct_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    return name_vaiolation(file, o, go_parser::scope::EntityKind::Typ, regex);
}
//name violation for variable names
pub fn variable_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    return name_vaiolation(file, o, go_parser::scope::EntityKind::Var, regex);
}


//works only with parser written by Pawel 
// pub fn if_depth(node: &AstNode, depth: &mut usize) -> usize {
//     let mut max_depth = *depth;
//     if node.kind == "If" {
//         *depth += 1;
//         max_depth = *depth;
//     }
//     for child in &node.children {
//         let child_depth = if_depth(child, depth);
//         if child_depth > max_depth {
//             max_depth = child_depth;
//         }
//     }
//     if node.kind == "If" {
//         *depth -= 1;
//     }
//     max_depth
// }