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

#[cfg(feature = "serde_borsh")]
use borsh::{
    maybestd::io::Result as BorshResult, maybestd::io::Write as BorshWrite, BorshDeserialize,
    BorshSerialize,
};
use std::borrow::Borrow;
use std::fmt;
use std::fmt::Write;
use std::rc::Rc;

pub type Pos = usize;

#[derive(Clone, Debug)]
pub struct FilePos {
    pub filename: Rc<String>,
    pub offset: usize, // offset in utf8 char
    pub line: usize,
    pub column: usize,
}

impl FilePos {
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.line > 0
    }

    #[must_use]
    pub fn null() -> Self {
        Self {
            filename: Rc::new("[null_file]".to_owned()),
            line: 0,
            offset: 0,
            column: 0,
        }
    }
}

impl fmt::Display for FilePos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::clone(&*self.filename);
        if self.is_valid() {
            if !s.is_empty() {
                s.push(':');
            }
            s.push_str(&self.line.to_string());
        }
        if self.column != 0 {
            write!(&mut s, ":{}", self.column)?;
        }
        if s.is_empty() {
            s.push('-');
        }
        f.write_str(&s)
    }
}

#[derive(Debug)]
pub struct File {
    name: Rc<String>,
    base: usize,
    size: usize,
    lines: Vec<usize>,
}

impl File {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name: Rc::new(name),
            base: 0,
            size: 0,
            lines: vec![0],
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn base(&self) -> usize {
        self.base
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn size(&self) -> usize {
        self.size
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn add_line(&mut self, offset: usize) {
        let i = self.line_count();
        if (i == 0 || self.lines[i - 1] < offset) && offset < self.size {
            self.lines.push(offset);
        }
    }

    /// Merges the specified line in the list of lines.
    ///
    /// # Panics
    ///
    /// Panics if `line` is less than 1, since line numbering starts at 1.
    /// Panics if `line` is greater than or equal to the number of lines,
    /// indicating an invalid line index.
    pub fn merge_line(&mut self, line: usize) {
        assert!((line >= 1), "illegal line number (line numbering starts at 1)");
        assert!((line < self.line_count()), "illegal line number");
        /*
        let mut shalf = self.lines.split_off(line);
        self.lines.pop().unwrap();
        self.lines.append(&mut shalf);
        */
        let lines = &self.lines;
        self.lines = lines
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != line)
            .map(|(_, l)| *l)
            .collect();
    }

    pub fn set_lines(&mut self, lines: Vec<usize>) -> bool {
        let size = self.size;
        for (i, &offset) in self.lines.iter().enumerate() {
            if (i == 0 && size <= offset) || offset < lines[i - 1] {
                return false;
            }
        }
        self.lines = lines;
        true
    }

    pub fn set_lines_for_content(&mut self, content: &mut std::str::Chars) {
        let (mut new_line, mut line) = (true, 0);
        for (offset, b) in content.enumerate() {
            if new_line {
                self.lines.push(line);
            }
            new_line = false;
            if b == '\n' {
                new_line = true;
                line = offset + 1;
            }
        }
    }

    /// Returns the byte offset at which the specified line starts.
    ///
    /// # Panics
    ///
    /// Panics if `line` is less than 1, since line numbering starts at 1.
    /// Panics if `line` is greater than or equal to the number of lines.
    #[must_use]
    pub fn line_start(&self, line: usize) -> usize {
        assert!((line >= 1), "illegal line number (line numbering starts at 1)");
        assert!((line < self.line_count()), "illegal line number");
        self.base + self.lines[line - 1]
    }

    /// Returns the position in the file corresponding to the given offset.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is greater than the file size.
    #[must_use]
    pub fn pos(&self, offset: usize) -> Pos {
        assert!((offset <= self.size()), "illegal file offset");
        self.base() + offset
    }

    /// Returns the `FilePos` (line, column, offset) corresponding to a `Pos`.
    ///
    /// # Panics
    ///
    /// Panics if `p` is not within the range of valid positions for this file,
    #[must_use]
    pub fn position(&self, p: Pos) -> FilePos {
        assert!(!(p < self.base || p > self.base + self.size), "illegal Pos value");

        let line_count = self.line_count();
        let offset = p - self.base;
        let line = match self
            .lines
            .iter()
            .enumerate()
            .find(|&(_, &line)| line > offset)
        {
            Some((i, _)) => i,
            None => line_count,
        };
        let column = offset - self.lines[line - 1] + 1;

        FilePos {
            filename: self.name.clone(),
            line,
            offset,
            column,
        }
    }
}

#[cfg(feature = "serde_borsh")]
impl BorshSerialize for File {
    #[inline]
    fn serialize<W: BorshWrite>(&self, writer: &mut W) -> BorshResult<()> {
        let name_str: &str = &*self.name;
        name_str.serialize(writer)?;
        self.base.serialize(writer)?;
        self.size.serialize(writer)?;
        self.lines.serialize(writer)
    }
}

#[cfg(feature = "serde_borsh")]
impl BorshDeserialize for File {
    #[inline]
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> BorshResult<Self> {
        let name = String::deserialize_reader(reader)?;
        let base = usize::deserialize_reader(reader)?;
        let size = usize::deserialize_reader(reader)?;
        let lines = Vec::<usize>::deserialize_reader(reader)?;
        Ok(File {
            name: Rc::new(name),
            base,
            size,
            lines,
        })
    }
}

#[cfg_attr(feature = "serde_borsh", derive(BorshDeserialize, BorshSerialize))]
#[derive(Debug)]
pub struct FileSet {
    base: usize,
    files: Vec<File>,
}

impl Default for FileSet {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSet {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base: 0,
            files: vec![],
        }
    }

    #[must_use]
    pub const fn base(&self) -> usize {
        self.base
    }

    #[allow(clippy::iter_without_into_iter)]
    #[must_use]
    pub const fn iter(&self) -> FileSetIter {
        FileSetIter { fs: self, cur: 0 }
    }

    #[must_use]
    pub fn file(&self, p: Pos) -> Option<&File> {
        self.files.iter().find(|&f| f.base <= p && f.base + f.size >= p)
    }

    #[must_use]
    pub fn position(&self, p: Pos) -> Option<FilePos> {
        self.file(p).map(|f| f.position(p))
    }

    pub fn index_file(&mut self, i: usize) -> Option<&mut File> {
        if i >= self.files.len() {
            None
        } else {
            Some(&mut self.files[i])
        }
    }

    pub fn recent_file(&mut self) -> Option<&mut File> {
        let c = self.files.len();
        if c == 0 {
            None
        } else {
            self.index_file(c - 1)
        }
    }

    /// Adds a file with the specified name, base offset, and size.
    ///
    /// # Panics
    ///
    /// Panics if the `real_base` is less than the current base, or if adding
    /// the file causes an overflow in the base offset.
    pub fn add_file(&mut self, name: String, base: Option<usize>, size: usize) -> &mut File {
        let real_base = if let Some(b) = base { b } else { self.base };
        assert!((real_base >= self.base), "illegal base");

        let mut f = File::new(name);
        f.base = real_base;
        f.size = size;
        let set_base = self.base + size + 1; // +1 because EOF also has a position
        assert!((set_base >= self.base), "token.Pos offset overflow (> 2G of source code in file set)");
        self.base = set_base;
        self.files.push(f);
        self.recent_file().unwrap()
    }
}

pub struct FileSetIter<'a> {
    fs: &'a FileSet,
    cur: usize,
}

impl<'a> Iterator for FileSetIter<'a> {
    type Item = &'a File;

    fn next(&mut self) -> Option<&'a File> {
        if self.cur < self.fs.files.len() {
            self.cur += 1;
            Some(self.fs.files[self.cur - 1].borrow())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_position() {
        let p = FilePos {
            filename: Rc::new("test.gs".to_owned()),
            offset: 0,
            line: 54321,
            column: 8,
        };
        print!("this is the position: {} ", p);
        let mut fs = FileSet::new();
        let mut f = File::new("test.gs".to_owned());
        f.size = 12345;
        f.add_line(123);
        f.add_line(133);
        f.add_line(143);
        print!("\nfile: {:?}", f);
        f.merge_line(1);
        print!("\nfile after merge: {:?}", f);

        {
            fs.add_file("testfile1.gs".to_owned(), None, 222);
            fs.add_file("testfile2.gs".to_owned(), None, 222);
            fs.add_file("testfile3.gs".to_owned(), None, 222);
            print!("\nset {:?}", fs);
        }

        for f in fs.iter() {
            print!("\nfiles in set: {:?}", f);
        }
        print!("\nfile at 100: {:?}", fs.file(100))
    }
}
