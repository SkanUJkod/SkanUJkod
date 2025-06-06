use crate::ast_node::AstNode;
use crate::parser_manager::ParserTrait;
use go_parser::ast::{Decl, Expr, FieldList, Spec, Stmt};
use go_parser::{AstObjects, ErrorList, FieldKey, FileSet, parse_file};

pub struct GoParser {
    objs: AstObjects,
    error_list: ErrorList,
    file_set: FileSet,
}

impl Default for GoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl GoParser {
    #[must_use]
    pub fn new() -> Self {
        Self {
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
                let path_literal_value = import_spec.path.token.to_string();
                let mut import_node = AstNode::new("Import");
                import_node.add_child(AstNode::new(&path_literal_value));

                if let Some(alias_key) = &import_spec.name {
                    let alias_name = &objs.idents[*alias_key].name;
                    import_node.name = Some(alias_name.clone());
                }
                import_node
            }
            Spec::Value(value_spec) => {
                let mut value_node = AstNode::new("ValueSpec");
                for &name_key in &value_spec.names {
                    let name = &objs.idents[name_key].name;
                    let mut name_node = AstNode::with_name("Identifier", name);

                    if let Some(typ) = &value_spec.typ {
                        let type_node = Self::convert_expr(typ, objs);
                        name_node.add_child(type_node);
                    }
                    value_node.add_child(name_node);
                }
                for value in &value_spec.values {
                    let value_node_child = Self::convert_expr(value, objs);
                    value_node.add_child(value_node_child);
                }
                value_node
            }
            Spec::Type(type_spec) => {
                let type_name = &objs.idents[type_spec.name].name;
                let mut type_node = AstNode::with_name("TypeSpec", type_name);
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
                let function_name = &objs.idents[func.name].name;
                let mut func_node = AstNode::with_name("Function", function_name);

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
            let field_name = if field.names.is_empty() {
                "Unnamed"
            } else {
                &objs.idents[field.names[0]].name
            };
            let parameter_node = AstNode::new(&format!("Parameter: {field_name}"));
            params_node.add_child(parameter_node);
        }
        params_node
    }

    #[allow(clippy::too_many_lines)]
    fn convert_stmt(stmt: &Stmt, objs: &AstObjects) -> AstNode {
        match stmt {
            Stmt::Decl(decl) => Self::convert_decl(decl, objs),
            Stmt::Expr(expr) => Self::convert_expr(expr, objs),
            Stmt::Assign(assign_key) => {
                let assign = &objs.a_stmts[*assign_key];
                let mut assign_node = AstNode::new("Assign");
                for expr in &assign.lhs {
                    let lhs_node = Self::convert_expr(expr, objs);
                    assign_node.add_child(lhs_node);
                }
                for expr in &assign.rhs {
                    let rhs_node = Self::convert_expr(expr, objs);
                    assign_node.add_child(rhs_node);
                }
                assign_node
            }
            Stmt::Empty(_) => AstNode::new("Empty"),
            Stmt::Labeled(labeled_key) => {
                let labeled = &objs.l_stmts[*labeled_key];
                let mut labeled_node = AstNode::new("Labeled");
                let label_name = &objs.idents[labeled.label].name;
                labeled_node.add_child(AstNode::with_name("Label", label_name));
                let stmt_node = Self::convert_stmt(&labeled.stmt, objs);
                labeled_node.add_child(stmt_node);
                labeled_node
            }
            Stmt::Send(send_stmt) => {
                let mut send_node = AstNode::new("Send");
                send_node.add_child(Self::convert_expr(&send_stmt.chan, objs));
                send_node.add_child(Self::convert_expr(&send_stmt.val, objs));
                send_node
            }
            Stmt::IncDec(inc_dec_stmt) => {
                let mut inc_dec_node = AstNode::new("IncDec");
                inc_dec_node.add_child(Self::convert_expr(&inc_dec_stmt.expr, objs));
                inc_dec_node
            }
            Stmt::Go(go_stmt) => {
                let mut go_node = AstNode::new("Go");
                go_node.add_child(Self::convert_expr(&go_stmt.call, objs));
                go_node
            }
            Stmt::Defer(defer_stmt) => {
                let mut defer_node = AstNode::new("Defer");
                defer_node.add_child(Self::convert_expr(&defer_stmt.call, objs));
                defer_node
            }
            Stmt::Return(return_stmt) => {
                let mut return_node = AstNode::new("Return");
                for expr in &return_stmt.results {
                    return_node.add_child(Self::convert_expr(expr, objs));
                }
                return_node
            }
            Stmt::Branch(branch_stmt) => {
                let label_str = branch_stmt
                    .label
                    .as_ref()
                    .map_or("NoLabel", |label| &objs.idents[*label].name);
                AstNode::with_name("Branch", label_str)
            }
            Stmt::Block(block_stmt) => {
                let mut block_node = AstNode::new("Block");
                let inner_block_node = Self::convert_stmt_list(&block_stmt.list, objs);
                block_node.add_child(inner_block_node);
                block_node
            }
            Stmt::If(if_stmt) => {
                let mut if_node = AstNode::new("If");
                if let Some(init_stmt) = &if_stmt.init {
                    if_node.add_child(Self::convert_stmt(init_stmt, objs));
                }
                if_node.add_child(Self::convert_expr(&if_stmt.cond, objs));
                if_node.add_child(Self::convert_stmt_list(&if_stmt.body.list, objs));
                if let Some(else_stmt) = &if_stmt.els {
                    if_node.add_child(Self::convert_stmt(else_stmt, objs));
                }
                if_node
            }
            Stmt::Case(case_stmt) => {
                let mut case_node = AstNode::new("Case");
                if let Some(list) = &case_stmt.list {
                    for expr in list {
                        case_node.add_child(Self::convert_expr(expr, objs));
                    }
                }
                for stmt in &case_stmt.body {
                    case_node.add_child(Self::convert_stmt(stmt, objs));
                }
                case_node
            }
            Stmt::Switch(switch_stmt) => {
                let mut switch_node = AstNode::new("Switch");
                if let Some(init_stmt) = &switch_stmt.init {
                    switch_node.add_child(Self::convert_stmt(init_stmt, objs));
                }
                if let Some(tag_expr) = &switch_stmt.tag {
                    switch_node.add_child(Self::convert_expr(tag_expr, objs));
                }
                switch_node.add_child(Self::convert_stmt_list(&switch_stmt.body.list, objs));
                switch_node
            }
            Stmt::TypeSwitch(type_switch_stmt) => {
                let mut type_switch_node = AstNode::new("TypeSwitch");
                if let Some(init_stmt) = &type_switch_stmt.init {
                    type_switch_node.add_child(Self::convert_stmt(init_stmt, objs));
                }
                type_switch_node.add_child(Self::convert_stmt(&type_switch_stmt.assign, objs));
                type_switch_node
                    .add_child(Self::convert_stmt_list(&type_switch_stmt.body.list, objs));
                type_switch_node
            }
            Stmt::Comm(comm_stmt) => {
                let mut comm_node = AstNode::new("Comm");
                if let Some(comm) = &comm_stmt.comm {
                    comm_node.add_child(Self::convert_stmt(comm, objs));
                }
                for stmt in &comm_stmt.body {
                    comm_node.add_child(Self::convert_stmt(stmt, objs));
                }
                comm_node
            }
            Stmt::Select(select_stmt) => {
                let mut select_node = AstNode::new("Select");
                select_node.add_child(Self::convert_stmt_list(&select_stmt.body.list, objs));
                select_node
            }
            Stmt::For(for_stmt) => {
                let mut for_node = AstNode::new("For");
                if let Some(init_stmt) = &for_stmt.init {
                    for_node.add_child(Self::convert_stmt(init_stmt, objs));
                }
                if let Some(cond_expr) = &for_stmt.cond {
                    for_node.add_child(Self::convert_expr(cond_expr, objs));
                }
                if let Some(post_stmt) = &for_stmt.post {
                    for_node.add_child(Self::convert_stmt(post_stmt, objs));
                }
                for_node.add_child(Self::convert_stmt_list(&for_stmt.body.list, objs));
                for_node
            }
            Stmt::Range(range_stmt) => {
                let mut range_node = AstNode::new("Range");
                if let Some(key_expr) = &range_stmt.key {
                    range_node.add_child(Self::convert_expr(key_expr, objs));
                }
                if let Some(val_expr) = &range_stmt.val {
                    range_node.add_child(Self::convert_expr(val_expr, objs));
                }
                range_node.add_child(Self::convert_expr(&range_stmt.expr, objs));
                range_node.add_child(Self::convert_stmt_list(&range_stmt.body.list, objs));
                range_node
            }
            Stmt::Bad(_) => AstNode::new("BadStatement"),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn convert_expr(expr: &Expr, objs: &AstObjects) -> AstNode {
        match expr {
            Expr::Ident(ident_key) => {
                let name = &objs.idents[*ident_key].name;
                AstNode::with_name("Ident", name)
            }
            Expr::BasicLit(basic_lit) => {
                let literal_value = basic_lit.token.to_string();
                AstNode::with_name("BasicLit", &literal_value)
            }
            Expr::FuncLit(func_lit) => {
                let mut func_node = AstNode::new("FuncLit");
                let func_type = &objs.ftypes[func_lit.typ];

                if let Some(func_type_result) = &func_type.results {
                    let result_node = Self::convert_params(func_type_result, objs);
                    func_node.add_child(result_node);
                }

                let body_node = Self::convert_stmt_list(&func_lit.body.list, objs);
                func_node.add_child(body_node);

                func_node
            }
            Expr::CompositeLit(composite_lit) => {
                let mut composite_node = AstNode::new("CompositeLit");
                if let Some(typ) = &composite_lit.typ {
                    let typ_node = Self::convert_expr(typ, objs);
                    composite_node.add_child(typ_node);
                }
                for elt in &composite_lit.elts {
                    let elt_node = Self::convert_expr(elt, objs);
                    composite_node.add_child(elt_node);
                }
                composite_node
            }
            Expr::Paren(paren_expr) => {
                let inner_expr_node = Self::convert_expr(&paren_expr.expr, objs);
                let mut paren_node = AstNode::new("Paren");
                paren_node.add_child(inner_expr_node);
                paren_node
            }
            Expr::Selector(selector_expr) => {
                let mut selector_node = AstNode::new("Selector");
                selector_node.add_child(Self::convert_expr(&selector_expr.expr, objs));
                let sel_name = &objs.idents[selector_expr.sel].name;
                selector_node.add_child(AstNode::with_name("Field", sel_name));
                selector_node
            }
            Expr::Index(index_expr) => {
                let mut index_node = AstNode::new("Index");
                index_node.add_child(Self::convert_expr(&index_expr.expr, objs));
                index_node.add_child(Self::convert_expr(&index_expr.index, objs));
                index_node
            }
            Expr::Slice(slice_expr) => {
                let mut slice_node = AstNode::new("Slice");
                slice_node.add_child(Self::convert_expr(&slice_expr.expr, objs));
                if let Some(low) = &slice_expr.low {
                    slice_node.add_child(Self::convert_expr(low, objs));
                }
                if let Some(high) = &slice_expr.high {
                    slice_node.add_child(Self::convert_expr(high, objs));
                }
                if let Some(max) = &slice_expr.max {
                    slice_node.add_child(Self::convert_expr(max, objs));
                }
                slice_node
            }
            Expr::TypeAssert(type_assert_expr) => {
                let mut type_assert_node = AstNode::new("TypeAssert");
                type_assert_node.add_child(Self::convert_expr(&type_assert_expr.expr, objs));
                if let Some(typ) = &type_assert_expr.typ {
                    type_assert_node.add_child(Self::convert_expr(typ, objs));
                }
                type_assert_node
            }
            Expr::Call(call_expr) => {
                let mut call_node = AstNode::new("Call");
                call_node.add_child(Self::convert_expr(&call_expr.func, objs));
                for arg in &call_expr.args {
                    call_node.add_child(Self::convert_expr(arg, objs));
                }
                call_node
            }
            Expr::Star(star_expr) => {
                let mut star_node = AstNode::new("Star");
                star_node.add_child(Self::convert_expr(&star_expr.expr, objs));
                star_node
            }
            Expr::Unary(unary_expr) => {
                let op = format!("{:?}", unary_expr.op);
                let mut unary_node = AstNode::with_name("Unary", &op);
                unary_node.add_child(Self::convert_expr(&unary_expr.expr, objs));
                unary_node
            }
            Expr::Binary(binary_expr) => {
                let op = format!("{:?}", binary_expr.op);
                let mut binary_node = AstNode::with_name("Binary", &op);
                binary_node.add_child(Self::convert_expr(&binary_expr.expr_a, objs));
                binary_node.add_child(Self::convert_expr(&binary_expr.expr_b, objs));
                binary_node
            }
            Expr::KeyValue(key_value_expr) => {
                let mut key_value_node = AstNode::new("KeyValue");
                key_value_node.add_child(Self::convert_expr(&key_value_expr.key, objs));
                key_value_node.add_child(Self::convert_expr(&key_value_expr.val, objs));
                key_value_node
            }
            Expr::Array(array_type) => {
                let mut array_node = AstNode::new("ArrayType");
                if let Some(len) = &array_type.len {
                    array_node.add_child(Self::convert_expr(len, objs));
                }
                array_node.add_child(Self::convert_expr(&array_type.elt, objs));
                array_node
            }
            Expr::Struct(struct_type) => {
                let mut struct_node = AstNode::new("StructType");
                for field_key in &struct_type.fields.list {
                    struct_node.add_child(Self::convert_field(*field_key, objs));
                }
                struct_node
            }
            Expr::Func(func_type_key) => {
                let func_type = &objs.ftypes[*func_type_key];
                let mut func_node = AstNode::new("FuncType");
                let params_node = Self::convert_params(&func_type.params, objs);
                func_node.add_child(params_node);
                if let Some(results) = &func_type.results {
                    let results_node = Self::convert_params(results, objs);
                    func_node.add_child(results_node);
                }
                func_node
            }
            Expr::Interface(interface_type) => {
                let mut interface_node = AstNode::new("InterfaceType");
                for field_key in &interface_type.methods.list {
                    interface_node.add_child(Self::convert_field(*field_key, objs));
                }
                interface_node
            }
            Expr::Map(map_type) => {
                let mut map_node = AstNode::new("MapType");
                map_node.add_child(Self::convert_expr(&map_type.key, objs));
                map_node.add_child(Self::convert_expr(&map_type.val, objs));
                map_node
            }
            Expr::Chan(chan_type) => {
                let mut chan_node = AstNode::new("ChanType");
                chan_node.add_child(Self::convert_expr(&chan_type.val, objs));
                chan_node
            }
            Expr::Ellipsis(ellipsis) => {
                let mut ellipsis_node = AstNode::new("Ellipsis");
                if let Some(elt) = &ellipsis.elt {
                    ellipsis_node.add_child(Self::convert_expr(elt, objs));
                }
                ellipsis_node
            }
            Expr::Bad(_) => AstNode::new("BadExpr"),
        }
    }

    fn convert_field(field_key: FieldKey, objs: &AstObjects) -> AstNode {
        let field = &objs.fields[field_key];

        let mut field_node = AstNode::new("Field");

        for name_key in &field.names {
            let name = &objs.idents[*name_key].name;
            field_node.add_child(AstNode::with_name("Name", name));
        }

        let type_node = Self::convert_expr(&field.typ, objs);
        field_node.add_child(type_node);

        if let Some(tag) = &field.tag {
            let tag_node = Self::convert_expr(tag, objs);
            field_node.add_child(tag_node);
        }

        field_node
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
                let wrapped_ast = Self::wrap_node(&ast_file, &self.objs);
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
