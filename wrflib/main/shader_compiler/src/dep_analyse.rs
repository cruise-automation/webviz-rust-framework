// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::env::{Env, Sym, VarKind};
use crate::ident::{Ident, IdentPath};
use crate::lit::Lit;
use crate::shaderast::*;
use crate::span::Span;
use crate::ty::{Ty, TyLit};
use std::cell::Cell;

#[derive(Clone, Debug)]
pub(crate) struct DepAnalyser<'a> {
    pub(crate) shader: &'a ShaderAst,
    pub(crate) decl: &'a FnDecl,
    pub(crate) env: &'a Env,
}

impl<'a> DepAnalyser<'a> {
    pub(crate) fn dep_analyse_expr(&mut self, expr: &Expr) {
        match expr.kind {
            ExprKind::Cond { span, ref expr, ref expr_if_true, ref expr_if_false } => {
                self.dep_analyse_cond_expr(span, expr, expr_if_true, expr_if_false)
            }
            ExprKind::Bin { span, op, ref left_expr, ref right_expr } => {
                self.dep_analyse_bin_expr(span, op, left_expr, right_expr)
            }
            ExprKind::Un { span, op, ref expr } => self.dep_analyse_un_expr(span, op, expr),
            ExprKind::MethodCall { span, ident, ref arg_exprs } => self.dep_analyse_method_call_expr(span, ident, arg_exprs),
            ExprKind::Field { span, ref expr, field_ident } => self.dep_analyse_field_expr(span, expr, field_ident),
            ExprKind::Index { span, ref expr, ref index_expr } => self.dep_analyse_index_expr(span, expr, index_expr),
            ExprKind::Call { span, ident_path, ref arg_exprs } => self.dep_analyse_call_expr(span, ident_path, arg_exprs),
            ExprKind::ConsCall { span, ty_lit, ref arg_exprs } => self.dep_analyse_cons_call_expr(span, ty_lit, arg_exprs),
            ExprKind::Var { span, ref kind, ident_path } => self.dep_analyse_var_expr(span, kind, ident_path),
            ExprKind::Lit { span, lit } => self.dep_analyse_lit_expr(span, lit),
        }
    }

    fn dep_analyse_cond_expr(&mut self, _span: Span, expr: &Expr, expr_if_true: &Expr, expr_if_false: &Expr) {
        self.dep_analyse_expr(expr);
        self.dep_analyse_expr(expr_if_true);
        self.dep_analyse_expr(expr_if_false);
    }

    fn dep_analyse_bin_expr(&mut self, _span: Span, _op: BinOp, left_expr: &Expr, right_expr: &Expr) {
        self.dep_analyse_expr(left_expr);
        self.dep_analyse_expr(right_expr);
    }

    fn dep_analyse_un_expr(&mut self, _span: Span, _op: UnOp, expr: &Expr) {
        self.dep_analyse_expr(expr);
    }

    fn dep_analyse_method_call_expr(&mut self, span: Span, method_ident: Ident, arg_exprs: &[Expr]) {
        match arg_exprs[0].ty.borrow().as_ref().unwrap() {
            Ty::Struct { ident } => {
                self.dep_analyse_call_expr(span, IdentPath::from_two(*ident, method_ident), arg_exprs);
            }
            _ => panic!(),
        }
    }

    fn dep_analyse_field_expr(&mut self, _span: Span, expr: &Expr, _field_ident: Ident) {
        self.dep_analyse_expr(expr);
    }

    fn dep_analyse_index_expr(&mut self, _span: Span, expr: &Expr, index_expr: &Expr) {
        self.dep_analyse_expr(expr);
        self.dep_analyse_expr(index_expr);
    }

    fn dep_analyse_call_expr(&mut self, _span: Span, ident_path: IdentPath, arg_exprs: &[Expr]) {
        //let ident = ident_path.get_single().expect("IMPL");
        for arg_expr in arg_exprs {
            self.dep_analyse_expr(arg_expr);
        }
        match self.env.find_sym(ident_path).unwrap() {
            Sym::Builtin => {}
            Sym::Fn => {
                self.decl.callees.borrow_mut().as_mut().unwrap().insert(ident_path);
            }
            _ => panic!(),
        }
    }

    fn dep_analyse_cons_call_expr(&mut self, _span: Span, ty_lit: TyLit, arg_exprs: &[Expr]) {
        for arg_expr in arg_exprs {
            self.dep_analyse_expr(arg_expr);
        }
        self.decl.cons_fn_deps.borrow_mut().as_mut().unwrap().insert((
            ty_lit,
            arg_exprs.iter().map(|arg_expr| arg_expr.ty.borrow().as_ref().unwrap().clone()).collect::<Vec<_>>(),
        ));
    }

    fn dep_analyse_var_expr(&mut self, _span: Span, kind: &Cell<Option<VarKind>>, ident_path: IdentPath) {
        match kind.get().unwrap() {
            VarKind::Geometry => {
                self.decl.geometry_deps.borrow_mut().as_mut().unwrap().insert(ident_path.get_single().expect("unexpected"));
            }
            VarKind::Instance => {
                self.decl.instance_deps.borrow_mut().as_mut().unwrap().insert(ident_path.get_single().expect("unexpected"));
            }
            VarKind::Texture => {
                self.decl.has_texture_deps.set(Some(true));
            }
            VarKind::Uniform => {
                self.decl.uniform_block_deps.borrow_mut().as_mut().unwrap().insert(
                    self.shader
                        .find_uniform_decl(ident_path.get_single().expect("unexpected"))
                        .unwrap()
                        .block_ident
                        .unwrap_or(Ident::new("default")),
                );
            }
            VarKind::Varying => {
                self.decl.has_varying_deps.set(Some(true));
            }
            _ => {}
        }
    }

    fn dep_analyse_lit_expr(&mut self, _span: Span, _lit: Lit) {}
}
