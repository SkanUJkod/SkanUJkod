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

use super::position::{File, FilePos, Pos};
use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Error {
    pub pos: FilePos,
    pub msg: String,
    pub soft: bool,
    pub by_parser: bool, // reported by parser (not type checker)
    order: usize,        // display order
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = if self.by_parser { "[Parser]" } else { "[TC]" };
        writeln!(f, "{} {}  {}", p, self.pos, self.msg)?;
        Ok(())
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Debug)]
pub struct ErrorList {
    errors: Rc<RefCell<Vec<Error>>>,
}

impl fmt::Display for ErrorList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Result: {} errors", self.errors.borrow().len())?;
        for e in self.errors.borrow().iter() {
            e.fmt(f)?;
        }
        Ok(())
    }
}

impl std::error::Error for ErrorList {}

impl Default for ErrorList {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorList {
    #[must_use]
    pub fn new() -> ErrorList {
        ErrorList {
            errors: Rc::new(RefCell::new(vec![])),
        }
    }

    /// Adds a new error to the error list.
    ///
    /// # Panics
    ///
    /// Panics if `msg` starts with a tab character (`'\t'`) and there is no
    /// existing error in the list whose message does **not** start with a tab.
    /// This happens when trying to determine the insertion order based on
    /// previous non-indented messages.
    pub fn add(&self, p: Option<FilePos>, msg: String, soft: bool, by_parser: bool) {
        let fp = p.unwrap_or_else(FilePos::null);
        let order = if msg.starts_with('\t') {
            self.errors
                .borrow()
                .iter()
                .rev()
                .find(|x| !x.msg.starts_with('\t'))
                .unwrap()
                .pos
                .offset
        } else {
            fp.offset
        };
        self.errors.borrow_mut().push(Error {
            pos: fp,
            msg,
            soft,
            by_parser,
            order,
        });
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.errors.borrow().len()
    }

    pub fn sort(&self) {
        self.errors.borrow_mut().sort_by_key(|e| e.order);
    }

    #[must_use]
    pub fn borrow(&self) -> Ref<Vec<Error>> {
        self.errors.borrow()
    }
}

#[derive(Clone, Debug)]
pub struct FilePosErrors<'a> {
    file: &'a File,
    elist: &'a ErrorList,
}

impl<'a> FilePosErrors<'a> {
    #[must_use]
    pub fn new(file: &'a File, elist: &'a ErrorList) -> FilePosErrors<'a> {
        FilePosErrors {
            file,
            elist,
        }
    }

    pub fn add(&self, pos: Pos, msg: String, soft: bool) {
        let p = self.file.position(pos);
        self.elist.add(Some(p), msg, soft, false);
    }

    pub fn add_str(&self, pos: Pos, s: &str, soft: bool) {
        self.add(pos, s.to_string(), soft);
    }

    pub fn parser_add(&self, pos: Pos, msg: String) {
        let p = self.file.position(pos);
        self.elist.add(Some(p), msg, false, true);
    }

    pub fn parser_add_str(&self, pos: Pos, s: &str) {
        self.parser_add(pos, s.to_string());
    }
}
