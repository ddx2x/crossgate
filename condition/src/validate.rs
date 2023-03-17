#![allow(clippy::unnecessary_wraps)]

use lrlex::lrlex_mod;
use lrpar::{lrpar_mod, Span};
use serde_json::Value;

lrlex_mod!("validate.l");
lrpar_mod!("validate.y");

#[derive(Clone, Debug)]
pub enum Validate {
    And {
        span: Span,
        lhs: Box<Validate>,
        rhs: Box<Validate>,
    },
    Or {
        span: Span,
        lhs: Box<Validate>,
        rhs: Box<Validate>,
    },
    Eq {
        span: Span,
        field: String,
        value: Value,
    },
    Ne {
        span: Span,
        field: String,
        value: Value,
    },
    Gt {
        span: Span,
        field: String,
        value: Value,
    },
    Gte {
        span: Span,
        field: String,
        value: Value,
    },
    Lt {
        span: Span,
        field: String,
        value: Value,
    },
    Lte {
        span: Span,
        field: String,
        value: Value,
    },
    In {
        span: Span,
        field: String,
        value: Value,
    },
    NotIn {
        span: Span,
        field: String,
        value: Value,
    },
    IsNull {
        span: Span,
        field: String,
        value: Value,
    },
    IsNotNull {
        span: Span,
        field: String,
        value: Value,
    },
    IsNumber {
        span: Span,
        field: String,
        value: Value,
    },
    IsString {
        span: Span,
        field: String,
        value: Value,
    },
    LenField {
        // len(name) > 1, first this field must be string
        span: Span,
        field: String,
        expr: Box<Validate>,
        value: Value,
    },
    Join {
        from: String,
        expr: Box<Validate>, // just use compare expr
        field: String,
        value: Value,
    },
}
