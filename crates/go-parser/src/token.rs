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

#![allow(non_camel_case_types)]
use std::fmt;

pub const LOWEST_PREC: usize = 0; // non-operators

//pub(crate) const UNARY_PREC: usize = 6;
//pub(crate) const HIGHEST_PREC: usize = 7;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum Token {
    // Special tokens
    NONE,
    ILLEGAL(TokenData),
    EOF,
    COMMENT(TokenData),

    // Identifiers and basic type literals
    IDENT(TokenData),  // main
    INT(TokenData),    // 12345
    FLOAT(TokenData),  // 123.45
    IMAG(TokenData),   // 123.45i
    CHAR(TokenData),   // 'a'
    STRING(TokenData), // "abc"
    // Operator
    ADD, // +
    SUB, // -
    MUL, // *
    QUO, // /
    REM, // %

    AND,     // &
    OR,      // |
    XOR,     // ^
    SHL,     // <<
    SHR,     // >>
    AND_NOT, // &^

    ADD_ASSIGN, // +=
    SUB_ASSIGN, // -=
    MUL_ASSIGN, // *=
    QUO_ASSIGN, // /=
    REM_ASSIGN, // %=

    AND_ASSIGN,     // &=
    OR_ASSIGN,      // |=
    XOR_ASSIGN,     // ^=
    SHL_ASSIGN,     // <<=
    SHR_ASSIGN,     // >>=
    AND_NOT_ASSIGN, // &^=

    LAND,  // &&
    LOR,   // ||
    ARROW, // <-
    INC,   // ++
    DEC,   // --

    EQL,    // ==
    LSS,    // <
    GTR,    // >
    ASSIGN, // =
    NOT,    // !

    NEQ,      // !=
    LEQ,      // <=
    GEQ,      // >=
    DEFINE,   // :=
    ELLIPSIS, // ...

    LPAREN, // (
    LBRACK, // [
    LBRACE, // {
    COMMA,  // ,
    PERIOD, // .

    RPAREN,               // )
    RBRACK,               // ]
    RBRACE,               // }
    SEMICOLON(TokenData), // ; true if SEMICOLON is NOT inserted by scanner
    COLON,                // :

    // Keywords
    BREAK,
    CASE,
    CHAN,
    CONST,
    CONTINUE,

    DEFAULT,
    DEFER,
    ELSE,
    FALLTHROUGH,
    FOR,

    FUNC,
    GO,
    GOTO,
    IF,
    IMPORT,

    INTERFACE,
    MAP,
    PACKAGE,
    RANGE,
    RETURN,

    SELECT,
    STRUCT,
    SWITCH,
    TYPE,
    VAR,
}

pub enum TokenType {
    Literal,
    Operator,
    Keyword,
    Other,
}

impl Token {
    #[must_use]
    pub const fn token_property(&self) -> (TokenType, &str) {
        match self {
            Self::NONE => (TokenType::Other, "NONE"),
            Self::ILLEGAL(_) => (TokenType::Other, "ILLEGAL"),
            Self::EOF => (TokenType::Other, "EOF"),
            Self::COMMENT(_) => (TokenType::Other, "COMMENT"),
            Self::IDENT(_) => (TokenType::Literal, "IDENT"),
            Self::INT(_) => (TokenType::Literal, "INT"),
            Self::FLOAT(_) => (TokenType::Literal, "FLOAT"),
            Self::IMAG(_) => (TokenType::Literal, "IMAG"),
            Self::CHAR(_) => (TokenType::Literal, "CHAR"),
            Self::STRING(_) => (TokenType::Literal, "STRING"),
            Self::ADD => (TokenType::Operator, "+"),
            Self::SUB => (TokenType::Operator, "-"),
            Self::MUL => (TokenType::Operator, "*"),
            Self::QUO => (TokenType::Operator, "/"),
            Self::REM => (TokenType::Operator, "%"),
            Self::AND => (TokenType::Operator, "&"),
            Self::OR => (TokenType::Operator, "|"),
            Self::XOR => (TokenType::Operator, "^"),
            Self::SHL => (TokenType::Operator, "<<"),
            Self::SHR => (TokenType::Operator, ">>"),
            Self::AND_NOT => (TokenType::Operator, "&^"),
            Self::ADD_ASSIGN => (TokenType::Operator, "+="),
            Self::SUB_ASSIGN => (TokenType::Operator, "-="),
            Self::MUL_ASSIGN => (TokenType::Operator, "*="),
            Self::QUO_ASSIGN => (TokenType::Operator, "/="),
            Self::REM_ASSIGN => (TokenType::Operator, "%="),
            Self::AND_ASSIGN => (TokenType::Operator, "&="),
            Self::OR_ASSIGN => (TokenType::Operator, "|="),
            Self::XOR_ASSIGN => (TokenType::Operator, "^="),
            Self::SHL_ASSIGN => (TokenType::Operator, "<<="),
            Self::SHR_ASSIGN => (TokenType::Operator, ">>="),
            Self::AND_NOT_ASSIGN => (TokenType::Operator, "&^="),
            Self::LAND => (TokenType::Operator, "&&"),
            Self::LOR => (TokenType::Operator, "||"),
            Self::ARROW => (TokenType::Operator, "<-"),
            Self::INC => (TokenType::Operator, "++"),
            Self::DEC => (TokenType::Operator, "--"),
            Self::EQL => (TokenType::Operator, "=="),
            Self::LSS => (TokenType::Operator, "<"),
            Self::GTR => (TokenType::Operator, ">"),
            Self::ASSIGN => (TokenType::Operator, "="),
            Self::NOT => (TokenType::Operator, "!"),
            Self::NEQ => (TokenType::Operator, "!="),
            Self::LEQ => (TokenType::Operator, "<="),
            Self::GEQ => (TokenType::Operator, ">="),
            Self::DEFINE => (TokenType::Operator, ":="),
            Self::ELLIPSIS => (TokenType::Operator, "..."),
            Self::LPAREN => (TokenType::Operator, "("),
            Self::LBRACK => (TokenType::Operator, "["),
            Self::LBRACE => (TokenType::Operator, "{"),
            Self::COMMA => (TokenType::Operator, ","),
            Self::PERIOD => (TokenType::Operator, "."),
            Self::RPAREN => (TokenType::Operator, ")"),
            Self::RBRACK => (TokenType::Operator, "]"),
            Self::RBRACE => (TokenType::Operator, "}"),
            Self::SEMICOLON(_) => (TokenType::Operator, ";"),
            Self::COLON => (TokenType::Operator, ":"),
            Self::BREAK => (TokenType::Keyword, "break"),
            Self::CASE => (TokenType::Keyword, "case"),
            Self::CHAN => (TokenType::Keyword, "chan"),
            Self::CONST => (TokenType::Keyword, "const"),
            Self::CONTINUE => (TokenType::Keyword, "continue"),
            Self::DEFAULT => (TokenType::Keyword, "default"),
            Self::DEFER => (TokenType::Keyword, "defer"),
            Self::ELSE => (TokenType::Keyword, "else"),
            Self::FALLTHROUGH => (TokenType::Keyword, "fallthrough"),
            Self::FOR => (TokenType::Keyword, "for"),
            Self::FUNC => (TokenType::Keyword, "func"),
            Self::GO => (TokenType::Keyword, "go"),
            Self::GOTO => (TokenType::Keyword, "goto"),
            Self::IF => (TokenType::Keyword, "if"),
            Self::IMPORT => (TokenType::Keyword, "import"),
            Self::INTERFACE => (TokenType::Keyword, "interface"),
            Self::MAP => (TokenType::Keyword, "map"),
            Self::PACKAGE => (TokenType::Keyword, "package"),
            Self::RANGE => (TokenType::Keyword, "range"),
            Self::RETURN => (TokenType::Keyword, "return"),
            Self::SELECT => (TokenType::Keyword, "select"),
            Self::STRUCT => (TokenType::Keyword, "struct"),
            Self::SWITCH => (TokenType::Keyword, "switch"),
            Self::TYPE => (TokenType::Keyword, "type"),
            Self::VAR => (TokenType::Keyword, "var"),
        }
    }

    #[must_use]
    pub fn ident_token(ident: String) -> Self {
        match ident.as_str() {
            "break" => Self::BREAK,
            "case" => Self::CASE,
            "chan" => Self::CHAN,
            "const" => Self::CONST,
            "continue" => Self::CONTINUE,
            "default" => Self::DEFAULT,
            "defer" => Self::DEFER,
            "else" => Self::ELSE,
            "fallthrough" => Self::FALLTHROUGH,
            "for" => Self::FOR,
            "func" => Self::FUNC,
            "go" => Self::GO,
            "goto" => Self::GOTO,
            "if" => Self::IF,
            "import" => Self::IMPORT,
            "interface" => Self::INTERFACE,
            "map" => Self::MAP,
            "package" => Self::PACKAGE,
            "range" => Self::RANGE,
            "return" => Self::RETURN,
            "select" => Self::SELECT,
            "struct" => Self::STRUCT,
            "switch" => Self::SWITCH,
            "type" => Self::TYPE,
            "var" => Self::VAR,
            _ => Self::IDENT(ident.into()),
        }
    }

    #[must_use]
    pub fn int1() -> Self {
        Self::INT("1".to_owned().into())
    }

    #[must_use]
    pub const fn precedence(&self) -> usize {
        match self {
            Self::LOR => 1,
            Self::LAND => 2,
            Self::EQL | Self::NEQ | Self::LSS | Self::LEQ | Self::GTR | Self::GEQ => 3,
            Self::ADD | Self::SUB | Self::OR | Self::XOR => 4,
            Self::MUL
            | Self::QUO
            | Self::REM
            | Self::SHL
            | Self::SHR
            | Self::AND
            | Self::AND_NOT => 5,
            _ => LOWEST_PREC,
        }
    }

    #[must_use]
    pub const fn text(&self) -> &str {
        let (_, t) = self.token_property();
        t
    }

    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(self.token_property().0, TokenType::Literal)
    }

    #[must_use]
    pub const fn is_operator(&self) -> bool {
        matches!(self.token_property().0, TokenType::Operator)
    }

    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        matches!(self.token_property().0, TokenType::Keyword)
    }

    #[must_use]
    pub fn get_literal(&self) -> &str {
        match self {
            Self::INT(l) | Self::FLOAT(l) | Self::IMAG(l) | Self::CHAR(l) | Self::STRING(l) => l.as_str(),
            _ => "",
        }
    }

    #[must_use]
    pub const fn is_stmt_start(&self) -> bool {
        matches!(self, Self::BREAK | Self::CONST | Self::CONTINUE | Self::DEFER | Self::FALLTHROUGH | Self::FOR | Self::GO | Self::GOTO | Self::IF | Self::RETURN | Self::SELECT | Self::SWITCH | Self::TYPE | Self::VAR)
    }

    #[must_use]
    pub const fn is_decl_start(&self) -> bool {
        matches!(self, Self::CONST | Self::TYPE | Self::VAR)
    }

    #[must_use]
    pub const fn is_expr_end(&self) -> bool {
        matches!(self, Self::COMMA | Self::COLON | Self::SEMICOLON(_) | Self::RPAREN | Self::RBRACK | Self::RBRACE)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = self.text();
        match self {
            Self::IDENT(l)
            | Self::INT(l)
            | Self::FLOAT(l)
            | Self::IMAG(l)
            | Self::CHAR(l)
            | Self::STRING(l) => f.write_str(l.as_str()),
            _ => write!(f, "{text}"),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = self.text();
        match self {
            Self::IDENT(l)
            | Self::INT(l)
            | Self::FLOAT(l)
            | Self::IMAG(l)
            | Self::CHAR(l)
            | Self::STRING(l) => write!(f, "{} {}", text, l.as_str()),
            Self::SEMICOLON(real) if !*real.as_bool() => write!(f, "\"{text}(inserted)\""),
            token if token.is_operator() || token.is_keyword() => write!(f, "\"{text}\""),
            _ => write!(f, "{text}"),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
enum RawTokenData {
    Bool(bool),
    Str(String),
    StrStr(String, String),
    StrChar(String, char),
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct TokenData(Box<RawTokenData>);

impl From<bool> for TokenData {
    fn from(b: bool) -> Self {
        Self(Box::new(RawTokenData::Bool(b)))
    }
}

impl From<String> for TokenData {
    fn from(s: String) -> Self {
        Self(Box::new(RawTokenData::Str(s)))
    }
}

impl From<(String, String)> for TokenData {
    fn from(ss: (String, String)) -> Self {
        Self(Box::new(RawTokenData::StrStr(ss.0, ss.1)))
    }
}

impl From<(String, char)> for TokenData {
    fn from(ss: (String, char)) -> Self {
        Self(Box::new(RawTokenData::StrChar(ss.0, ss.1)))
    }
}

impl AsRef<bool> for TokenData {
    fn as_ref(&self) -> &bool {
        self.as_bool()
    }
}

impl AsRef<String> for TokenData {
    fn as_ref(&self) -> &String {
        self.as_str()
    }
}

impl AsMut<String> for TokenData {
    fn as_mut(&mut self) -> &mut String {
        self.as_str_mut()
    }
}

impl TokenData {
    #[must_use]
    pub fn as_bool(&self) -> &bool {
        match self.0.as_ref() {
            RawTokenData::Bool(b) => b,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &String {
        match self.0.as_ref() {
            RawTokenData::Str(s) | RawTokenData::StrStr(s, _) | RawTokenData::StrChar(s, _) => s,
            _ => unreachable!(),
        }
    }

    pub fn as_str_mut(&mut self) -> &mut String {
        match self.0.as_mut() {
            RawTokenData::Str(s) | RawTokenData::StrStr(s, _) | RawTokenData::StrChar(s, _) => s,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn as_str_str(&self) -> (&String, &String) {
        match self.0.as_ref() {
            RawTokenData::StrStr(s1, s2) => (s1, s2),
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn as_str_char(&self) -> (&String, &char) {
        match self.0.as_ref() {
            RawTokenData::StrChar(s, c) => (s, c),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn token_test() {
        print!(
            "testxxxxx \n{}\n{}\n{}\n{}\n. ",
            Token::ILLEGAL("asd".to_owned().into()),
            Token::SWITCH,
            Token::IDENT("some_var".to_owned().into()),
            Token::FLOAT("3.14".to_owned().into()),
        );
    }
}
