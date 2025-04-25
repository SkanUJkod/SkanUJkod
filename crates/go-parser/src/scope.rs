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

use super::ast::Node;
use super::map::Map;
use super::objects::{
    AssignStmtKey, AstObjects, EntityKey, FieldKey, FuncDeclKey, LabeledStmtKey,
    ScopeKey, SpecKey};
use super::position;
use std::fmt;

#[derive(Debug, Clone)]
pub enum EntityKind {
    Bad,
    Pkg,
    Con,
    Typ,
    Var,
    Fun,
    Lbl,
}

impl EntityKind {
    #[must_use]
    pub const fn kind_text(&self) -> &str {
        match self {
            Self::Bad => "bad",
            Self::Pkg => "package",
            Self::Con => "const",
            Self::Typ => "type",
            Self::Var => "var",
            Self::Fun => "func",
            Self::Lbl => "label",
        }
    }
}

#[derive(Debug, Clone)]
pub enum DeclObj {
    Field(FieldKey),
    Spec(SpecKey),
    FuncDecl(FuncDeclKey),
    LabeledStmt(LabeledStmtKey),
    AssignStmt(AssignStmtKey),
    NoDecl,
}

#[derive(Debug, Clone)]
pub enum EntityData {
    PkgScope(ScopeKey),
    ConIota(isize),
    NoData,
}

// An Entity describes a named language entity such as a package,
// constant, type, variable, function (incl. methods), or label.
#[derive(Debug, Clone)]
pub struct Entity {
    pub kind: EntityKind,
    pub name: String,
    pub decl: DeclObj,
    pub data: EntityData,
}

impl Entity {
    #[must_use]
    pub const fn new(kind: EntityKind, name: String, decl: DeclObj, data: EntityData) -> Self {
        Self {
            kind,
            name,
            decl,
            data,
        }
    }

    #[must_use]
    pub const fn with_no_data(kind: EntityKind, name: String, decl: DeclObj) -> Self {
        Self::new(kind, name, decl, EntityData::NoData)
    }

    #[must_use]
    pub fn pos(&self, objs: &AstObjects) -> position::Pos {
        match &self.decl {
            DeclObj::Field(i) => i.pos(objs),
            DeclObj::Spec(i) => objs.specs[*i].pos(objs),
            DeclObj::FuncDecl(i) => objs.fdecls[*i].pos(objs),
            DeclObj::LabeledStmt(i) => objs.l_stmts[*i].pos(objs),
            DeclObj::AssignStmt(i) => objs.a_stmts[*i].pos(objs),
            DeclObj::NoDecl => 0,
        }
    }
}

pub struct Scope {
    pub outer: Option<ScopeKey>,
    pub entities: Map<String, EntityKey>,
}

impl Scope {
    #[must_use]
    pub fn new(outer: Option<ScopeKey>) -> Self {
        Self {
            outer,
            entities: Map::new(),
        }
    }

    #[must_use]
    pub fn look_up(&self, name: &String) -> Option<&EntityKey> {
        self.entities.get(name)
    }

    pub fn insert(&mut self, name: String, entity: EntityKey) -> Option<EntityKey> {
        self.entities.insert(name, entity)
    }

    /// Formats the scope for debugging purposes.
    ///
    /// # Errors
    ///
    /// This function can return an error if there is an issue writing to the formatter.
    pub fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "scope {self:p} {{")?;
        for k in self.entities.keys() {
            writeln!(f, "\t{k}")?;
        }
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod test {}
