// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::env::VarKind;
use crate::ident::{Ident, IdentPath};
use crate::lit::Lit;
use crate::shaderast::*;
use crate::span::Span;
use crate::ty::{Ty, TyLit};
use std::cell::Cell;

#[derive(Clone, Debug)]
pub(crate) struct ConstGatherer<'a> {
    pub(crate) shader: &'a ShaderAst,
}

impl<'a> ConstGatherer<'a> {
    pub(crate) fn const_gather_expr(&self, expr: &Expr) {
        match expr.kind {
            ExprKind::Cond { span, ref expr, ref expr_if_true, ref expr_if_false } => {
                self.const_gather_cond_expr(span, expr, expr_if_true, expr_if_false)
            }
            ExprKind::Bin { span, op, ref left_expr, ref right_expr } => {
                self.const_gather_bin_expr(span, op, left_expr, right_expr)
            }
            ExprKind::Un { span, op, ref expr } => self.const_gather_un_expr(span, op, expr),
            ExprKind::MethodCall { span, ident, ref arg_exprs } => self.const_gather_method_call_expr(span, ident, arg_exprs),
            ExprKind::Field { span, ref expr, field_ident } => self.const_gather_field_expr(span, expr, field_ident),
            ExprKind::Index { span, ref expr, ref index_expr } => self.const_gather_index_expr(span, expr, index_expr),
            ExprKind::Call { span, ident_path, ref arg_exprs } => self.const_gather_call_expr(span, ident_path, arg_exprs),
            ExprKind::ConsCall { span, ty_lit, ref arg_exprs } => self.const_gather_cons_call_expr(span, ty_lit, arg_exprs),
            ExprKind::Var { span, ref kind, ident_path } => self.const_gather_var_expr(span, kind, ident_path),
            ExprKind::Lit { span, lit } => self.const_gather_lit_expr(span, lit),
        }
    }

    fn const_gather_cond_expr(&self, _span: Span, expr: &Expr, expr_if_true: &Expr, expr_if_false: &Expr) {
        self.const_gather_expr(expr);
        self.const_gather_expr(expr_if_true);
        self.const_gather_expr(expr_if_false);
    }

    fn const_gather_bin_expr(&self, _span: Span, _op: BinOp, left_expr: &Expr, right_expr: &Expr) {
        self.const_gather_expr(left_expr);
        self.const_gather_expr(right_expr);
    }

    fn const_gather_un_expr(&self, _span: Span, _op: UnOp, expr: &Expr) {
        self.const_gather_expr(expr);
    }

    fn const_gather_method_call_expr(&self, span: Span, ident: Ident, arg_exprs: &[Expr]) {
        match arg_exprs[0].ty.borrow().as_ref().unwrap() {
            Ty::Struct { ident: struct_ident } => {
                self.const_gather_call_expr(span, IdentPath::from_two(*struct_ident, ident), arg_exprs)
            }
            _ => panic!(),
        }
    }

    fn const_gather_field_expr(&self, _span: Span, expr: &Expr, _field_ident: Ident) {
        self.const_gather_expr(expr);
    }

    fn const_gather_index_expr(&self, _span: Span, expr: &Expr, _index_expr: &Expr) {
        self.const_gather_expr(expr);
    }

    fn const_gather_call_expr(&self, _span: Span, _ident_path: IdentPath, arg_exprs: &[Expr]) {
        for arg_expr in arg_exprs {
            self.const_gather_expr(arg_expr);
        }
    }

    fn const_gather_cons_call_expr(&self, _span: Span, _ty_lit: TyLit, arg_exprs: &[Expr]) {
        for arg_expr in arg_exprs {
            self.const_gather_expr(arg_expr);
        }
    }

    fn const_gather_var_expr(&self, _span: Span, _kind: &Cell<Option<VarKind>>, _ident_path: IdentPath) {}

    fn const_gather_lit_expr(&self, _span: Span, _lit: Lit) {}
}
