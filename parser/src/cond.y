%start Expr
%token TEXT INT '>=' '<=' '>' '<' 
%left '||'
%right '&&'
// %nonassoc '||' 

%%
Expr -> Expr:
    Factor { $1 }
  | Exprs  { $1 }
  ;


Exprs -> Expr:
    Factor '||' Factor { Expr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' Factor { Expr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }  
  | Exprs  '||' Factor { Expr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Exprs  '&&' Factor { Expr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Exprs  '||' Exprs  { Expr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Exprs  '&&' Exprs  { Expr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  ;

Factor -> Expr:
    Ident '=' Str { Expr::Eq { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '=' Number { Expr::Eq { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '>' Number { Expr::Gt { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<' Number { Expr::Lt { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '>=' Number { Expr::Gte { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<=' Number { Expr::Lte { span: $span, field: $1, value: Value::Number($3) } }
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