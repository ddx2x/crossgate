#![allow(clippy::unnecessary_wraps)]

use lrlex::lrlex_mod;
use lrpar::{lrpar_mod, Span};
use serde_json::Number;

lrlex_mod!("cond.l");
lrpar_mod!("cond.y");

#[derive(Clone, Debug)]
pub enum Compare {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

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
    },
    IsNotNull {
        span: Span,
        field: String,
    },
    Len {
        span: Span,
        field: String,
        cmp: Compare,
        value: Value,
    },
    Belong {
        span: Span,
        field: String,
        value: Value,
    },
    NoBelong {
        span: Span,
        field: String,
        value: Value,
    },
}

#[derive(Clone, Debug)]
pub enum Value {
    Text(String),   // abc="123"
    Number(Number), // abc=123, abc=1.2
    Bool(bool),
    List(Vec<Value>),
    Len(String),
    Null,
}

pub fn yacc_parse<'a, S: ToString + ?Sized>(s: &'a S) -> anyhow::Result<Expr> {
    let lexerdef = cond_l::lexerdef();

    let binding = s.to_string();
    let lexer = lexerdef.lexer(&binding);
    let (res, errs) = cond_y::parse(&lexer);

    let mut errors = vec![];
    for e in errs {
        errors.push(format!(
            "{}, text: \"{}\"",
            e.pp(&lexer, &cond_y::token_epp),
            binding
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

#[cfg(test)]
mod tests {
    use super::yacc_parse as parse;

    #[test]
    fn test_base() {
        let sym = r#"a=1&&b=2||b=2&&c=1||com_id=1||com-id=2&&com-name='abc'"#;

        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }

        let sym = "a.x.x=1";

        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_double_quotes() {
        match parse(r#"abc="123""#) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_base2() {
        let sym = "a=1 && ( b=1 ) && c=1";

        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_base3() {
        let sym = "a=2 && ( b=1 || b=2 ) && b=2";

        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_base4() {
        let sym = r#"a="2" && b="2""#;

        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_like() {
        let sym = "a ! '^abc' ";
        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }

        let sym = "a !! '^abc' ";
        match parse(sym) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_in_array() {
        // in
        match parse("id ~ (1)") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };

        match parse("id ~ (1,2,3,4)") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }

        match parse("id ~ ('1')") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };

        match parse("id ~ ('1','2','3','4')") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }

        match parse("id ~~ ('1','2','3','4')") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_in_string() {
        match parse("id ~ ('1','2','3','4')") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }

        match parse(r#"id ~ ("1","2")"#) {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_like_notlike() {
        // like
        match parse("full_id ! '^1.'") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };

        // no like
        match parse("full_id !! '^1.'") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_bool() {
        // is false
        match parse("active = false") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };

        // is true
        match parse("active = true") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_null() {
        // is null
        match parse("a ^ null") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };

        // is not null
        match parse("a ^^ null") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_len() {
        // len(a) = 1
        match parse("len(a) = 1") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_belong() {
        // a ∈ (1,2,3)
        match parse("a << (1,2,3)") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_nobelong() {
        // a ∈/ (1,2,3)
        match parse("a >> (1,2,3)") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_string_compare() {
        match parse("a >= 'abc'") {
            Ok(rs) => println!("{:#?}", rs),
            Err(e) => panic!("{}", e),
        };
    }
}
