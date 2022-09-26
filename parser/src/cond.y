%start Expr
%token 'TEXT' INT '&&' '||' '>=' '<='
%%
Expr -> Expr:
    BaseExpr
     { $1 }
  | AndExpr { $1 }
  | OrExpr { $1 }
  ;

OrExpr -> Expr:
  BaseExpr
   '||' BaseExpr
    { Expr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  ;

AndExpr -> Expr:
  BaseExpr
   '&&' BaseExpr

    { Expr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  ;

BaseExpr
 -> Expr:
    Ident '=' Str { Expr::Eq { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '=' Number { Expr::Eq { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '>=' Number { Expr::Gte { span: $span, field: $1, value: Value::Number($3) } }
  ;

Ident -> String:
   Str { $1 }
  ;
Number -> u64:
  'INT' { $lexer.span_str($1.as_ref().unwrap().span()).parse::<u64>().unwrap() }
  ;
Str -> String:
  'TEXT' { String::from($lexer.span_str($1.as_ref().unwrap().span())) } 
  ;

%%

use lrpar::Span;
use crate::*;

// String::from($lexer.span_str($2.as_ref().unwrap().span()))