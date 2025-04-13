// Copyright 2022 The Goscript Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
//
//
// This code is adapted from the official Go code written in Go
// with license as follows:
// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use super::objects::{
    AssignStmtKey, AstObjects, EntityKey, FieldKey, FuncDeclKey, FuncTypeKey,
    IdentKey, LabeledStmtKey, ScopeKey, SpecKey};
use super::position;
use super::scope;
use super::token;
use std::hash::Hash;
use std::rc::Rc;
use std::ptr;

/// `NodeId` can be used as key of `HashMaps`
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug)]
pub enum NodeId {
    Address(usize),
    IdentExpr(IdentKey),
    FuncTypeExpr(FuncTypeKey),
    LabeledStmt(LabeledStmtKey),
    AssignStmt(AssignStmtKey),
    FuncDecl(FuncDeclKey),
    FuncType(FuncTypeKey),
    Field(FieldKey),
    File(IdentKey),
}

pub trait Node {
    fn pos(&self, objs: &AstObjects) -> position::Pos;

    fn end(&self, objs: &AstObjects) -> position::Pos;

    fn id(&self) -> NodeId;
}

#[derive(Clone, Debug)]
pub enum Expr {
    Bad(Rc<BadExpr>),
    Ident(IdentKey),
    Ellipsis(Rc<Ellipsis>),
    BasicLit(Rc<BasicLit>),
    FuncLit(Rc<FuncLit>),
    CompositeLit(Rc<CompositeLit>),
    Paren(Rc<ParenExpr>),
    Selector(Rc<SelectorExpr>),
    Index(Rc<IndexExpr>),
    Slice(Rc<SliceExpr>),
    TypeAssert(Rc<TypeAssertExpr>),
    Call(Rc<CallExpr>),
    Star(Rc<StarExpr>),
    Unary(Rc<UnaryExpr>),
    Binary(Rc<BinaryExpr>),
    KeyValue(Rc<KeyValueExpr>),
    Array(Rc<ArrayType>),
    Struct(Rc<StructType>),
    Func(FuncTypeKey),
    Interface(Rc<InterfaceType>),
    Map(Rc<MapType>),
    Chan(Rc<ChanType>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Bad(Rc<BadStmt>),
    Decl(Rc<Decl>),
    Empty(Rc<EmptyStmt>),
    Labeled(LabeledStmtKey),
    Expr(Rc<Expr>),
    Send(Rc<SendStmt>),
    IncDec(Rc<IncDecStmt>),
    Assign(AssignStmtKey),
    Go(Rc<GoStmt>),
    Defer(Rc<DeferStmt>),
    Return(Rc<ReturnStmt>),
    Branch(Rc<BranchStmt>),
    Block(Rc<BlockStmt>),
    If(Rc<IfStmt>),
    Case(Rc<CaseClause>),
    Switch(Rc<SwitchStmt>),
    TypeSwitch(Rc<TypeSwitchStmt>),
    Comm(Rc<CommClause>),
    Select(Rc<SelectStmt>),
    For(Rc<ForStmt>),
    Range(Rc<RangeStmt>),
}

#[derive(Clone, Debug)]
pub enum Spec {
    Import(Rc<ImportSpec>),
    Value(Rc<ValueSpec>),
    Type(Rc<TypeSpec>),
}

#[derive(Clone, Debug)]
pub enum Decl {
    Bad(Rc<BadDecl>),
    Gen(Rc<GenDecl>),
    Func(FuncDeclKey),
}

impl Expr {
    #[must_use]
    pub fn new_bad(from: position::Pos, to: position::Pos) -> Self {
        Self::Bad(Rc::new(BadExpr { from, to }))
    }

    #[must_use]
    pub fn new_selector(x: Self, sel: IdentKey) -> Self {
        Self::Selector(Rc::new(SelectorExpr { expr: x, sel }))
    }

    #[must_use]
    pub fn new_ellipsis(pos: position::Pos, x: Option<Self>) -> Self {
        Self::Ellipsis(Rc::new(Ellipsis { pos, elt: x }))
    }

    #[must_use]
    pub fn new_basic_lit(pos: position::Pos, token: token::Token) -> Self {
        Self::BasicLit(Rc::new(BasicLit {
            pos,
            token,
        }))
    }

    #[must_use]
    pub fn new_unary_expr(pos: position::Pos, op: token::Token, expr: Self) -> Self {
        Self::Unary(Rc::new(UnaryExpr {
            op_pos: pos,
            op,
            expr,
        }))
    }

    pub fn box_func_type(ft: FuncType, objs: &mut AstObjects) -> Self {
        Self::Func(objs.ftypes.insert(ft))
    }

    #[must_use]
    pub const fn clone_ident(&self) -> Option<Self> {
        if let Self::Ident(i) = self {
            Some(Self::Ident(*i))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn try_as_ident(&self) -> Option<&IdentKey> {
        if let Self::Ident(ident) = self {
            Some(ident)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn is_bad(&self) -> bool {
        matches!(self, Self::Bad(_))
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_type_switch_assert(&self) -> bool {
        if let Self::TypeAssert(t) = self {
            t.typ.is_none()
        } else {
            false
        }
    }
}

impl Node for Expr {
    fn pos(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Bad(e) => e.from,
            Self::Ident(e) => objs.idents[*e].pos,
            Self::Ellipsis(e) => e.pos,
            Self::BasicLit(e) => e.pos,
            Self::FuncLit(e) => {
                let typ = &objs.ftypes[e.typ];
                typ.func.map_or_else(
                    || typ.params.pos(objs),
                    |p| p,
                )
            }
            Self::CompositeLit(e) => e.typ.as_ref().map_or_else(
                || e.l_brace,
                |expr| expr.pos(objs),
            ),
            Self::Paren(e) => e.l_paren,
            Self::Selector(e) => e.expr.pos(objs),
            Self::Index(e) => e.expr.pos(objs),
            Self::Slice(e) => e.expr.pos(objs),
            Self::TypeAssert(e) => e.expr.pos(objs),
            Self::Call(e) => e.func.pos(objs),
            Self::Star(e) => e.star,
            Self::Unary(e) => e.op_pos,
            Self::Binary(e) => e.expr_a.pos(objs),
            Self::KeyValue(e) => e.key.pos(objs),
            Self::Array(e) => e.l_brack,
            Self::Struct(e) => e.struct_pos,
            Self::Func(e) => e.pos(objs),
            Self::Interface(e) => e.interface,
            Self::Map(e) => e.map,
            Self::Chan(e) => e.begin,
        }
    }

    fn end(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Bad(e) => e.to,
            Self::Ident(e) => objs.idents[*e].end(),
            Self::Ellipsis(e) => e.elt.as_ref().map_or_else(
                || e.pos + 3,
                |expr| expr.end(objs),
            ),
            Self::BasicLit(e) => e.pos + e.token.get_literal().len(),
            Self::FuncLit(e) => e.body.end(),
            Self::CompositeLit(e) => e.r_brace + 1,
            Self::Paren(e) => e.r_paren + 1,
            Self::Selector(e) => objs.idents[e.sel].end(),
            Self::Index(e) => e.r_brack + 1,
            Self::Slice(e) => e.r_brack + 1,
            Self::TypeAssert(e) => e.r_paren + 1,
            Self::Call(e) => e.r_paren + 1,
            Self::Star(e) => e.expr.end(objs),
            Self::Unary(e) => e.expr.end(objs),
            Self::Binary(e) => e.expr_b.end(objs),
            Self::KeyValue(e) => e.val.end(objs),
            Self::Array(e) => e.elt.end(objs),
            Self::Struct(e) => e.fields.end(objs),
            Self::Func(e) => e.end(objs),
            Self::Interface(e) => e.methods.end(objs),
            Self::Map(e) => e.val.end(objs),
            Self::Chan(e) => e.val.end(objs),
        }
    }

    fn id(&self) -> NodeId {
        match &self {
            Self::Bad(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Ident(e) => NodeId::IdentExpr(*e),
            Self::Ellipsis(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::BasicLit(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::FuncLit(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::CompositeLit(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Paren(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Selector(e) => e.id(),
            Self::Index(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Slice(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::TypeAssert(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Call(e) => e.id(),
            Self::Star(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Unary(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Binary(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::KeyValue(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Array(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Struct(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Func(e) => NodeId::FuncTypeExpr(*e),
            Self::Interface(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Map(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Chan(e) => NodeId::Address(Rc::as_ptr(e) as usize),
        }
    }
}

impl Stmt {
    #[must_use]
    pub fn new_bad(from: position::Pos, to: position::Pos) -> Self {
        Self::Bad(Rc::new(BadStmt { from, to }))
    }

    pub fn new_assign(
        objs: &mut AstObjects,
        lhs: Vec<Expr>,
        tpos: position::Pos,
        tok: token::Token,
        rhs: Vec<Expr>,
    ) -> Self {
        Self::Assign(AssignStmt::arena_new(objs, lhs, tpos, tok, rhs))
    }

    #[must_use]
    pub fn box_block(block: BlockStmt) -> Self {
        Self::Block(Rc::new(block))
    }
}

impl Node for Stmt {
    fn pos(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Bad(s) => s.from,
            Self::Decl(d) => d.pos(objs),
            Self::Empty(s) => s.semi,
            Self::Labeled(s) => {
                let label = objs.l_stmts[*s].label;
                objs.idents[label].pos
            }
            Self::Expr(e) => e.pos(objs),
            Self::Send(s) => s.chan.pos(objs),
            Self::IncDec(s) => s.expr.pos(objs),
            Self::Assign(s) => {
                let assign = &objs.a_stmts[*s];
                assign.pos(objs)
            }
            Self::Go(s) => s.go,
            Self::Defer(s) => s.defer,
            Self::Return(s) => s.ret,
            Self::Branch(s) => s.token_pos,
            Self::Block(s) => s.pos(),
            Self::If(s) => s.if_pos,
            Self::Case(s) => s.case,
            Self::Switch(s) => s.switch,
            Self::TypeSwitch(s) => s.switch,
            Self::Comm(s) => s.case,
            Self::Select(s) => s.select,
            Self::For(s) => s.for_pos,
            Self::Range(s) => s.for_pos,
        }
    }
    fn end(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Bad(s) => s.to,
            Self::Decl(d) => d.end(objs),
            Self::Empty(s) => {
                if s.implicit {
                    s.semi
                } else {
                    s.semi + 1
                }
            }
            Self::Labeled(s) => {
                let ls = &objs.l_stmts[*s];
                ls.stmt.end(objs)
            }
            Self::Expr(e) => e.end(objs),
            Self::Send(s) => s.val.end(objs),
            Self::IncDec(s) => s.token_pos + 2,
            Self::Assign(s) => {
                let assign = &objs.a_stmts[*s];
                assign.rhs[assign.rhs.len() - 1].end(objs)
            }
            Self::Go(s) => s.call.end(objs),
            Self::Defer(s) => s.call.end(objs),
            Self::Return(s) => {
                let n = s.results.len();
                if n > 0 {
                    s.results[n - 1].end(objs)
                } else {
                    s.ret + 6
                }
            }
            Self::Branch(s) => s.label.as_ref().map_or_else(
                || s.token_pos + s.token.text().len(),
                |l| objs.idents[*l].end(),
            ),
            Self::Block(s) => s.end(),
            Self::If(s) => s.els.as_ref().map_or_else(
                || s.body.end(),
                |e| e.end(objs),
            ),
            Self::Case(s) => {
                let n = s.body.len();
                if n > 0 {
                    s.body[n - 1].end(objs)
                } else {
                    s.colon + 1
                }
            }
            Self::Switch(s) => s.body.end(),
            Self::TypeSwitch(s) => s.body.end(),
            Self::Comm(s) => {
                let n = s.body.len();
                if n > 0 {
                    s.body[n - 1].end(objs)
                } else {
                    s.colon + 1
                }
            }
            Self::Select(s) => s.body.end(),
            Self::For(s) => s.body.end(),
            Self::Range(s) => s.body.end(),
        }
    }

    fn id(&self) -> NodeId {
        match &self {
            Self::Bad(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Decl(d) => NodeId::Address(Rc::as_ptr(d) as usize),
            Self::Empty(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Labeled(s) => NodeId::LabeledStmt(*s),
            Self::Expr(e) => NodeId::Address(Rc::as_ptr(e) as usize),
            Self::Send(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::IncDec(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Assign(s) => NodeId::AssignStmt(*s),
            Self::Go(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Defer(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Return(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Branch(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Block(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::If(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Case(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Switch(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::TypeSwitch(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Comm(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Select(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::For(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Range(s) => NodeId::Address(Rc::as_ptr(s) as usize),
        }
    }
}

impl Node for Spec {
    fn pos(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Import(s) => s.name.as_ref().map_or_else(
                || s.path.pos,
                |i| objs.idents[*i].pos,
            ),
            Self::Value(s) => objs.idents[s.names[0]].pos,
            Self::Type(s) => objs.idents[s.name].pos,
        }
    }

    fn end(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Import(s) => s.end_pos.map_or_else(
                || s.path.pos,
                |p| p,
            ),
            Self::Value(s) => {
                let n = s.values.len();
                if n > 0 {
                    s.values[n - 1].end(objs)
                } else {
                    s.typ.as_ref().map_or_else(
                        || objs.idents[s.names[s.names.len() - 1]].end(),
                        |t| t.end(objs),
                    )
                }
            }
            Self::Type(t) => t.typ.end(objs),
        }
    }

    fn id(&self) -> NodeId {
        match self {
            Self::Import(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Value(s) => NodeId::Address(Rc::as_ptr(s) as usize),
            Self::Type(s) => NodeId::Address(Rc::as_ptr(s) as usize),
        }
    }
}

impl Node for Decl {
    fn pos(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Bad(d) => d.from,
            Self::Gen(d) => d.token_pos,
            Self::Func(d) => objs.fdecls[*d].pos(objs),
        }
    }

    fn end(&self, objs: &AstObjects) -> position::Pos {
        match &self {
            Self::Bad(d) => d.to,
            Self::Gen(d) => d.r_paren.as_ref().map_or_else(
                || objs.specs[d.specs[0]].end(objs),
                |p| p + 1,
            ),
            Self::Func(d) => {
                let fd = &objs.fdecls[*d];
                fd.body.as_ref().map_or_else(
                    || fd.typ.end(objs),
                    |b| b.end(),
                )
            }
        }
    }

    fn id(&self) -> NodeId {
        match self {
            Self::Bad(d) => NodeId::Address(Rc::as_ptr(d) as usize),
            Self::Gen(d) => NodeId::Address(Rc::as_ptr(d) as usize),
            Self::Func(d) => NodeId::FuncDecl(*d),
        }
    }
}

#[derive(Debug)]
pub struct File {
    pub package: position::Pos,
    pub name: IdentKey,
    pub decls: Vec<Decl>,
    pub scope: ScopeKey,
    pub imports: Vec<SpecKey>, //ImportSpec
    pub unresolved: Vec<IdentKey>,
}

impl Node for File {
    fn pos(&self, _arena: &AstObjects) -> position::Pos {
        self.package
    }

    fn end(&self, objs: &AstObjects) -> position::Pos {
        let n = self.decls.len();
        if n > 0 {
            self.decls[n - 1].end(objs)
        } else {
            objs.idents[self.name].end()
        }
    }

    fn id(&self) -> NodeId {
        NodeId::File(self.name)
    }
}

// pub struct Package {
//     name: String,
//     scope: ScopeKey,
//     imports: Map<String, EntityKey>,
//     files: Map<String, Box<File>>,
// }

// A BadExpr node is a placeholder for expressions containing
// syntax errors for which no correct expression nodes can be
// created.
#[derive(Debug)]
pub struct BadExpr {
    pub from: position::Pos,
    pub to: position::Pos,
}

#[derive(Debug, Clone)]
pub enum IdentEntity {
    NoEntity,
    Sentinel,
    Entity(EntityKey),
}

impl IdentEntity {
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::NoEntity)
    }
}

/// Checks if the given string is exported, i.e., its first character is uppercase.
///
/// # Panics
///
/// This function panics if the input string `s` is empty, as calling `unwrap()` on an empty iterator will panic.
#[must_use] pub fn is_exported(s: &str) -> bool {
    s.chars().next().unwrap().is_uppercase()
}

// An Ident node represents an identifier.
#[derive(Debug, Clone)]
pub struct Ident {
    pub pos: position::Pos,
    pub name: String,
    pub entity: IdentEntity,
}

impl Ident {
    #[must_use]
    pub fn blank(pos: position::Pos) -> Self {
        Self::with_str(pos, "_")
    }

    #[must_use]
    pub fn true_(pos: position::Pos) -> Self {
        Self::with_str(pos, "true")
    }

    #[must_use]
    pub fn with_str(pos: position::Pos, s: &str) -> Self {
        Self {
            pos,
            name: s.to_owned(),
            entity: IdentEntity::NoEntity,
        }
    }

    #[must_use]
    pub fn end(&self) -> position::Pos {
        self.pos + self.name.len()
    }

    #[must_use]
    pub fn entity_obj<'a>(&self, objs: &'a AstObjects) -> Option<&'a scope::Entity> {
        match self.entity {
            IdentEntity::Entity(i) => Some(&objs.entities[i]),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_blank(&self) -> bool {
        &self.name == "_"
    }

    #[must_use]
    pub fn is_exported(&self) -> bool {
        is_exported(&self.name)
    }
}

// An Ellipsis node stands for the "..." type in a
// parameter list or the "..." length in an array type.
#[derive(Debug)]
pub struct Ellipsis {
    pub pos: position::Pos,
    pub elt: Option<Expr>, // ellipsis element type (parameter lists only)
}

// A BasicLit node represents a literal of basic type.
#[derive(Debug)]
pub struct BasicLit {
    pub pos: position::Pos,
    pub token: token::Token,
}

// A FuncLit node represents a function literal.
#[derive(Debug)]
pub struct FuncLit {
    pub typ: FuncTypeKey,
    pub body: Rc<BlockStmt>,
}

// A CompositeLit node represents a composite literal.
#[derive(Debug)]
pub struct CompositeLit {
    pub typ: Option<Expr>,
    pub l_brace: position::Pos,
    pub elts: Vec<Expr>,
    pub r_brace: position::Pos,
    pub incomplete: bool,
}

// A ParenExpr node represents a parenthesized expression.
#[derive(Debug)]
pub struct ParenExpr {
    pub l_paren: position::Pos,
    pub expr: Expr,
    pub r_paren: position::Pos,
}
// A SelectorExpr node represents an expression followed by a selector.
#[derive(Debug)]
pub struct SelectorExpr {
    pub expr: Expr,
    pub sel: IdentKey,
}

impl SelectorExpr {
    #[must_use]
    pub fn id(&self) -> NodeId {
        NodeId::Address(ptr::from_ref::<Self>(self) as usize)
    }
}

// An IndexExpr node represents an expression followed by an index.
#[derive(Debug)]
pub struct IndexExpr {
    pub expr: Expr,
    pub l_brack: position::Pos,
    pub index: Expr,
    pub r_brack: position::Pos,
}

// An SliceExpr node represents an expression followed by slice indices.
#[derive(Debug)]
pub struct SliceExpr {
    pub expr: Expr,
    pub l_brack: position::Pos,
    pub low: Option<Expr>,
    pub high: Option<Expr>,
    pub max: Option<Expr>,
    pub slice3: bool,
    pub r_brack: position::Pos,
}

// A TypeAssertExpr node represents an expression followed by a
// type assertion.
#[derive(Debug)]
pub struct TypeAssertExpr {
    pub expr: Expr,
    pub l_paren: position::Pos,
    pub typ: Option<Expr>,
    pub r_paren: position::Pos,
}

// A CallExpr node represents an expression followed by an argument list.
#[derive(Debug)]
pub struct CallExpr {
    pub func: Expr,
    pub l_paren: position::Pos,
    pub args: Vec<Expr>,
    pub ellipsis: Option<position::Pos>,
    pub r_paren: position::Pos,
}

impl CallExpr {
    #[must_use]
    pub fn id(&self) -> NodeId {
         NodeId::Address(ptr::from_ref::<Self>(self) as usize)
    }
}

// A StarExpr node represents an expression of the form "*" Expression.
// Semantically it could be a unary "*" expression, or a pointer type.
#[derive(Debug)]
pub struct StarExpr {
    pub star: position::Pos,
    pub expr: Expr,
}

// A UnaryExpr node represents a unary expression.
// Unary "*" expressions are represented via StarExpr nodes.
#[derive(Debug)]
pub struct UnaryExpr {
    pub op_pos: position::Pos,
    pub op: token::Token,
    pub expr: Expr,
}

// A BinaryExpr node represents a binary expression.
#[derive(Debug)]
pub struct BinaryExpr {
    pub expr_a: Expr,
    pub op_pos: position::Pos,
    pub op: token::Token,
    pub expr_b: Expr,
}

// A KeyValueExpr node represents (key : value) pairs
// in composite literals.
#[derive(Debug)]
pub struct KeyValueExpr {
    pub key: Expr,
    pub colon: position::Pos,
    pub val: Expr,
}

// An ArrayType node represents an array or slice type.
#[derive(Debug)]
pub struct ArrayType {
    pub l_brack: position::Pos,
    pub len: Option<Expr>, // Ellipsis node for [...]T array types, None for slice types
    pub elt: Expr,
}

// A StructType node represents a struct type.
#[derive(Debug)]
pub struct StructType {
    pub struct_pos: position::Pos,
    pub fields: FieldList,
    pub incomplete: bool,
}

// Pointer types are represented via StarExpr nodes.

// A FuncType node represents a function type.
#[derive(Clone, Debug)]
pub struct FuncType {
    pub func: Option<position::Pos>,
    pub params: FieldList,
    pub results: Option<FieldList>,
}

impl FuncType {
    #[must_use]
    pub const fn new(
        func: Option<position::Pos>,
        params: FieldList,
        results: Option<FieldList>,
    ) -> Self {
        Self {
            func,
            params,
            results,
        }
    }
}

impl Node for FuncTypeKey {
    fn pos(&self, objs: &AstObjects) -> position::Pos {
        let self_ = &objs.ftypes[*self];
        self_.func.map_or_else(
            || self_.params.pos(objs),
            |p| p,
        )
    }

    fn end(&self, objs: &AstObjects) -> position::Pos {
        let self_ = &objs.ftypes[*self];
        self_.results.as_ref().map_or_else(
            || self_.params.end(objs),
            |r| (*r).end(objs),
        )
    }

    fn id(&self) -> NodeId {
        NodeId::FuncType(*self)
    }
}

// An InterfaceType node represents an interface type.
#[derive(Clone, Debug)]
pub struct InterfaceType {
    pub interface: position::Pos,
    pub methods: FieldList,
    pub incomplete: bool,
}

// A MapType node represents a map type.
#[derive(Debug)]
pub struct MapType {
    pub map: position::Pos,
    pub key: Expr,
    pub val: Expr,
}

// A ChanType node represents a channel type.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ChanDir {
    Send = 1,
    Recv = 2,
    SendRecv = 3,
}

#[derive(Clone, Debug)]
pub struct ChanType {
    pub begin: position::Pos,
    pub arrow: position::Pos,
    pub dir: ChanDir,
    pub val: Expr,
}

// An ImportSpec node represents a single package import.
#[derive(Debug)]
pub struct ImportSpec {
    pub name: Option<IdentKey>,
    pub path: BasicLit,
    pub end_pos: Option<position::Pos>,
}

// A ValueSpec node represents a constant or variable declaration
// (ConstSpec or VarSpec production).
#[derive(Debug)]
pub struct ValueSpec {
    pub names: Vec<IdentKey>,
    pub typ: Option<Expr>,
    pub values: Vec<Expr>,
}

// A TypeSpec node represents a type declaration (TypeSpec production).
#[derive(Debug)]
pub struct TypeSpec {
    pub name: IdentKey,
    pub assign: position::Pos,
    pub typ: Expr,
}

#[derive(Debug)]
pub struct BadDecl {
    pub from: position::Pos,
    pub to: position::Pos,
}

// A GenDecl node (generic declaration node) represents an import,
// constant, type or variable declaration. A valid Lparen position
// (Lparen.IsValid()) indicates a parenthesized declaration.
//
// Relationship between Tok value and Specs element type:
//
//	Token::IMPORT  ImportSpec
//	Token::CONST   ValueSpec
//	Token::TYPE    TypeSpec
//	Token::VAR     ValueSpec
#[derive(Debug)]
pub struct GenDecl {
    pub token_pos: position::Pos,
    pub token: token::Token,
    pub l_paran: Option<position::Pos>,
    pub specs: Vec<SpecKey>,
    pub r_paren: Option<position::Pos>,
}

// A FuncDecl node represents a function declaration.
#[derive(Debug)]
pub struct FuncDecl {
    pub recv: Option<FieldList>,
    pub name: IdentKey,
    pub typ: FuncTypeKey,
    pub body: Option<Rc<BlockStmt>>,
}

impl FuncDecl {
    #[must_use]
    pub fn pos(&self, objs: &AstObjects) -> position::Pos {
        self.typ.pos(objs)
    }
}

#[derive(Debug)]
pub struct BadStmt {
    pub from: position::Pos,
    pub to: position::Pos,
}

#[derive(Debug)]
pub struct EmptyStmt {
    pub semi: position::Pos,
    pub implicit: bool,
}

// A LabeledStmt node represents a labeled statement.
#[derive(Debug)]
pub struct LabeledStmt {
    pub label: IdentKey,
    pub colon: position::Pos,
    pub stmt: Stmt,
}

impl LabeledStmt {
    pub fn arena_new(
        objs: &mut AstObjects,
        label: IdentKey,
        colon: position::Pos,
        stmt: Stmt,
    ) -> LabeledStmtKey {
        let l = Self {
            label,
            colon,
            stmt,
        };
        objs.l_stmts.insert(l)
    }

    #[must_use]
    pub fn pos(&self, objs: &AstObjects) -> position::Pos {
        objs.idents[self.label].pos
    }
}

// A SendStmt node represents a send statement.
#[derive(Debug)]
pub struct SendStmt {
    pub chan: Expr,
    pub arrow: position::Pos,
    pub val: Expr,
}

// An IncDecStmt node represents an increment or decrement statement.
#[derive(Debug)]
pub struct IncDecStmt {
    pub expr: Expr,
    pub token_pos: position::Pos,
    pub token: token::Token,
}

// An AssignStmt node represents an assignment or
// a short variable declaration.
#[derive(Debug)]
pub struct AssignStmt {
    pub lhs: Vec<Expr>,
    pub token_pos: position::Pos,
    pub token: token::Token,
    pub rhs: Vec<Expr>,
}

impl AssignStmt {
    pub fn arena_new(
        objs: &mut AstObjects,
        lhs: Vec<Expr>,
        tpos: position::Pos,
        tok: token::Token,
        rhs: Vec<Expr>,
    ) -> AssignStmtKey {
        let ass = Self {
            lhs,
            token_pos: tpos,
            token: tok,
            rhs,
        };
        objs.a_stmts.insert(ass)
    }

    #[must_use]
    pub fn pos(&self, objs: &AstObjects) -> position::Pos {
        self.lhs[0].pos(objs)
    }
}

#[derive(Debug)]
pub struct GoStmt {
    pub go: position::Pos,
    pub call: Expr,
}
#[derive(Debug)]
pub struct DeferStmt {
    pub defer: position::Pos,
    pub call: Expr,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub ret: position::Pos,
    pub results: Vec<Expr>,
}

// A BranchStmt node represents a break, continue, goto,
// or fallthrough statement.
#[derive(Debug)]
pub struct BranchStmt {
    pub token_pos: position::Pos,
    pub token: token::Token,
    pub label: Option<IdentKey>,
}

#[derive(Debug)]
pub struct BlockStmt {
    pub l_brace: position::Pos,
    pub list: Vec<Stmt>,
    pub r_brace: position::Pos,
}

impl BlockStmt {
    #[must_use]
    pub const fn new(l: position::Pos, list: Vec<Stmt>, r: position::Pos) -> Self {
        Self {
            l_brace: l,
            list,
            r_brace: r,
        }
    }

    #[must_use]
    pub const fn pos(&self) -> position::Pos {
        self.l_brace
    }

    #[must_use]
    pub const fn end(&self) -> position::Pos {
        self.r_brace + 1
    }
}

#[derive(Debug)]
pub struct IfStmt {
    pub if_pos: position::Pos,
    pub init: Option<Stmt>,
    pub cond: Expr,
    pub body: Rc<BlockStmt>,
    pub els: Option<Stmt>,
}

// A CaseClause represents a case of an expression or type switch statement.
#[derive(Debug)]
pub struct CaseClause {
    pub case: position::Pos,
    pub list: Option<Vec<Expr>>,
    pub colon: position::Pos,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct SwitchStmt {
    pub switch: position::Pos,
    pub init: Option<Stmt>,
    pub tag: Option<Expr>,
    pub body: Rc<BlockStmt>,
}

#[derive(Debug)]
pub struct TypeSwitchStmt {
    pub switch: position::Pos,
    pub init: Option<Stmt>,
    pub assign: Stmt,
    pub body: Rc<BlockStmt>,
}

// A CommClause node represents a case of a select statement.
#[derive(Debug)]
pub struct CommClause {
    //communication
    pub case: position::Pos,
    pub comm: Option<Stmt>,
    pub colon: position::Pos,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct SelectStmt {
    pub select: position::Pos,
    pub body: Rc<BlockStmt>,
}

#[derive(Debug)]
pub struct ForStmt {
    pub for_pos: position::Pos,
    pub init: Option<Stmt>,
    pub cond: Option<Expr>,
    pub post: Option<Stmt>,
    pub body: Rc<BlockStmt>,
}

#[derive(Debug)]
pub struct RangeStmt {
    pub for_pos: position::Pos,
    pub key: Option<Expr>,
    pub val: Option<Expr>,
    pub token_pos: position::Pos,
    pub token: token::Token,
    pub expr: Expr,
    pub body: Rc<BlockStmt>,
}

#[derive(Debug)]
pub struct Field {
    pub names: Vec<IdentKey>,
    pub typ: Expr,
    pub tag: Option<Expr>,
}

impl Node for FieldKey {
    fn pos(&self, objs: &AstObjects) -> position::Pos {
        let self_ = &objs.fields[*self];
        if self_.names.is_empty() {
            self_.typ.pos(objs)
        } else {
            objs.idents[self_.names[0]].pos
        }
    }

    fn end(&self, objs: &AstObjects) -> position::Pos {
        let self_ = &objs.fields[*self];
        self_.tag.as_ref().map_or_else(
            || self_.typ.end(objs),
            |t| t.end(objs),
        )
    }

    fn id(&self) -> NodeId {
        NodeId::Field(*self)
    }
}

#[derive(Clone, Debug)]
pub struct FieldList {
    pub opening: Option<position::Pos>,
    pub list: Vec<FieldKey>,
    pub closing: Option<position::Pos>,
}

impl FieldList {
    #[must_use]
    pub const fn new(
        opening: Option<position::Pos>,
        list: Vec<FieldKey>,
        closing: Option<position::Pos>,
    ) -> Self {
        Self {
            opening,
            list,
            closing,
        }
    }

    #[must_use]
    pub fn pos(&self, objs: &AstObjects) -> position::Pos {
        self.opening.map_or_else(
            || self.list[0].pos(objs),
            |o| o,
        )
    }

    #[must_use]
    pub fn end(&self, objs: &AstObjects) -> position::Pos {
        self.closing.map_or_else(
            || self.list[self.list.len() - 1].pos(objs),
            |c| c,
        )
    }
}
