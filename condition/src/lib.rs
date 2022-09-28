#![allow(clippy::unnecessary_wraps)]

use lrlex::lrlex_mod;
use lrpar::{lrpar_mod, Span};

lrlex_mod!("cond.l");
lrpar_mod!("cond.y");

// a=1
// a=1 && b=1
// a=1 || b=1
// a=1 && b=1 || c=1 && b=2
// TODO :
//   1.添加(1,2,3) array 解析
//   2.添加 in、notin 解析
//   3.添加 like/not like 解析

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
        errors.push(format!(
            "{}, text: \"{}\"",
            e.pp(&lexer, &cond_y::token_epp),
            s
        ));
    }

    if errors.len() > 0 {
        return Err(anyhow::anyhow!("{}", errors.concat()));
    }

    match res {
        Some(expr) => Ok(expr),
        None => return Err(anyhow::anyhow!("{}", "Unable to evaluate expression.")),
    }
}

pub(crate) fn remove_apostrophe(s: String) -> String {
    s.trim_end_matches("'")
        .to_string()
        .trim_start_matches("'")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_base() {
        let sym = "a=1&&b=2||b=2&&c=1||com_id=1||com-id=2&&com-name='abc'";

        sym.starts_with("'");
        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }
}
