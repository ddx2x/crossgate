#![allow(clippy::unnecessary_wraps)]

use lrlex::lrlex_mod;
use lrpar::{lrpar_mod, Span};

lrlex_mod!("cond.l");
lrpar_mod!("cond.y");

// a=1
// a=1 && b=1
// a=1 || b=1
// a=1 && b=1 || c=1 && b=2
#[derive(Debug)]
pub enum Expr {
    And {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Or {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
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
    Le {
        span: Span,
        field: String,
        value: Value,
    },
    Lte {
        span: Span,
        field: String,
        value: Value,
    },
    Like {
        span: Span,
        field: String,
        value: Value,
    },
    NotLike {
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
}

#[derive(Debug)]
pub enum Value {
    Text(String), // abc="123"
    Number(u64),  // abc=123
    Bool(bool),
    List(Vec<Value>),
    Field(String), // a=b field.a = field.b
    Null,
}

pub fn parse<'a>(s: &'a str) -> anyhow::Result<Expr> {
    let lexerdef = cond_l::lexerdef();

    let lexer = lexerdef.lexer(s);
    let (res, errs) = cond_y::parse(&lexer);

    let mut errors = vec![];
    for e in errs {
        errors.push(format!("{}\r\n", e.pp(&lexer, &cond_y::token_epp)));
    }

    if errors.len() > 0 {
        return Err(anyhow::anyhow!("{}", errors.concat()));
    }

    match res {
        Some(expr) => Ok(expr),
        None => return Err(anyhow::anyhow!("{}", "Unable to evaluate expression.")),
    }
}
