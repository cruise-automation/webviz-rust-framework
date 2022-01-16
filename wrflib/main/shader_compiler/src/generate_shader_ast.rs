// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::analyse::analyse_shader;
use crate::builtin::generate_builtins;
use crate::builtin::Builtin;
use crate::detok::DeTokParserImpl;
use crate::error::ParseError;
use crate::ident::Ident;
use crate::lex::lex;
use crate::shaderast::ShaderAst;
use crate::span::CodeFragmentId;
use crate::token::{Token, TokenWithSpan};
use std::collections::HashMap;

/// TODO(JP): Would be nice if we can make [`ShaderAstGenerator::builtins`] a `const` so we don't
/// need to keep any state.
pub struct ShaderAstGenerator {
    builtins: HashMap<Ident, Builtin>,
}

/// Represents a location in a file, and the code string itself. Generate easily
/// using the `wrflib::code_fragment!()` macro.
#[derive(Debug, Clone)]
pub struct CodeFragment {
    pub filename: &'static str,
    pub line: usize,
    pub col: usize,
    pub code: &'static str,
}

impl CodeFragment {
    /// Offset the `line` and `col` fields by a certain number of characters.
    fn offset_line_col(&self, offset_chars: usize) -> (usize, usize) {
        let mut line = self.line;
        let mut col = self.col;
        for (char_index, ch) in self.code.chars().enumerate() {
            if char_index == offset_chars {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

impl ShaderAstGenerator {
    pub fn new() -> Self {
        Self { builtins: generate_builtins() }
    }

    /// Generate a complete [`ShaderAst`] from some code fragments.
    pub fn generate_shader_ast(&self, shader_name: &str, code_fragments: &[CodeFragment]) -> ShaderAst {
        let handle_parse_error = |parse_error: ParseError| {
            let code_fragment = &code_fragments[parse_error.span.code_fragment_id.0];
            let pos = parse_error.span.start;
            let (line, col) = code_fragment.offset_line_col(pos);
            // Hacky debugging: besides printing the file/line/col and so on, we also pull out a
            // string (without newlines) from the actual code, and position it such that `pos`
            // appears right under our "vvvvvv". :-)
            panic!(
                "Error parsing shader '{}' at {}:{}:{} => {}\n\naround:             vvvvvv\n{}\n\n",
                shader_name,
                code_fragment.filename,
                line,
                col,
                parse_error.message,
                code_fragment.code.chars().skip(pos - 20).take(50).collect::<String>().replace("\n", " "),
            );
        };

        let mut tokens: Vec<TokenWithSpan> = vec![];
        let code_fragments_len = code_fragments.len();
        for (index, code_fragment) in code_fragments.iter().enumerate() {
            for token_result in lex(code_fragment.code.chars(), CodeFragmentId(index)) {
                let token = token_result.map_err(handle_parse_error).unwrap();
                // Skip intermediate `Eof` tokens, but keep the last one.
                if token.token != Token::Eof || index == code_fragments_len - 1 {
                    tokens.push(token);
                }
            }
        }
        let shader_ast = DeTokParserImpl::new(&tokens).parse_shader().map_err(handle_parse_error).unwrap();
        analyse_shader(&self.builtins, &shader_ast).map_err(handle_parse_error).unwrap();
        shader_ast
    }
}
