%start Expr
%token TEXT INT
%%
Expr -> Expr:
  TEXT '=' TEXT 
  { 
    Expr::Eq {
      span: $span,
      field: String::from($lexer.span_str($1.as_ref().unwrap().span())),
      value: Value::Text(String::from($lexer.span_str($3.as_ref().unwrap().span()))),
    }
  }
  ;
%%

use lrpar::Span;
use crate::*;

// String::from($lexer.span_str($2.as_ref().unwrap().span()))