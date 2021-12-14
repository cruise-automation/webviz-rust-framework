// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use wrflib::*;

use crate::textcursor::*;
use crate::tokentype::*;

#[derive(Clone, Default)]
pub struct TextBuffer {
    // Vec<Vec<char>> was chosen because, for all practical use (code) most lines are short
    // Concatenating the total into a utf8 string is trivial, and O(1) for windowing into the lines is handy.
    // Also inserting lines is pretty cheap even approaching 100k lines.
    // If you want to load a 100 meg single line file or something with >100k lines
    // other options are better. But these are not usecases for this editor.
    pub lines: Vec<Vec<char>>,
    pub undo_stack: Vec<TextUndo>,
    pub redo_stack: Vec<TextUndo>,

    pub signal: Signal,

    pub mutation_id: u32,
    pub is_crlf: bool,
    pub markers: TextBufferMarkers,
    pub flat_text: Vec<char>,
    pub token_chunks: Vec<TokenChunk>,
    pub was_invalid_pair: bool,
    pub old_flat_text: Vec<char>,
    pub old_token_chunks: Vec<TokenChunk>,
    pub token_chunks_id: u32,
    pub keyboard: TextBufferKeyboard,
}

impl TextBuffer {
    pub const STATUS_MESSAGE_UPDATE: StatusId = location_hash!();
    pub const STATUS_SEARCH_UPDATE: StatusId = location_hash!();
    pub const STATUS_DATA_UPDATE: StatusId = location_hash!();
    pub const STATUS_KEYBOARD_UPDATE: StatusId = location_hash!();
    pub const TOKEN_CHUNKS_CHANGED: StatusId = location_hash!();
}

#[derive(Clone, Default)]
pub struct TextBufferKeyboard {
    pub modifiers: KeyModifiers,
    pub key_down: Option<KeyCode>,
    pub key_up: Option<KeyCode>,
}

#[derive(Clone, Default)]
pub struct TextBufferMarkers {
    pub mutation_id: u32,
    // only if this matches the textbuffer mutation id are the messages valid
    pub search_cursors: Vec<TextCursor>,
    pub message_cursors: Vec<TextCursor>,
    pub message_bodies: Vec<TextBufferMessage>,
}

#[derive(Clone, PartialEq)]
pub enum TextBufferMessageLevel {
    Error,
    Warning,
    Log,
}

#[derive(Clone)]
pub struct TextBufferMessage {
    pub level: TextBufferMessageLevel,
    pub body: String,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct TextPos {
    pub row: usize,
    pub col: usize,
}

impl TextPos {
    pub fn dist(&self, other: &TextPos) -> f64 {
        let dr = (self.row as f64) - (other.row as f64);
        let dc = (self.col as f64) - (other.col as f64);
        (dr * dr + dc * dc).sqrt()
    }

    pub fn zero() -> TextPos {
        TextPos { row: 0, col: 0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextUndoGrouping {
    Space,
    LiveEdit(u64),
    Newline,
    Character(u64),
    Backspace(u64),
    Delete(usize),
    Block,
    Tab,
    Cut,
    Format,
    Other,
}

impl Default for TextUndoGrouping {
    fn default() -> TextUndoGrouping {
        TextUndoGrouping::Other
    }
}

impl TextUndoGrouping {
    fn wants_grouping(&self) -> bool {
        match self {
            TextUndoGrouping::Space => true,
            TextUndoGrouping::LiveEdit(_) => true,
            TextUndoGrouping::Newline => false,
            TextUndoGrouping::Character(_) => true,
            TextUndoGrouping::Backspace(_) => true,
            TextUndoGrouping::Delete(_) => true,
            TextUndoGrouping::Block => false,
            TextUndoGrouping::Tab => false,
            TextUndoGrouping::Format => false,
            TextUndoGrouping::Cut => false,
            TextUndoGrouping::Other => false,
        }
    }
}

#[derive(Clone)]
pub struct TextUndo {
    pub ops: Vec<TextOp>,
    pub grouping: TextUndoGrouping,
    pub cursors: TextCursorSet,
}

#[derive(Clone)]
pub struct TextOp {
    pub start: usize,
    pub len: usize,
    pub lines: Vec<Vec<char>>,
}

fn calc_char_count(lines: &Vec<Vec<char>>) -> usize {
    let mut char_count = 0;
    for line in lines {
        char_count += line.len()
    }
    char_count += lines.len() - 1;
    // invisible newline chars
    char_count
}

impl TextBuffer {
    pub fn from_utf8(data: &str) -> Self {
        let mut tb = TextBuffer::default();
        tb.load_from_utf8(data);
        tb
    }

    pub fn needs_token_chunks(&mut self) -> bool {
        if self.token_chunks_id != self.mutation_id {
            self.token_chunks_id = self.mutation_id;
            if !self.was_invalid_pair {
                std::mem::swap(&mut self.token_chunks, &mut self.old_token_chunks);
                std::mem::swap(&mut self.flat_text, &mut self.old_flat_text);
            }
            self.was_invalid_pair = false;
            self.token_chunks.truncate(0);
            self.flat_text.truncate(0);
            return true;
        }
        false
    }

    pub fn scan_token_chunks_prev_line(&self, token: usize, lines: usize) -> (usize, isize) {
        let mut nls = 0;
        for i in (0..token).rev() {
            if let TokenType::Newline = self.token_chunks[i].token_type {
                nls += 1;
                if nls == lines {
                    return (i + 1, -1);
                }
            }
        }
        (0, 0)
    }

    pub fn scan_token_chunks_next_line(&self, token: usize, lines: usize) -> usize {
        let mut nls = 0;
        for i in token..self.token_chunks.len() {
            if let TokenType::Newline = self.token_chunks[i].token_type {
                nls += 1;
                if nls == lines {
                    return i + 1;
                }
            }
        }
        self.token_chunks.len()
    }

    pub fn offset_to_text_pos(&self, char_offset: usize) -> TextPos {
        let mut char_count = 0;
        for (row, line) in self.lines.iter().enumerate() {
            let next_char_count = char_count + line.len() + 1;
            if next_char_count > char_offset {
                return TextPos { row, col: char_offset - char_count };
            }
            char_count = next_char_count;
        }
        TextPos { row: self.lines.len().max(1) - 1, col: 0 }
    }

    pub fn offset_to_text_pos_next(&self, query_off: usize, old_pos: TextPos, old_off: usize) -> TextPos {
        let mut row = old_pos.row;
        let mut iter_off = old_off - old_pos.col;
        while row < self.lines.len() {
            let line = &self.lines[row];
            let next_off = iter_off + line.len() + 1;
            if next_off > query_off {
                return TextPos { row, col: query_off - iter_off };
            }
            iter_off = next_off;
            row += 1;
        }
        TextPos { row: self.lines.len().max(1) - 1, col: 0 }
    }

    pub fn text_pos_to_offset(&self, pos: TextPos) -> usize {
        let mut char_count = 0;
        if pos.row >= self.lines.len() {
            return self.calc_char_count();
        }
        for (ln_row, line) in self.lines.iter().enumerate() {
            if ln_row == pos.row {
                return char_count + (line.len()).min(pos.col);
            }
            char_count += line.len() + 1;
        }
        0
    }

    pub fn get_nearest_line_range(&self, offset: usize) -> (usize, usize) {
        let pos = self.offset_to_text_pos(offset);
        let line = &self.lines[pos.row];
        (offset - pos.col, line.len() + if pos.row < (line.len().max(1) - 1) { 1 } else { 0 })
    }

    pub fn calc_next_line_indent_depth(&self, offset: usize, tabsize: usize) -> (usize, usize) {
        let pos = self.offset_to_text_pos(offset);
        let line = &self.lines[pos.row];
        let mut prev_index = pos.col;
        if prev_index == 0 || prev_index > line.len() {
            return (offset - pos.col, 0);
        };

        let mut instep = 0;
        while prev_index > 0 {
            let prev = line[prev_index - 1];
            if prev == ')' || prev == '}' || prev == ']' {
                break;
            }
            if prev == '{' || prev == '(' || prev == '[' {
                instep = tabsize;
                break;
            }
            prev_index -= 1;
        }
        for (i, ch) in line.iter().enumerate() {
            if *ch != ' ' {
                return (offset - pos.col, i + instep);
            }
        }
        (offset - pos.col, line.len())
    }

    pub fn calc_line_indent_depth(&self, row: usize) -> usize {
        let line = &self.lines[row];
        for (i, ch) in line.iter().enumerate() {
            if *ch != ' ' {
                return i;
            }
        }
        line.len()
    }

    pub fn calc_backspace_line_indent_depth_and_pair(&self, offset: usize) -> (usize, usize) {
        let pos = self.offset_to_text_pos(offset);
        let line = &self.lines[pos.row];
        // check pair removal
        if pos.col >= 1 && pos.col < line.len() {
            let pch = line[pos.col - 1];
            let nch = line[pos.col];
            if pch == '{' && nch == '}' || pch == '(' && nch == ')' || pch == '[' && nch == ']' {
                return (offset - 1, 2);
            }
        }
        (offset - 1, 1)
    }

    pub fn calc_deletion_whitespace(&self, offset: usize) -> Option<(usize, usize, usize, usize)> {
        let pos = self.offset_to_text_pos(offset);
        if self.lines.is_empty() || pos.row >= self.lines.len() - 1 {
            return None;
        }
        let line1 = &self.lines[pos.row];
        let mut line1_ws = 0;
        for ch in line1 {
            if *ch != ' ' {
                break;
            }
            line1_ws += 1;
        }

        let line2 = &self.lines[pos.row + 1];
        let mut line2_ws = 0;
        for ch in line2 {
            if *ch != ' ' {
                break;
            }
            line2_ws += 1;
        }

        Some((offset - pos.col, line1_ws, line1.len(), line2_ws))
    }

    pub fn calc_deindent_whitespace(&self, offset: usize) -> Option<(usize, usize, usize)> {
        let pos = self.offset_to_text_pos(offset);
        if self.lines.is_empty() || pos.row >= self.lines.len() {
            return None;
        }
        let line1 = &self.lines[pos.row];
        let mut line1_ws = 0;
        for ch in line1 {
            if *ch != ' ' {
                break;
            }
            line1_ws += 1;
        }

        Some((offset - pos.col, line1_ws, line1.len()))
    }

    pub fn calc_char_count(&self) -> usize {
        calc_char_count(&self.lines)
    }

    pub fn get_line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 0 || self.lines.len() == 1 && self.lines[0].len() == 0
    }

    pub fn get_range_as_string(&self, start: usize, len: usize, ret: &mut String) {
        let mut pos = self.offset_to_text_pos(start);
        for _ in 0..len {
            let line = &self.lines[pos.row];
            if pos.col >= line.len() {
                ret.push('\n');
                pos.col = 0;
                pos.row += 1;
                if pos.row >= self.lines.len() {
                    return;
                }
            } else {
                ret.push(line[pos.col]);
                pos.col += 1;
            }
        }
    }

    pub fn get_char(&self, start: usize) -> char {
        let pos = self.offset_to_text_pos(start);
        let line = &self.lines[pos.row];
        if pos.row == self.lines.len() - 1 && pos.col >= line.len() {
            return '\0';
        }
        if pos.col >= line.len() {
            return '\n';
        }
        line[pos.col]
    }

    pub fn get_as_string(&self) -> String {
        let mut ret = String::new();
        for i in 0..self.lines.len() {
            let line = &self.lines[i];
            for ch in line {
                ret.push(*ch);
            }
            if i != self.lines.len() - 1 {
                if self.is_crlf {
                    ret.push('\r');
                    ret.push('\n');
                } else {
                    ret.push('\n');
                }
            }
        }
        ret
    }

    pub fn load_from_utf8(&mut self, utf8: &str) {
        self.is_crlf = !!utf8.contains("\r\n");
        self.lines = TextBuffer::split_string_to_lines(utf8);
        self.mutation_id += 1;
    }

    pub fn replace_line(&mut self, row: usize, start_col: usize, len: usize, rep_line: Vec<char>) -> Vec<char> {
        self.mutation_id += 1;
        self.lines[row].splice(start_col..(start_col + len), rep_line).collect()
    }

    pub fn copy_line(&self, row: usize, start_col: usize, len: usize) -> Vec<char> {
        let line = &self.lines[row];
        if start_col >= line.len() {
            return vec![];
        }
        if start_col + len > line.len() {
            self.lines[row][start_col..line.len()].to_vec()
        } else {
            self.lines[row][start_col..(start_col + len)].to_vec()
        }
    }

    pub fn mark_clean(&mut self) {
        self.token_chunks_id = self.mutation_id;
    }

    pub fn replace_range(&mut self, start: usize, len: usize, mut rep_lines: Vec<Vec<char>>) -> Vec<Vec<char>> {
        self.mutation_id += 1;
        let start_pos = self.offset_to_text_pos(start);
        let end_pos = self.offset_to_text_pos_next(start + len, start_pos, start);

        if start_pos.row == end_pos.row && rep_lines.len() == 1 {
            // replace in one line
            let rep_line_zero = rep_lines.drain(0..1).next().unwrap();

            if start_pos.col > end_pos.col {
                return vec![];
            }
            let line = self.lines[start_pos.row].splice(start_pos.col..end_pos.col, rep_line_zero).collect();
            return vec![line];
        } else if rep_lines.len() == 1 {
            // we are replacing multiple lines with one line
            // drain first line
            let rep_line_zero = rep_lines.drain(0..1).next().unwrap();

            // replace it in the first
            let first = self.lines[start_pos.row].splice(start_pos.col.., rep_line_zero).collect();

            // collect the middle ones
            let mut middle: Vec<Vec<char>> = self.lines.drain((start_pos.row + 1)..(end_pos.row)).collect();

            // cut out the last bit
            let last: Vec<char> = self.lines[start_pos.row + 1].drain(0..end_pos.col).collect();

            // last line bit
            let mut last_line = self.lines.drain((start_pos.row + 1)..(start_pos.row + 2)).next().unwrap();

            // merge start_row+1 into start_row
            self.lines[start_pos.row].append(&mut last_line);

            // concat it all together
            middle.insert(0, first);
            middle.push(last);

            middle
        } else if start_pos.row == end_pos.row {
            // replacing single line with multiple lines

            let mut last_bit: Vec<char> = self.lines[start_pos.row].drain(end_pos.col..).collect();
            // but we have co drain end_col..

            // replaced first line
            let rep_lines_len = rep_lines.len();
            let rep_line_first: Vec<char> = rep_lines.drain(0..1).next().unwrap();
            let line = self.lines[start_pos.row].splice(start_pos.col.., rep_line_first).collect();

            // splice in middle rest
            let rep_line_mid = rep_lines.drain(0..(rep_lines.len()));
            self.lines.splice((start_pos.row + 1)..(start_pos.row + 1), rep_line_mid);

            // append last bit
            self.lines[start_pos.row + rep_lines_len - 1].append(&mut last_bit);

            return vec![line];
        } else {
            // replaceing multiple lines with multiple lines
            // drain and replace last line
            let rep_line_last = rep_lines.drain((rep_lines.len() - 1)..(rep_lines.len())).next().unwrap();
            let last = self.lines[end_pos.row].splice(..end_pos.col, rep_line_last).collect();

            // swap out middle lines and drain them
            let rep_line_mid = rep_lines.drain(1..(rep_lines.len()));
            let mut middle: Vec<Vec<char>> = self.lines.splice((start_pos.row + 1)..end_pos.row, rep_line_mid).collect();

            // drain and replace first line
            let rep_line_zero = rep_lines.drain(0..1).next().unwrap();
            let first = self.lines[start_pos.row].splice(start_pos.col.., rep_line_zero).collect();

            // concat it all together
            middle.insert(0, first);
            middle.push(last);
            middle
        }
    }

    pub fn replace_lines(&mut self, start_row: usize, end_row: usize, rep_lines: Vec<Vec<char>>) -> TextOp {
        let start = self.text_pos_to_offset(TextPos { row: start_row, col: 0 });
        let end = self.text_pos_to_offset(TextPos { row: end_row, col: 0 });
        let end_mark = if end_row >= self.lines.len() { 0 } else { 1 };
        let rep_lines_chars = calc_char_count(&rep_lines);
        let lines = self.replace_range(start, end - start - end_mark, rep_lines);
        TextOp { start, len: rep_lines_chars, lines }
    }

    pub fn split_string_to_lines(string: &str) -> Vec<Vec<char>> {
        if !!string.contains("\r\n") {
            return string.split("\r\n").map(|s| s.chars().collect()).collect();
        } else {
            return string.split('\n').map(|s| s.chars().collect()).collect();
        }
    }

    pub fn replace_lines_with_string(&mut self, start: usize, len: usize, string: &str) -> TextOp {
        let rep_lines = Self::split_string_to_lines(string);
        let rep_lines_chars = calc_char_count(&rep_lines);
        let lines = self.replace_range(start, len, rep_lines);

        TextOp { start, len: rep_lines_chars, lines }
    }

    pub fn replace_line_with_string(&mut self, start: usize, row: usize, col: usize, len: usize, string: &str) -> TextOp {
        let rep_line: Vec<char> = string.chars().collect();
        let rep_line_chars = rep_line.len();
        let line = self.replace_line(row, col, len, rep_line);
        TextOp { start, len: rep_line_chars, lines: vec![line] }
    }

    pub fn replace_with_textop(&mut self, text_op: TextOp) -> TextOp {
        let rep_lines_chars = calc_char_count(&text_op.lines);
        let lines = self.replace_range(text_op.start, text_op.len, text_op.lines);
        TextOp { start: text_op.start, len: rep_lines_chars, lines }
    }

    pub fn save_buffer(&mut self) {
        //let out = self.lines.join("\n");
    }

    pub fn live_edit(&mut self, start: usize, end: usize, value: &str) -> bool {
        let was_dirty = self.token_chunks_id != self.mutation_id;
        let op = self.replace_lines_with_string(start, end - start, value);
        self.undo_stack.push(TextUndo {
            ops: vec![op],
            grouping: TextUndoGrouping::LiveEdit(0),
            cursors: TextCursorSet {
                set: vec![TextCursor { head: 0, tail: 0, max: 0 }],
                last_cursor: 0,
                insert_undo_group: 0,
                last_clamp_range: None,
            },
        });
        // check if we can hotpatch tokenchunks
        if !was_dirty && value.len() == end - start {
            for (index, c) in value.chars().enumerate() {
                self.flat_text[start + index] = c;
            }
            self.token_chunks_id = self.mutation_id;
            return true;
        }
        false
    }

    pub fn undoredo(&mut self, mut text_undo: TextUndo, cursor_set: &mut TextCursorSet) -> TextUndo {
        let mut ops = Vec::new();
        while !text_undo.ops.is_empty() {
            let op = text_undo.ops.pop().unwrap();
            //text_undo.ops.len() - 1);
            ops.push(self.replace_with_textop(op));
        }
        let text_undo_inverse = TextUndo { ops, grouping: text_undo.grouping, cursors: cursor_set.clone() };
        cursor_set.set = text_undo.cursors.set.clone();
        cursor_set.last_cursor = text_undo.cursors.last_cursor;
        text_undo_inverse
    }

    // todo make more reuse in these functions
    pub fn undo(&mut self, grouped: bool, cursor_set: &mut TextCursorSet) {
        if self.undo_stack.is_empty() {
            return;
        }
        let mut last_grouping = TextUndoGrouping::Other;
        let mut first = true;
        while !self.undo_stack.is_empty() {
            if !first && !grouped {
                break;
            }
            if self.undo_stack.last().unwrap().grouping != last_grouping && !first {
                break;
            }
            first = false;
            let text_undo = self.undo_stack.pop().unwrap();
            let wants_grouping = text_undo.grouping.wants_grouping();
            last_grouping = text_undo.grouping.clone();
            let text_redo = self.undoredo(text_undo, cursor_set);
            self.redo_stack.push(text_redo);
            if !wants_grouping {
                break;
            }
        }
    }

    pub fn redo(&mut self, grouped: bool, cursor_set: &mut TextCursorSet) {
        if self.redo_stack.is_empty() {
            return;
        }
        let mut last_grouping = TextUndoGrouping::Other;
        let mut first = true;
        while !self.redo_stack.is_empty() {
            if !first && (self.redo_stack.last().unwrap().grouping != last_grouping || !grouped) {
                break;
            }
            first = false;
            let text_redo = self.redo_stack.pop().unwrap();
            let wants_grouping = text_redo.grouping.wants_grouping();
            last_grouping = text_redo.grouping.clone();
            let text_undo = self.undoredo(text_redo, cursor_set);
            self.undo_stack.push(text_undo);
            if !wants_grouping {
                break;
            }
        }
    }
}

pub struct LineTokenizer<'a> {
    pub prev: char,
    pub cur: char,
    pub next: char,
    iter: std::str::Chars<'a>,
}

impl<'a> LineTokenizer<'a> {
    pub fn new(st: &'a str) -> Self {
        let mut ret = Self { prev: '\0', cur: '\0', next: '\0', iter: st.chars() };
        ret.advance();
        ret
    }

    pub fn advance(&mut self) {
        if let Some(next) = self.iter.next() {
            self.next = next;
        } else {
            self.next = '\0'
        }
    }

    pub fn next_is_digit(&self) -> bool {
        self.next >= '0' && self.next <= '9'
    }

    pub fn next_is_letter(&self) -> bool {
        self.next >= 'a' && self.next <= 'z' || self.next >= 'A' && self.next <= 'Z'
    }

    pub fn next_is_lowercase_letter(&self) -> bool {
        self.next >= 'a' && self.next <= 'z'
    }

    pub fn next_is_uppercase_letter(&self) -> bool {
        self.next >= 'A' && self.next <= 'Z'
    }

    pub fn next_is_hex(&self) -> bool {
        self.next >= '0' && self.next <= '9' || self.next >= 'a' && self.next <= 'f' || self.next >= 'A' && self.next <= 'F'
    }

    pub fn advance_with_cur(&mut self) {
        self.cur = self.next;
        self.advance();
    }

    pub fn advance_with_prev(&mut self) {
        self.prev = self.cur;
        self.cur = self.next;
        self.advance();
    }

    pub fn keyword(&mut self, chunk: &mut Vec<char>, word: &str) -> bool {
        for m in word.chars() {
            if m == self.next {
                chunk.push(m);
                self.advance();
            } else {
                return false;
            }
        }
        true
    }
}
