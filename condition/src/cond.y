%start Expr
%token STRING NUMBER IDENT '>=' '<=' '>' '<' '<>' '!=' '(' ')' 'BOOL' 'LIKE' 'NLIKE' 'IN' 'NIN' 'NUMBER_ARRAY' 'STRING_ARRAY' 'IS' 'IS_NOT' 'NULL' 'LEN' 'BELONG'
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
  | BelongCompare { $1 }
  | BoolExpr { $1 }
  | IsExpr { $1 }
  | LenExpr { $1 }
  ;

LenExpr -> Expr:
    'LEN''('Ident')' '='  Number   { Expr::Len { span: $span, field: $3, cmp: Compare::Eq, value: Value::Number($6) } }
  | 'LEN''('Ident')' '>=' Number   { Expr::Len { span: $span, field: $3, cmp: Compare::Gte, value: Value::Number($6) } }
  | 'LEN''('Ident')' '<=' Number   { Expr::Len { span: $span, field: $3, cmp: Compare::Lte, value: Value::Number($6) } }
  | 'LEN''('Ident')' '>'  Number   { Expr::Len { span: $span, field: $3, cmp: Compare::Gt, value: Value::Number($6) } }
  | 'LEN''('Ident')' '<'  Number   { Expr::Len { span: $span, field: $3, cmp: Compare::Lt, value: Value::Number($6) } }
  | 'LEN''('Ident')' '!='  Number  { Expr::Len { span: $span, field: $3, cmp: Compare::Ne, value: Value::Number($6) } }
  | 'LEN''('Ident')' '<>'  Number  { Expr::Len { span: $span, field: $3, cmp: Compare::Ne, value: Value::Number($6) } }
  ;

IsExpr -> Expr:
    Ident 'IS' Null     { Expr::IsNull    { span: $span, field: $1 } }
  | Ident 'IS_NOT' Null { Expr::IsNotNull { span: $span, field: $1 } }
  ;

BoolExpr -> Expr:
    Ident '='  Bool { Expr::Eq { span: $span, field: $1, value: Value::Bool($3) } }
  | Ident '<>' Bool { Expr::Ne { span: $span, field: $1, value: Value::Bool($3) } }
  | Ident '!=' Bool { Expr::Ne { span: $span, field: $1, value: Value::Bool($3) } }
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

BelongCompare -> Expr:
    Ident 'BELONG'  TextArray { Expr::Belong { span: $span, field: $1, value: $3 } }
  | Ident 'BELONG'  IntArray  { Expr::Belong { span: $span, field: $1, value: $3 } }
  ;

Text -> String:
  'STRING' { remove_apostrophe($lexer.span_str($1.as_ref().unwrap().span()).to_string()) } 
  ;
Ident -> String:
  'IDENT' { remove_apostrophe($lexer.span_str($1.as_ref().unwrap().span()).to_string()) } 
  ;
Number -> Number:
  'NUMBER' { $lexer.span_str($1.as_ref().unwrap().span()).parse::<Number>().unwrap() }
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
          rs.push(Value::Text(remove_apostrophe(item.parse::<String>().unwrap())));
      }
      Value::List(rs)
  }
  ;

Null -> Value:
  'NULL' { Value::Null }
  ;

%%

// use lrpar::Span;
use crate::*;
use serde_json::Number;