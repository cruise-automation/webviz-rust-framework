// Copyright (c) 2021-present, Cruise LLC
//
// This source code is licensed under the Apache License, Version 2.0,
// found in the LICENSE-APACHE file in the root directory of this source tree.
// You may not use this file except in compliance with the License.

use crate::builtin::Builtin;
use crate::env::{Env, Sym, VarKind};
use crate::error::ParseError;
use crate::ident::{Ident, IdentPath};
use crate::lhs_check::LhsChecker;
use crate::lit::Lit;
use crate::shaderast::*;
use crate::span::Span;
use crate::swizzle::Swizzle;
use crate::ty::{Ty, TyExpr, TyExprKind, TyLit};
use crate::util::CommaSep;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct TyChecker<'a> {
    pub(crate) builtins: &'a HashMap<Ident, Builtin>,
    pub(crate) shader: &'a ShaderAst,
    pub(crate) env: &'a Env,
}

impl<'a> TyChecker<'a> {
    fn lhs_checker(&self) -> LhsChecker {
        LhsChecker { env: self.env }
    }

    pub(crate) fn ty_check_ty_expr(&mut self, ty_expr: &TyExpr) -> Result<Ty, ParseError> {
        let ty = match ty_expr.kind {
            TyExprKind::Array { span, ref elem_ty_expr, len } => self.ty_check_array_ty_expr(span, elem_ty_expr, len),
            TyExprKind::Var { span, ident } => self.ty_check_var_ty_expr(span, ident),
            TyExprKind::Lit { span, ty_lit } => self.ty_check_lit_ty_expr(span, ty_lit),
        }?;
        *ty_expr.ty.borrow_mut() = Some(ty.clone());
        Ok(ty)
    }

    fn ty_check_array_ty_expr(&mut self, _span: Span, elem_ty_expr: &TyExpr, len: u32) -> Result<Ty, ParseError> {
        let elem_ty = Rc::new(self.ty_check_ty_expr(elem_ty_expr)?);
        let len = len as usize;
        Ok(Ty::Array { elem_ty, len })
    }

    fn ty_check_var_ty_expr(&mut self, span: Span, ident: Ident) -> Result<Ty, ParseError> {
        match self
            .env
            .find_sym(IdentPath::from_ident(ident))
            .ok_or_else(|| ParseError { span, message: format!("`{}` is not defined in this scope", ident) })?
        {
            Sym::TyVar { ty } => Ok(ty),
            _ => Err(ParseError { span, message: format!("`{}` is not a type variable", ident) }),
        }
    }

    fn ty_check_lit_ty_expr(&mut self, _span: Span, ty_lit: TyLit) -> Result<Ty, ParseError> {
        Ok(ty_lit.to_ty())
    }

    pub(crate) fn ty_check_expr_with_expected_ty(&mut self, span: Span, expr: &Expr, expected_ty: &Ty) -> Result<Ty, ParseError> {
        let actual_ty = self.ty_check_expr(expr)?;
        if &actual_ty != expected_ty {
            return Err(ParseError {
                span,
                message: format!("can't match expected type `{}` with actual type `{}", expected_ty, actual_ty),
            });
        }
        Ok(actual_ty)
    }

    pub(crate) fn ty_check_expr(&mut self, expr: &Expr) -> Result<Ty, ParseError> {
        let ty = match expr.kind {
            ExprKind::Cond { span, ref expr, ref expr_if_true, ref expr_if_false, .. } => {
                self.ty_check_cond_expr(span, expr, expr_if_true, expr_if_false)
            }
            ExprKind::Bin { span, op, ref left_expr, ref right_expr, .. } => {
                self.ty_check_bin_expr(span, op, left_expr, right_expr)
            }
            ExprKind::Un { span, op, ref expr } => self.ty_check_un_expr(span, op, expr),
            ExprKind::MethodCall { span, ident, ref arg_exprs } => self.ty_check_method_call_expr(span, ident, arg_exprs),
            ExprKind::Field { span, ref expr, field_ident } => self.ty_check_field_expr(span, expr, field_ident),
            ExprKind::Index { span, ref expr, ref index_expr } => self.ty_check_index_expr(span, expr, index_expr),
            ExprKind::Call { span, ident_path, ref arg_exprs } => self.ty_check_call_expr(span, ident_path, arg_exprs),
            ExprKind::ConsCall { span, ty_lit, ref arg_exprs } => self.ty_check_cons_call_expr(span, ty_lit, arg_exprs),
            ExprKind::Var { span, ref kind, ident_path } => self.ty_check_var_expr(span, kind, ident_path),
            ExprKind::Lit { span, lit } => self.ty_check_lit_expr(span, lit),
        }?;
        *expr.ty.borrow_mut() = Some(ty.clone());
        Ok(ty)
    }

    fn ty_check_cond_expr(
        &mut self,
        span: Span,
        expr: &Expr,
        expr_if_true: &Expr,
        expr_if_false: &Expr,
    ) -> Result<Ty, ParseError> {
        self.ty_check_expr_with_expected_ty(span, expr, &Ty::Bool)?;
        let ty_if_true = self.ty_check_expr(expr_if_true)?;
        self.ty_check_expr_with_expected_ty(span, expr_if_false, &ty_if_true)?;
        Ok(ty_if_true)
    }

    fn ty_check_bin_expr(&mut self, span: Span, op: BinOp, left_expr: &Expr, right_expr: &Expr) -> Result<Ty, ParseError> {
        let left_ty = self.ty_check_expr(left_expr)?;
        let right_ty = self.ty_check_expr(right_expr)?;
        match op {
            BinOp::Assign | BinOp::AddAssign | BinOp::SubAssign | BinOp::MulAssign | BinOp::DivAssign => {
                self.lhs_checker().lhs_check_expr(left_expr)?;
            }
            _ => {}
        }
        match op {
            BinOp::Assign => {
                if left_ty == right_ty {
                    Some(left_ty.clone())
                } else {
                    None
                }
            }
            BinOp::AddAssign | BinOp::SubAssign | BinOp::DivAssign => match (&left_ty, &right_ty) {
                (Ty::Int, Ty::Int) => Some(Ty::Int),
                (Ty::Float, Ty::Float) => Some(Ty::Float),
                (Ty::Ivec2, Ty::Int) => Some(Ty::Ivec2),
                (Ty::Ivec2, Ty::Ivec2) => Some(Ty::Ivec2),
                (Ty::Ivec3, Ty::Int) => Some(Ty::Ivec3),
                (Ty::Ivec3, Ty::Ivec3) => Some(Ty::Ivec3),
                (Ty::Ivec4, Ty::Int) => Some(Ty::Ivec4),
                (Ty::Ivec4, Ty::Ivec4) => Some(Ty::Ivec4),
                (Ty::Vec2, Ty::Float) => Some(Ty::Vec2),
                (Ty::Vec2, Ty::Vec2) => Some(Ty::Vec2),
                (Ty::Vec3, Ty::Float) => Some(Ty::Vec3),
                (Ty::Vec3, Ty::Vec3) => Some(Ty::Vec3),
                (Ty::Vec4, Ty::Float) => Some(Ty::Vec4),
                (Ty::Vec4, Ty::Vec4) => Some(Ty::Vec4),
                (Ty::Mat2, Ty::Float) => Some(Ty::Mat2),
                (Ty::Mat2, Ty::Mat2) => Some(Ty::Mat2),
                (Ty::Mat3, Ty::Float) => Some(Ty::Mat3),
                (Ty::Mat3, Ty::Mat3) => Some(Ty::Mat3),
                (Ty::Mat4, Ty::Float) => Some(Ty::Mat4),
                (Ty::Mat4, Ty::Mat4) => Some(Ty::Mat4),
                _ => None,
            },
            BinOp::MulAssign => match (&left_ty, &right_ty) {
                (Ty::Int, Ty::Int) => Some(Ty::Int),
                (Ty::Float, Ty::Float) => Some(Ty::Float),
                (Ty::Ivec2, Ty::Int) => Some(Ty::Ivec2),
                (Ty::Ivec2, Ty::Ivec2) => Some(Ty::Ivec2),
                (Ty::Ivec3, Ty::Int) => Some(Ty::Ivec3),
                (Ty::Ivec3, Ty::Ivec3) => Some(Ty::Ivec3),
                (Ty::Ivec4, Ty::Int) => Some(Ty::Ivec4),
                (Ty::Ivec4, Ty::Ivec4) => Some(Ty::Ivec4),
                (Ty::Vec2, Ty::Float) => Some(Ty::Vec2),
                (Ty::Vec2, Ty::Vec2) => Some(Ty::Vec2),
                (Ty::Vec2, Ty::Mat2) => Some(Ty::Vec2),
                (Ty::Vec3, Ty::Float) => Some(Ty::Vec3),
                (Ty::Vec3, Ty::Vec3) => Some(Ty::Vec3),
                (Ty::Vec3, Ty::Mat3) => Some(Ty::Vec3),
                (Ty::Vec4, Ty::Float) => Some(Ty::Vec4),
                (Ty::Vec4, Ty::Vec4) => Some(Ty::Vec4),
                (Ty::Vec4, Ty::Mat4) => Some(Ty::Vec4),
                (Ty::Mat2, Ty::Float) => Some(Ty::Mat2),
                (Ty::Mat2, Ty::Mat2) => Some(Ty::Mat2),
                (Ty::Mat3, Ty::Float) => Some(Ty::Mat3),
                (Ty::Mat3, Ty::Mat3) => Some(Ty::Mat3),
                (Ty::Mat4, Ty::Float) => Some(Ty::Mat4),
                (Ty::Mat4, Ty::Mat4) => Some(Ty::Mat4),
                _ => None,
            },
            BinOp::Or | BinOp::And => match (&left_ty, &right_ty) {
                (Ty::Bool, Ty::Bool) => Some(Ty::Bool),
                _ => None,
            },
            BinOp::Eq | BinOp::Ne => match (&left_ty, &right_ty) {
                (Ty::Bool, Ty::Bool) => Some(Ty::Bool),
                (Ty::Int, Ty::Int) => Some(Ty::Bool),
                (Ty::Float, Ty::Float) => Some(Ty::Bool),
                (Ty::Ivec2, Ty::Ivec2) => Some(Ty::Bool),
                (Ty::Ivec3, Ty::Ivec3) => Some(Ty::Bool),
                (Ty::Ivec4, Ty::Ivec4) => Some(Ty::Bool),
                (Ty::Vec2, Ty::Vec2) => Some(Ty::Bool),
                (Ty::Vec3, Ty::Vec3) => Some(Ty::Bool),
                (Ty::Vec4, Ty::Vec4) => Some(Ty::Bool),
                (Ty::Mat2, Ty::Mat2) => Some(Ty::Bool),
                (Ty::Mat3, Ty::Mat3) => Some(Ty::Bool),
                (Ty::Mat4, Ty::Mat4) => Some(Ty::Bool),
                _ => None,
            },
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => match (&left_ty, &right_ty) {
                (Ty::Int, Ty::Int) => Some(Ty::Bool),
                (Ty::Float, Ty::Float) => Some(Ty::Bool),
                _ => None,
            },
            BinOp::Add | BinOp::Sub | BinOp::Div => match (&left_ty, &right_ty) {
                (Ty::Int, Ty::Int) => Some(Ty::Int),
                (Ty::Float, Ty::Float) => Some(Ty::Float),
                (Ty::Float, Ty::Vec2) => Some(Ty::Vec2),
                (Ty::Float, Ty::Vec3) => Some(Ty::Vec3),
                (Ty::Float, Ty::Vec4) => Some(Ty::Vec4),
                (Ty::Float, Ty::Mat2) => Some(Ty::Mat2),
                (Ty::Float, Ty::Mat3) => Some(Ty::Mat3),
                (Ty::Float, Ty::Mat4) => Some(Ty::Mat4),
                (Ty::Ivec2, Ty::Int) => Some(Ty::Ivec2),
                (Ty::Ivec2, Ty::Ivec2) => Some(Ty::Ivec2),
                (Ty::Ivec3, Ty::Int) => Some(Ty::Ivec3),
                (Ty::Ivec3, Ty::Ivec3) => Some(Ty::Ivec3),
                (Ty::Ivec4, Ty::Int) => Some(Ty::Ivec4),
                (Ty::Ivec4, Ty::Ivec4) => Some(Ty::Ivec4),
                (Ty::Vec2, Ty::Float) => Some(Ty::Vec2),
                (Ty::Vec2, Ty::Vec2) => Some(Ty::Vec2),
                (Ty::Vec3, Ty::Float) => Some(Ty::Vec3),
                (Ty::Vec3, Ty::Vec3) => Some(Ty::Vec3),
                (Ty::Vec4, Ty::Float) => Some(Ty::Vec4),
                (Ty::Vec4, Ty::Vec4) => Some(Ty::Vec4),
                (Ty::Mat2, Ty::Float) => Some(Ty::Mat2),
                (Ty::Mat2, Ty::Mat2) => Some(Ty::Mat2),
                (Ty::Mat3, Ty::Float) => Some(Ty::Mat3),
                (Ty::Mat3, Ty::Mat3) => Some(Ty::Mat3),
                (Ty::Mat4, Ty::Float) => Some(Ty::Mat4),
                (Ty::Mat4, Ty::Mat4) => Some(Ty::Mat4),
                _ => None,
            },
            BinOp::Mul => match (&left_ty, &right_ty) {
                (Ty::Int, Ty::Int) => Some(Ty::Int),
                (Ty::Float, Ty::Float) => Some(Ty::Float),
                (Ty::Float, Ty::Vec2) => Some(Ty::Vec2),
                (Ty::Float, Ty::Vec3) => Some(Ty::Vec3),
                (Ty::Float, Ty::Vec4) => Some(Ty::Vec4),
                (Ty::Float, Ty::Mat2) => Some(Ty::Mat2),
                (Ty::Float, Ty::Mat3) => Some(Ty::Mat3),
                (Ty::Float, Ty::Mat4) => Some(Ty::Mat4),
                (Ty::Ivec2, Ty::Int) => Some(Ty::Ivec2),
                (Ty::Ivec2, Ty::Ivec2) => Some(Ty::Ivec2),
                (Ty::Ivec3, Ty::Int) => Some(Ty::Ivec3),
                (Ty::Ivec3, Ty::Ivec3) => Some(Ty::Ivec3),
                (Ty::Ivec4, Ty::Int) => Some(Ty::Ivec4),
                (Ty::Ivec4, Ty::Ivec4) => Some(Ty::Ivec4),
                (Ty::Vec2, Ty::Float) => Some(Ty::Vec2),
                (Ty::Vec2, Ty::Vec2) => Some(Ty::Vec2),
                (Ty::Vec2, Ty::Mat2) => Some(Ty::Vec2),
                (Ty::Vec3, Ty::Float) => Some(Ty::Vec3),
                (Ty::Vec3, Ty::Vec3) => Some(Ty::Vec3),
                (Ty::Vec3, Ty::Mat3) => Some(Ty::Vec3),
                (Ty::Vec4, Ty::Float) => Some(Ty::Vec4),
                (Ty::Vec4, Ty::Vec4) => Some(Ty::Vec4),
                (Ty::Vec4, Ty::Mat4) => Some(Ty::Vec4),
                (Ty::Mat2, Ty::Float) => Some(Ty::Mat2),
                (Ty::Mat2, Ty::Vec2) => Some(Ty::Vec2),
                (Ty::Mat2, Ty::Mat2) => Some(Ty::Mat2),
                (Ty::Mat3, Ty::Float) => Some(Ty::Mat3),
                (Ty::Mat3, Ty::Vec3) => Some(Ty::Vec3),
                (Ty::Mat3, Ty::Mat3) => Some(Ty::Mat3),
                (Ty::Mat4, Ty::Float) => Some(Ty::Mat4),
                (Ty::Mat4, Ty::Vec4) => Some(Ty::Vec4),
                (Ty::Mat4, Ty::Mat4) => Some(Ty::Mat4),
                _ => None,
            },
        }
        .ok_or_else(|| ParseError {
            span,
            message: format!("can't apply binary operator `{}` to operands of type `{}` and `{}", op, left_ty, right_ty),
        })
    }

    fn ty_check_un_expr(&mut self, span: Span, op: UnOp, expr: &Expr) -> Result<Ty, ParseError> {
        let ty = self.ty_check_expr(expr)?;
        match op {
            UnOp::Not => match ty {
                Ty::Bool => Some(Ty::Bool),
                _ => None,
            },
            UnOp::Neg => match ty {
                Ty::Int => Some(Ty::Int),
                Ty::Float => Some(Ty::Float),
                Ty::Vec2 => Some(Ty::Vec2),
                Ty::Vec3 => Some(Ty::Vec3),
                Ty::Vec4 => Some(Ty::Vec4),
                _ => None,
            },
        }
        .ok_or_else(|| ParseError { span, message: format!("can't apply unary operator `{}` to operand of type `{}`", op, ty) })
    }

    fn ty_check_method_call_expr(&mut self, span: Span, ident: Ident, arg_exprs: &[Expr]) -> Result<Ty, ParseError> {
        let ty = self.ty_check_expr(&arg_exprs[0])?;
        match ty {
            Ty::Struct { ident: struct_ident } => {
                self.ty_check_call_expr(span, IdentPath::from_two(struct_ident, ident), arg_exprs)
            }
            _ => Err(ParseError { span, message: format!("method `{}` is not defined on type `{}`", ident, ty) }),
        }
    }

    fn ty_check_field_expr(&mut self, span: Span, expr: &Expr, field_ident: Ident) -> Result<Ty, ParseError> {
        let ty = self.ty_check_expr(expr)?;
        match ty {
            ref ty if ty.is_vector() => {
                let swizzle = Swizzle::parse(field_ident)
                    .filter(|swizzle| {
                        if swizzle.len() > 4 {
                            return false;
                        }
                        let size = ty.size();
                        for &index in swizzle {
                            if index > size {
                                return false;
                            }
                        }
                        true
                    })
                    .ok_or_else(|| ParseError {
                        span,
                        message: format!("field `{}` is not defined on type `{}`", field_ident, ty),
                    })?;
                Ok(match ty {
                    Ty::Bvec2 | Ty::Bvec3 | Ty::Bvec4 => match swizzle.len() {
                        1 => Ty::Bool,
                        2 => Ty::Bvec2,
                        3 => Ty::Bvec3,
                        4 => Ty::Bvec4,
                        _ => panic!(),
                    },
                    Ty::Ivec2 | Ty::Ivec3 | Ty::Ivec4 => match swizzle.len() {
                        1 => Ty::Int,
                        2 => Ty::Ivec2,
                        3 => Ty::Ivec3,
                        4 => Ty::Ivec4,
                        _ => panic!(),
                    },
                    Ty::Vec2 | Ty::Vec3 | Ty::Vec4 => match swizzle.len() {
                        1 => Ty::Float,
                        2 => Ty::Vec2,
                        3 => Ty::Vec3,
                        4 => Ty::Vec4,
                        _ => panic!(),
                    },
                    _ => panic!(),
                })
            }
            Ty::Struct { ident } => Ok(self
                .shader
                .find_struct_decl(ident)
                .unwrap()
                .find_field(field_ident)
                .ok_or(ParseError { span, message: format!("field `{}` is not defined on type `{}`", field_ident, ident) })?
                .ty_expr
                .ty
                .borrow()
                .as_ref()
                .unwrap()
                .clone()),
            _ => Err(ParseError { span, message: format!("can't access field on value of type `{}`", ty) }),
        }
    }

    fn ty_check_index_expr(&mut self, span: Span, expr: &Expr, index_expr: &Expr) -> Result<Ty, ParseError> {
        let ty = self.ty_check_expr(expr)?;
        let index_ty = self.ty_check_expr(index_expr)?;
        let elem_ty = match ty {
            Ty::Bvec2 | Ty::Bvec3 | Ty::Bvec4 => Ty::Bool,
            Ty::Ivec2 | Ty::Ivec3 | Ty::Ivec4 => Ty::Int,
            Ty::Vec2 | Ty::Vec3 | Ty::Vec4 => Ty::Float,
            Ty::Mat2 => Ty::Vec2,
            Ty::Mat3 => Ty::Vec3,
            Ty::Mat4 => Ty::Vec4,
            Ty::Array { elem_ty, len: _ } => elem_ty.as_ref().clone(),
            _ => return Err(ParseError { span, message: format!("can't index into value of type `{}`", ty) }),
        };
        if index_ty != Ty::Int {
            return Err(ParseError { span, message: "index is not an integer".into() });
        }
        Ok(elem_ty)
    }

    fn ty_check_call_expr(&mut self, span: Span, ident_path: IdentPath, arg_exprs: &[Expr]) -> Result<Ty, ParseError> {
        for arg_expr in arg_exprs {
            self.ty_check_expr(arg_expr)?;
        }

        match self
            .env
            .find_sym(ident_path)
            .ok_or_else(|| ParseError { span, message: format!("`{}` is not defined", ident_path) })?
        {
            Sym::Builtin => {
                let builtin = self.builtins.get(&ident_path.get_single().expect("unexpected")).unwrap();
                let arg_tys = arg_exprs.iter().map(|arg_expr| arg_expr.ty.borrow().as_ref().unwrap().clone()).collect::<Vec<_>>();
                Ok(builtin
                    .return_tys
                    .get(&arg_tys)
                    .ok_or({
                        let mut message = String::new();
                        write!(message, "can't apply builtin `{}` to arguments of types ", ident_path).unwrap();
                        let mut sep = "";
                        for arg_ty in arg_tys {
                            write!(message, "{}{}", sep, arg_ty).unwrap();
                            sep = ", ";
                        }
                        ParseError { span, message }
                    })?
                    .clone())
            }
            Sym::Fn => {
                let fn_decl = self.shader.find_fn_decl(ident_path).unwrap();
                if arg_exprs.len() < fn_decl.params.len() {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "not enough arguments for call to function `{}`: expected {}, got {}",
                            ident_path,
                            fn_decl.params.len(),
                            arg_exprs.len(),
                        ),
                    });
                }
                if arg_exprs.len() > fn_decl.params.len() {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "too many arguments for call to function `{}`: expected {}, got {}",
                            ident_path,
                            fn_decl.params.len(),
                            arg_exprs.len()
                        ),
                    });
                }
                for (index, (arg_expr, param)) in arg_exprs.iter().zip(fn_decl.params.iter()).enumerate() {
                    let arg_ty = arg_expr.ty.borrow();
                    let arg_ty = arg_ty.as_ref().unwrap();
                    let param_ty = param.ty_expr.ty.borrow();
                    let param_ty = param_ty.as_ref().unwrap();
                    if arg_ty != param_ty {
                        return Err(ParseError {
                            span,
                            message: format!(
                                "wrong type for argument {} in call to function `{}`: expected `{}`, got `{}`",
                                index + 1,
                                ident_path,
                                param_ty,
                                arg_ty,
                            ),
                        });
                    }
                    if param.is_inout {
                        self.lhs_checker().lhs_check_expr(arg_expr)?;
                    }
                }
                Ok(fn_decl.return_ty.borrow().as_ref().unwrap().clone())
            }
            _ => Err(ParseError { span, message: format!("`{}` is not a function", ident_path) }),
        }
    }

    #[allow(clippy::redundant_closure_call)]
    fn ty_check_cons_call_expr(&mut self, span: Span, ty_lit: TyLit, arg_exprs: &[Expr]) -> Result<Ty, ParseError> {
        let ty = ty_lit.to_ty();
        let arg_tys = arg_exprs.iter().map(|arg_expr| self.ty_check_expr(arg_expr)).collect::<Result<Vec<_>, _>>()?;
        match (&ty, arg_tys.as_slice()) {
            (ty, [arg_ty]) if ty.is_scalar() && arg_ty.is_scalar() => Ok(ty.clone()),
            (ty, [arg_ty]) if ty.is_vector() && arg_ty.is_scalar() => Ok(ty.clone()),
            (ty, [arg_ty]) if ty.is_matrix() && arg_ty.is_scalar() || arg_ty.is_matrix() => Ok(ty.clone()),
            (ty, arg_tys)
                if ty.is_vector()
                    && (|| arg_tys.iter().all(|arg_ty| arg_ty.is_scalar() || arg_ty.is_vector() || arg_ty.is_matrix()))()
                    || ty.is_matrix()
                        && (|| arg_tys.iter().all(|arg_ty| arg_ty.is_scalar() || arg_ty.is_vector() || arg_ty.is_matrix()))() =>
            {
                let expected_size = ty.size();
                let actual_size = arg_tys.iter().map(|arg_ty| arg_ty.size()).sum::<usize>();
                if actual_size < expected_size {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "not enough components for call to constructor `{}`: expected {}, got {}",
                            ty_lit, actual_size, expected_size,
                        ),
                    });
                }
                if actual_size > expected_size {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "too many components for call to constructor `{}`: expected {}, got {}",
                            ty_lit, expected_size, actual_size,
                        ),
                    });
                }
                Ok(ty.clone())
            }
            _ => Err(ParseError {
                span,
                message: format!("can't construct value of type `{}` with arguments of types `{}`", ty, CommaSep(&arg_tys)),
            }),
        }
    }

    fn ty_check_var_expr(&mut self, span: Span, kind: &Cell<Option<VarKind>>, ident_path: IdentPath) -> Result<Ty, ParseError> {
        match self
            .env
            .find_sym(ident_path)
            .ok_or_else(|| ParseError { span, message: format!("`{}` is not defined in this scope", ident_path) })?
        {
            Sym::Var { ref ty, kind: new_kind, .. } => {
                kind.set(Some(new_kind));
                Ok(ty.clone())
            }
            _ => Err(ParseError { span, message: format!("`{}` is not a variable", ident_path) }),
        }
    }

    fn ty_check_lit_expr(&mut self, _span: Span, lit: Lit) -> Result<Ty, ParseError> {
        Ok(lit.to_ty())
    }
}
