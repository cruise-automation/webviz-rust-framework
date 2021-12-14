// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::ident::Ident;
use crate::lit::Lit;
use crate::span::Span;
use crate::ty::TyLit;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TokenWithSpan {
    pub(crate) span: Span,
    pub(crate) token: Token,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Token {
    Eof,
    Not,
    NotEq,
    AndAnd,
    LeftParen,
    RightParen,
    Star,
    StarEq,
    Plus,
    PlusEq,
    Comma,
    Minus,
    MinusEq,
    Arrow,
    Dot,
    Slash,
    SlashEq,
    Colon,
    PathSep,
    Semi,
    Lt,
    LtEq,
    Eq,
    EqEq,
    Gt,
    GtEq,
    Question,
    Break,
    Const,
    Continue,
    Else,
    For,
    Fn,
    //From,
    If,
    //In,
    Inout,
    Let,
    Return,
    //Crate,
    Splat,
    //Self_,
    Struct,
    //To,
    LeftBracket,
    RightBracket,
    LeftBrace,
    OrOr,
    RightBrace,
    String(Ident),
    Ident(Ident),
    Lit(Lit),
    TyLit(TyLit),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Eof => write!(f, "<eof>"),
            Token::Not => write!(f, "!"),
            Token::NotEq => write!(f, "!="),
            Token::AndAnd => write!(f, "&&"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Star => write!(f, "*"),
            Token::StarEq => write!(f, "*="),
            Token::Plus => write!(f, "+"),
            Token::PlusEq => write!(f, "+="),
            Token::Comma => write!(f, ","),
            Token::Minus => write!(f, "-"),
            Token::Arrow => write!(f, "=>"),
            Token::Dot => write!(f, "."),
            Token::Splat => write!(f, ".."),
            Token::MinusEq => write!(f, "-="),
            Token::Slash => write!(f, "/"),
            Token::SlashEq => write!(f, "/="),
            Token::Colon => write!(f, ":"),
            Token::PathSep => write!(f, ":"),
            Token::Semi => write!(f, ";"),
            Token::Lt => write!(f, "<"),
            Token::LtEq => write!(f, "<="),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::Gt => write!(f, ">"),
            Token::GtEq => write!(f, ">="),
            Token::Question => write!(f, "?"),
            Token::Break => write!(f, "break"),
            Token::Const => write!(f, "const"),
            Token::Continue => write!(f, "continue"),
            Token::Else => write!(f, "else"),
            Token::Fn => write!(f, "fn"),
            Token::For => write!(f, "for"),
            Token::If => write!(f, "if"),
            Token::Inout => write!(f, "inout"),
            Token::Let => write!(f, "let"),
            Token::Return => write!(f, "return"),
            Token::Struct => write!(f, "struct"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::OrOr => write!(f, "||"),
            Token::RightBrace => write!(f, "}}"),
            Token::Ident(ident) => write!(f, "{}", ident),
            Token::String(ident) => write!(f, "\"{}\"", ident),
            Token::Lit(lit) => write!(f, "{}", lit),
            Token::TyLit(lit) => write!(f, "{}", lit),
        }
    }
}
