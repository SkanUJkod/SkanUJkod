



use std::{any::Any, ptr::eq};
use std::fs;
use std::path::Path;
use regex::{Matches, Regex};
use go_parser::ast::{Node, TypeSpec};
use std::mem::{self, discriminant};



pub fn fun_counter(file: &go_parser::ast::File) -> u32 { 
    return file.decls.iter().filter(|decl| matches!(decl, go_parser::ast::Decl::Func(_))).count() as u32;
}

pub fn typ_counter<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects) -> u32 {
    return o.entities.vec().iter().
    filter(|entity| matches!(&entity.kind, go_parser::scope::EntityKind::Typ)).count() as u32;	
}


pub fn name_vaiolation<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, kind_of_check: go_parser::scope::EntityKind, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    let re = Regex::new(&regex).unwrap();
    let information = String::from(format!("Name violation. Name do not match regex: {}", regex));
    return (o.entities.vec().iter().filter(|entity| (discriminant(&entity.kind)  == discriminant(&kind_of_check) )).
                filter(|entity| !re.is_match(&entity.name)).collect(),
            information);	
}

pub fn fun_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    return name_vaiolation(file, o, go_parser::scope::EntityKind::Fun, regex);
}

pub fn struct_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    return name_vaiolation(file, o, go_parser::scope::EntityKind::Typ, regex);
}

pub fn variable_name<'a>(file: &'a go_parser::ast::File, o: &'a go_parser::AstObjects, regex: &str) -> (Vec<&'a go_parser::scope::Entity>, String) {
    return name_vaiolation(file, o, go_parser::scope::EntityKind::Var, regex);
}
