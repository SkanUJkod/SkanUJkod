use crate::ast_node::AstNode;
use crate::parser_manager::ParserTrait;
use go_parser::ast::{Decl, Expr, FieldList, Spec, Stmt};
use go_parser::{AstObjects, ErrorList, FileSet, parse_file};

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
        let go_file = self
            .file_set
            .add_file(file_path.to_string(), None, source_code.len());
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

    fn convert_spec(spec: &Spec, objs: &AstObjects) -> AstNode {
        match spec {
            Spec::Import(import_spec) => {
                let mut import_node = AstNode::new("Import");

                // Assume BasicLit struct has a method or field for literal values directly, like a path
                let path_literal_value = import_spec.path.token.to_string();
                let path_node = AstNode::new(&path_literal_value);
                import_node.add_child(path_node);

                if let Some(alias_key) = &import_spec.name {
                    let alias_name = &objs.idents[*alias_key].name;
                    let alias_node = AstNode::new(alias_name);
                    import_node.add_child(alias_node);
                }
                import_node
            }
            Spec::Value(value_spec) => {
                let mut value_node = AstNode::new("ValueSpec");

                for &name_key in &value_spec.names {
                    let name = &objs.idents[name_key].name;
                    let name_node = AstNode::new(name);
                    value_node.add_child(name_node);
                }

                if let Some(typ) = &value_spec.typ {
                    let type_node = Self::convert_expr(typ, objs);
                    value_node.add_child(type_node);
                }

                for value in &value_spec.values {
                    let value_node_child = Self::convert_expr(value, objs);
                    value_node.add_child(value_node_child);
                }
                value_node
            }
            Spec::Type(type_spec) => {
                let mut type_node = AstNode::new("TypeSpec");

                let type_name = &objs.idents[type_spec.name].name;
                let name_node = AstNode::new(type_name);
                type_node.add_child(name_node);

                let typ_expr_node = Self::convert_expr(&type_spec.typ, objs);
                type_node.add_child(typ_expr_node);
                type_node
            }
        }
    }

    fn convert_decl(decl: &Decl, objs: &AstObjects) -> AstNode {
        match decl {
            Decl::Func(func_key) => {
                let func = &objs.fdecls[*func_key];
                let func_type = &objs.ftypes[func.typ];
                let mut func_node = AstNode::new("Function");

                let function_name = &objs.idents[func.name].name;
                func_node.add_child(AstNode::new(&format!("Name: {}", function_name)));

                let mut signature_node = AstNode::new("Signature");
                let params_node = Self::convert_params(&func_type.params, objs);
                signature_node.add_child(params_node);

                if let Some(results) = &func_type.results {
                    let results_node = Self::convert_params(results, objs);
                    signature_node.add_child(results_node);
                }

                func_node.add_child(signature_node);

                if let Some(body_block) = &func.body {
                    let body_node = Self::convert_stmt_list(&body_block.list, objs);
                    func_node.add_child(body_node);
                }
                func_node
            }
            Decl::Gen(gen_decl) => {
                let mut gen_node = AstNode::new("Generic");
                for spec_key in &gen_decl.specs {
                    let spec = &objs.specs[*spec_key];
                    gen_node.add_child(Self::convert_spec(spec, objs));
                }
                gen_node
            }
            Decl::Bad(_) => AstNode::new("BadDeclaration"),
        }
    }

    fn convert_params(params: &FieldList, objs: &AstObjects) -> AstNode {
        let mut params_node = AstNode::new("Parameters");
        for field_key in &params.list {
            let field = &objs.fields[*field_key];
            let field_name = if !field.names.is_empty() {
                &objs.idents[field.names[0]].name
            } else {
                "Unnamed"
            };
            let param_node = AstNode::new(&format!("Param: {}", field_name));
            params_node.add_child(param_node);
        }
        params_node
    }

    fn convert_stmt(stmt: &Stmt, objs: &AstObjects) -> AstNode {
        match stmt {
            Stmt::Decl(decl) => Self::convert_decl(decl, objs),
            Stmt::Expr(expr) => Self::convert_expr(expr, objs),
            Stmt::Assign(_) => AstNode::new("Assign"),
            Stmt::Empty(_) => AstNode::new("Empty"),
            Stmt::Labeled(_) => AstNode::new("Labeled"),
            Stmt::Send(_) => AstNode::new("Send"),
            Stmt::IncDec(_) => AstNode::new("IncDec"),
            Stmt::Go(_) => AstNode::new("Go"),
            Stmt::Defer(_) => AstNode::new("Defer"),
            Stmt::Return(_) => AstNode::new("Return"),
            Stmt::Branch(_) => AstNode::new("Branch"),
            Stmt::Block(_) => AstNode::new("Block"),
            Stmt::If(_) => AstNode::new("If"),
            Stmt::Case(_) => AstNode::new("Case"),
            Stmt::Switch(_) => AstNode::new("Switch"),
            Stmt::TypeSwitch(_) => AstNode::new("TypeSwitch"),
            Stmt::Comm(_) => AstNode::new("Comm"),
            Stmt::Select(_) => AstNode::new("Select"),
            Stmt::For(_) => AstNode::new("For"),
            Stmt::Range(_) => AstNode::new("Range"),
            Stmt::Bad(_) => AstNode::new("BadStatement"),
        }
    }

    fn convert_expr(expr: &Expr, objs: &AstObjects) -> AstNode {
        match expr {
            Expr::Ident(_) => AstNode::new("Ident"),
            Expr::BasicLit(_) => AstNode::new("BasicLit"),
            Expr::FuncLit(_) => AstNode::new("FuncLit"),
            Expr::CompositeLit(_) => AstNode::new("CompositeLit"),
            Expr::Paren(_) => AstNode::new("Paren"),
            Expr::Selector(_) => AstNode::new("Selector"),
            Expr::Index(_) => AstNode::new("Index"),
            Expr::Slice(_) => AstNode::new("Slice"),
            Expr::TypeAssert(_) => AstNode::new("TypeAssert"),
            Expr::Call(_) => AstNode::new("Call"),
            Expr::Star(_) => AstNode::new("Star"),
            Expr::Unary(_) => AstNode::new("Unary"),
            Expr::Binary(_) => AstNode::new("Binary"),
            Expr::KeyValue(_) => AstNode::new("KeyValue"),
            Expr::Array(_) => AstNode::new("Array"),
            Expr::Struct(_) => AstNode::new("Struct"),
            Expr::Func(_) => AstNode::new("Func"),
            Expr::Interface(_) => AstNode::new("Interface"),
            Expr::Map(_) => AstNode::new("Map"),
            Expr::Chan(_) => AstNode::new("Chan"),
            Expr::Ellipsis(_) => AstNode::new("Ellipsis"),
            Expr::Bad(_) => AstNode::new("BadExpr"),
        }
    }

    fn convert_stmt_list(stmts: &[Stmt], objs: &AstObjects) -> AstNode {
        let mut block_node = AstNode::new("Block");
        for stmt in stmts {
            let stmt_node = Self::convert_stmt(stmt, objs);
            block_node.add_child(stmt_node);
        }
        block_node
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
            .map(|e| {
                format!(
                    "Error at {}:{}:{}: {}",
                    e.pos.filename, e.pos.line, e.pos.column, e.msg
                )
            })
            .collect()
    }
}
