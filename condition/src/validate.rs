#![allow(clippy::unnecessary_wraps)]

use crate::Value;
use lrlex::lrlex_mod;
use lrpar::{lrpar_mod, Span};
use serde_json::Number;

lrlex_mod!("validate.l");
lrpar_mod!("validate.y");

#[derive(Clone, Debug)]
pub enum Compare {
    EQ,
    NE,
    GT,
    GTE,
    LT,
    LTE,
}

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
        value: bool,
    },
    IsString {
        span: Span,
        field: String,
        value: bool,
    },
    // len(name) > 1, first this field must be string or array
    LenField {
        span: Span,
        field: String,
        compare: Compare,
        value: Number,
    },

    // src.field > 1 || src.field > tag.field
    Corss {
        span: Span,
        model: Model,
    },
}

#[derive(Clone, Debug)]
pub enum Model {
    Field {
        src_field: String,
        tag_field: String,
        compare: Compare,
    },
    Value {
        object: Object,
        field: String,
        compare: Compare,
        value: Value,
    },
}

#[derive(Clone, Debug)]
pub enum Object {
    Src,
    Tag,
}
