use crate::parser_manager::ParserTrait;
use crate::ast_node::AstNode;
use go_parser::{AstObjects, ErrorList, FileSet, parse_file};
use go_parser::ast::{Node, Decl, Spec};

pub struct GoParser {
    objs: AstObjects,
    error_list: ErrorList,
    file_set: FileSet,
}

impl GoParser {
    pub fn new() -> Self {
        GoParser {
            objs: AstObjects::new(),
            error_list: ErrorList::new(),
            file_set: FileSet::new(),
        }
    }

    fn set_file(&mut self, file_path: &str, source_code: &str) {
        let go_file = self.file_set.add_file(file_path.to_string(), None, source_code.len());
        go_file.set_lines_for_content(&mut source_code.chars());
    }

    fn wrap_node(go_node: &go_parser::ast::File, objs: &AstObjects) -> AstNode {
        let mut root = AstNode::new("File");
        
        let package_node = AstNode::new("Package");
        root.add_child(package_node);
        
        for &import_key in &go_node.imports {
            let spec = &objs.specs[import_key];
            let import_spec_node = Self::convert_spec(spec, objs);
            root.add_child(import_spec_node);
        }
        
        for decl in &go_node.decls {
            let decl_node = Self::convert_decl(decl, objs);
            root.add_child(decl_node);
        }
        root
    }

    fn convert_spec(spec: &Spec, _objs: &AstObjects) -> AstNode {
        let spec_node = AstNode::new("Import");
        // TODO: Add conversion logic for import specifics
        spec_node
    }

    fn convert_decl(decl: &Decl, objs: &AstObjects) -> AstNode {
        match decl {
            Decl::Func(func) => {
                let mut func_node = AstNode::new("Function");
                // TODO: Add additional conversion logic for function specific details
                func_node.add_child(AstNode::new("Signature"));
                func_node.add_child(AstNode::new("Body"));
                func_node
            }
            Decl::Gen(r#gen) => {
                let gen_node = AstNode::new("Generic");
                // TODO: Add additional logic for generic declarations
                gen_node
            }
            Decl::Bad(_) => {
                AstNode::new("BadDeclaration")
            }
        }
    }
}

impl ParserTrait for GoParser {
    fn parse(&mut self, file_path: &str, source_code: &str) -> Result<AstNode, String> {
        self.set_file(file_path, source_code);

        let (_, maybe_file) = parse_file(
            &mut self.objs,
            &mut self.file_set,
            &self.error_list,
            file_path,
            source_code,
            false, // trace
        );

        if let Some(ast_file) = maybe_file {
            if self.error_list.len() == 0 {
                let wrapped_ast = GoParser::wrap_node(&ast_file, &self.objs);
                Ok(wrapped_ast)
            } else {
                Err("Parsing encountered errors.".to_string())
            }
        } else {
            Err("Parsing failed to produce an AST.".to_string())
        }
    }

    fn get_errors(&self) -> Vec<String> {
        self.error_list
            .borrow()
            .iter()
            .map(|e| format!("Error at {}:{}:{}: {}", e.pos.filename, e.pos.line, e.pos.column, e.msg))
            .collect()
    }
}