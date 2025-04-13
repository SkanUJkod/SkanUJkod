// Copyright 2022 The Goscript Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use super::ast::{
    BadDecl, BadExpr, BadStmt, BasicLit, BlockStmt, BranchStmt, CaseClause, ChanDir,
    CommClause, CompositeLit, Decl, DeferStmt, EmptyStmt, Expr, ForStmt, FuncLit,
    GenDecl, GoStmt, IfStmt, IncDecStmt, InterfaceType, RangeStmt, ReturnStmt,
    SelectStmt, SendStmt, Stmt, StructType, SwitchStmt, TypeSwitchStmt};
use super::objects::{AssignStmtKey, FuncDeclKey, FuncTypeKey, IdentKey, LabeledStmtKey};
use super::token::Token;

pub trait ExprVisitor {
    type Result;

    fn visit_expr_ident(&mut self, this: &Expr, ident: &IdentKey) -> Self::Result;

    fn visit_expr_ellipsis(&mut self, this: &Expr, els: &Option<Expr>) -> Self::Result;

    fn visit_expr_basic_lit(&mut self, this: &Expr, blit: &BasicLit) -> Self::Result;

    fn visit_expr_func_lit(&mut self, this: &Expr, flit: &FuncLit) -> Self::Result;

    fn visit_expr_composite_lit(&mut self, this: &Expr, clit: &CompositeLit) -> Self::Result;

    fn visit_expr_paren(&mut self, this: &Expr, expr: &Expr) -> Self::Result;

    fn visit_expr_selector(&mut self, this: &Expr, expr: &Expr, ident: &IdentKey) -> Self::Result; //add: lvalue

    fn visit_expr_index(&mut self, this: &Expr, expr: &Expr, index: &Expr) -> Self::Result;

    fn visit_expr_slice(
        &mut self,
        this: &Expr,
        expr: &Expr,
        low: &Option<Expr>,
        high: &Option<Expr>,
        max: &Option<Expr>,
    ) -> Self::Result;

    fn visit_expr_type_assert(
        &mut self,
        this: &Expr,
        expr: &Expr,
        typ: &Option<Expr>,
    ) -> Self::Result;

    fn visit_expr_call(
        &mut self,
        this: &Expr,
        func: &Expr,
        args: &[Expr],
        ellipsis: bool,
    ) -> Self::Result;

    fn visit_expr_star(&mut self, this: &Expr, expr: &Expr) -> Self::Result;

    fn visit_expr_unary(&mut self, this: &Expr, expr: &Expr, op: &Token) -> Self::Result;

    fn visit_expr_binary(
        &mut self,
        this: &Expr,
        left: &Expr,
        op: &Token,
        right: &Expr,
    ) -> Self::Result;

    fn visit_expr_key_value(&mut self, this: &Expr, key: &Expr, val: &Expr) -> Self::Result;

    /// codegen needs the unwrapped expr
    fn visit_expr_array_type(
        &mut self,
        this: &Expr,
        len: &Option<Expr>,
        elm: &Expr,
    ) -> Self::Result;

    fn visit_expr_struct_type(&mut self, this: &Expr, s: &StructType) -> Self::Result;

    fn visit_expr_func_type(&mut self, this: &Expr, s: &FuncTypeKey) -> Self::Result;

    fn visit_expr_interface_type(&mut self, this: &Expr, s: &InterfaceType) -> Self::Result;

    /// codegen needs the unwrapped expr
    fn visit_map_type(&mut self, this: &Expr, key: &Expr, val: &Expr, map: &Expr) -> Self::Result;

    fn visit_chan_type(&mut self, this: &Expr, chan: &Expr, dir: &ChanDir) -> Self::Result;

    fn visit_bad_expr(&mut self, this: &Expr, e: &BadExpr) -> Self::Result;
}

pub trait StmtVisitor {
    type Result;

    fn visit_decl(&mut self, decl: &Decl) -> Self::Result;

    fn visit_stmt_decl_gen(&mut self, gdecl: &GenDecl) -> Self::Result;

    fn visit_stmt_decl_func(&mut self, fdecl: &FuncDeclKey) -> Self::Result;

    fn visit_stmt_labeled(&mut self, lstmt: &LabeledStmtKey) -> Self::Result;

    fn visit_stmt_send(&mut self, sstmt: &SendStmt) -> Self::Result;

    fn visit_stmt_incdec(&mut self, idcstmt: &IncDecStmt) -> Self::Result;

    fn visit_stmt_assign(&mut self, astmt: &AssignStmtKey) -> Self::Result;

    fn visit_stmt_go(&mut self, gostmt: &GoStmt) -> Self::Result;

    fn visit_stmt_defer(&mut self, dstmt: &DeferStmt) -> Self::Result;

    fn visit_stmt_return(&mut self, rstmt: &ReturnStmt) -> Self::Result;

    fn visit_stmt_branch(&mut self, bstmt: &BranchStmt) -> Self::Result;

    fn visit_stmt_block(&mut self, bstmt: &BlockStmt) -> Self::Result;

    fn visit_stmt_if(&mut self, ifstmt: &IfStmt) -> Self::Result;

    fn visit_stmt_case(&mut self, cclause: &CaseClause) -> Self::Result;

    fn visit_stmt_switch(&mut self, sstmt: &SwitchStmt) -> Self::Result;

    fn visit_stmt_type_switch(&mut self, tstmt: &TypeSwitchStmt) -> Self::Result;

    fn visit_stmt_comm(&mut self, cclause: &CommClause) -> Self::Result;

    fn visit_stmt_select(&mut self, sstmt: &SelectStmt) -> Self::Result;

    fn visit_stmt_for(&mut self, fstmt: &ForStmt) -> Self::Result;

    fn visit_stmt_range(&mut self, rstmt: &RangeStmt) -> Self::Result;

    fn visit_expr_stmt(&mut self, stmt: &Expr) -> Self::Result;

    fn visit_empty_stmt(&mut self, e: &EmptyStmt) -> Self::Result;

    fn visit_bad_stmt(&mut self, b: &BadStmt) -> Self::Result;

    fn visit_bad_decl(&mut self, b: &BadDecl) -> Self::Result;
}

pub fn walk_expr<R>(v: &mut dyn ExprVisitor<Result = R>, expr: &Expr) -> R {
    match expr {
        Expr::Bad(e) => v.visit_bad_expr(expr, e.as_ref()),
        Expr::Ident(e) => v.visit_expr_ident(expr, e),
        Expr::Ellipsis(e) => v.visit_expr_ellipsis(expr, &e.as_ref().elt),
        Expr::BasicLit(e) => v.visit_expr_basic_lit(expr, e.as_ref()),
        Expr::FuncLit(e) => v.visit_expr_func_lit(expr, e.as_ref()),
        Expr::CompositeLit(e) => v.visit_expr_composite_lit(expr, e.as_ref()),
        Expr::Paren(e) => v.visit_expr_paren(expr, &e.as_ref().expr),
        Expr::Selector(e) => {
            let selexp = e.as_ref();
            v.visit_expr_selector(expr, &selexp.expr, &selexp.sel)
        }
        Expr::Index(e) => {
            let indexp = e.as_ref();
            v.visit_expr_index(expr, &indexp.expr, &indexp.index)
        }
        Expr::Slice(e) => {
            let slexp = e.as_ref();
            v.visit_expr_slice(expr, &slexp.expr, &slexp.low, &slexp.high, &slexp.max)
        }
        Expr::TypeAssert(e) => {
            let taexp = e.as_ref();
            v.visit_expr_type_assert(expr, &taexp.expr, &taexp.typ)
        }
        Expr::Call(e) => {
            let callexp = e.as_ref();
            v.visit_expr_call(
                expr,
                &callexp.func,
                &callexp.args,
                callexp.ellipsis.is_some(),
            )
        }
        Expr::Star(e) => v.visit_expr_star(expr, &e.as_ref().expr),
        Expr::Unary(e) => {
            let uexp = e.as_ref();
            v.visit_expr_unary(expr, &uexp.expr, &uexp.op)
        }
        Expr::Binary(e) => {
            let bexp = e.as_ref();
            v.visit_expr_binary(expr, &bexp.expr_a, &bexp.op, &bexp.expr_b)
        }
        Expr::KeyValue(e) => {
            let kvexp = e.as_ref();
            v.visit_expr_key_value(expr, &kvexp.key, &kvexp.val)
        }
        Expr::Array(e) => v.visit_expr_array_type(expr, &e.as_ref().len, &e.as_ref().elt),
        Expr::Struct(e) => v.visit_expr_struct_type(expr, e.as_ref()),
        Expr::Func(e) => v.visit_expr_func_type(expr, e),
        Expr::Interface(e) => v.visit_expr_interface_type(expr, e.as_ref()),
        Expr::Map(e) => {
            let mexp = e.as_ref();
            v.visit_map_type(expr, &mexp.key, &mexp.val, expr)
        }
        Expr::Chan(e) => {
            let cexp = e.as_ref();
            v.visit_chan_type(expr, &cexp.val, &cexp.dir)
        }
    }
}

pub fn walk_stmt<V: StmtVisitor<Result = R> + ExprVisitor<Result = R>, R>(
    v: &mut V,
    stmt: &Stmt,
) -> R {
    match stmt {
        Stmt::Bad(bad_stmt) => v.visit_bad_stmt(bad_stmt),
        Stmt::Decl(decl) => v.visit_decl(decl),
        Stmt::Empty(empty_stmt) => v.visit_empty_stmt(empty_stmt),
        Stmt::Labeled(labeled_stmt) => v.visit_stmt_labeled(labeled_stmt),
        Stmt::Expr(expr) => v.visit_expr_stmt(expr),
        Stmt::Send(send_stmt) => v.visit_stmt_send(send_stmt),
        Stmt::IncDec(inc_dec_stmt) => v.visit_stmt_incdec(inc_dec_stmt),
        Stmt::Assign(assign_stmt) => v.visit_stmt_assign(assign_stmt),
        Stmt::Go(go_stmt) => v.visit_stmt_go(go_stmt),
        Stmt::Defer(defer_stmt) => v.visit_stmt_defer(defer_stmt),
        Stmt::Return(return_stmt) => v.visit_stmt_return(return_stmt),
        Stmt::Branch(branch_stmt) => v.visit_stmt_branch(branch_stmt),
        Stmt::Block(block_stmt) => v.visit_stmt_block(block_stmt),
        Stmt::If(if_stmt) => v.visit_stmt_if(if_stmt),
        Stmt::Case(case_clause) => v.visit_stmt_case(case_clause),
        Stmt::Switch(switch_stmt) => v.visit_stmt_switch(switch_stmt),
        Stmt::TypeSwitch(type_switch_stmt) => v.visit_stmt_type_switch(type_switch_stmt),
        Stmt::Comm(comm_clause) => v.visit_stmt_comm(comm_clause),
        Stmt::Select(select_stmt) => v.visit_stmt_select(select_stmt),
        Stmt::For(for_stmt) => v.visit_stmt_for(for_stmt),
        Stmt::Range(range_stmt) => v.visit_stmt_range(range_stmt),
    }
}

pub fn walk_decl<R>(v: &mut dyn StmtVisitor<Result = R>, decl: &Decl) -> R {
    match decl {
        Decl::Bad(b) => v.visit_bad_decl(b),
        Decl::Gen(gdecl) => v.visit_stmt_decl_gen(gdecl),
        Decl::Func(fdecl) => v.visit_stmt_decl_func(fdecl),
    }
}
