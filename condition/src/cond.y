%start Expr
%token STRING NUMBER IDENT '>=' '<=' '>' '<' '<>' '!=' '(' ')' 'BOOL' 'LIKE' 'NLIKE' 'IN' 'NIN' 'NUMBER_ARRAY' 'STRING_ARRAY'
%left '||'
%right '&&'

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
  | Factor '||' Exprs { Expr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' Exprs { Expr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Exprs  '||' Exprs  { Expr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Exprs  '&&' Exprs  { Expr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | '(' Exprs ')' { $2 }
  ;

Factor -> Expr:
    '(' Factor ')'  { $2 }
  | TextCompare   { $1 }
  | NumberCompare { $1 }
  | IdentCompare  { $1 }
  ;

IdentCompare -> Expr:
    Ident '='  Ident { Expr::Eq { span: $span, field: $1, value: Value::Field($3) } }
  | Ident '>'  Ident { Expr::Gt { span: $span, field: $1, value: Value::Field($3) } }
  | Ident '<'  Ident { Expr::Lt { span: $span, field: $1, value: Value::Field($3) } }
  | Ident '>=' Ident { Expr::Gte { span: $span, field: $1, value: Value::Field($3) } }
  | Ident '<=' Ident { Expr::Lte { span: $span, field: $1, value: Value::Field($3) } }
  | Ident '<>' Ident { Expr::Ne { span: $span, field: $1, value: Value::Field($3) } }
  | Ident '!=' Ident { Expr::Ne { span: $span, field: $1, value: Value::Field($3) } }
  ;

TextCompare -> Expr:
    Ident '='  Text { Expr::Eq { span: $span, field: $1, value: Value::Text($3) } } 
  | Ident '>'  Text { Expr::Gt { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '<'  Text { Expr::Lt { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '>=' Text { Expr::Gte { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '<=' Text { Expr::Lte { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '<>' Text { Expr::Ne { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '!=' Text { Expr::Ne { span: $span, field: $1, value: Value::Text($3) } }
  | Ident 'LIKE'  Text  { Expr::Like { span: $span, field: $1, value: Value::Text($3) } }
  | Ident 'NLIKE' Text  { Expr::NotLike { span: $span, field: $1, value: Value::Text($3) } }
  | Ident 'IN'  TextArray { Expr::In { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' TextArray { Expr::NotIn { span: $span, field: $1, value: $3 } }
  ;

NumberCompare -> Expr:
    Ident '='  Number { Expr::Eq { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '>'  Number { Expr::Gt { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<'  Number { Expr::Lt { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '>=' Number { Expr::Gte { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<=' Number { Expr::Lte { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<>' Number { Expr::Ne { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '!=' Number { Expr::Ne { span: $span, field: $1, value: Value::Number($3) } }
  | Ident 'IN'  IntArray { Expr::In    { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' IntArray { Expr::NotIn { span: $span, field: $1, value: $3 } }
  ;
Ident -> String:
  'IDENT' { String::from($lexer.span_str($1.as_ref().unwrap().span())) } 
  ;
Number -> Number:
  'NUMBER' { $lexer.span_str($1.as_ref().unwrap().span()).parse::<Number>().unwrap() }
  ;
Text -> String:
  'STRING' { remove_apostrophe(String::from($lexer.span_str($1.as_ref().unwrap().span()))) } 
  ;
Bool -> bool:
  'BOOL' { $lexer.span_str($1.as_ref().unwrap().span()).parse::<bool>().unwrap() }
  ;
IntArray -> Value:
  'NUMBER_ARRAY' 
  {
       let mut rs = vec![];
       let src = String::from($lexer.span_str($1.as_ref().unwrap().span()));
        let binding = src
            .trim_start_matches("(")
            .to_string();
        let items = binding
            .trim_end_matches(")")
            .split(",");
      for item in items {
          rs.push(Value::Number(item.parse::<Number>().unwrap()));
      }
      Value::List(rs)
  }
  ;
TextArray -> Value:
  'STRING_ARRAY' 
  {
       let mut rs = vec![];
       let src = String::from($lexer.span_str($1.as_ref().unwrap().span()));
        let binding = src
            .trim_start_matches("(")
            .to_string();
        let items = binding
            .trim_end_matches(")")
            .split(",");
      for item in items {
          rs.push(Value::Text(item.parse::<String>().unwrap()));
      }
      Value::List(rs)
  }
  ;

%%

// use lrpar::Span;
use crate::*;
use serde_json::Number;