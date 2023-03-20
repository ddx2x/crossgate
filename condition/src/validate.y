%start Validate
%token STRING NUMBER IDENT '>=' '<=' '>' '<' '<>' '!=' '(' ')' 'BOOL' 'IN' 'NIN' 'NUMBER_ARRAY' 'STRING_ARRAY' 'IS' 'IS_NOT' 'LEN'
%left '||'
%right '&&'

%%
Validate -> Validate:
    Factor { $1 }
  | Validates  { $1 }
  ;

Validates -> Validate:
    Factor '||' Factor { Validate::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' Factor { Validate::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }  
  | Validates  '||' Factor { Validate::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Validates  '&&' Factor { Validate::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '||' Validates { Validate::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' Validates { Validate::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Validates  '||' Validates  { Validate::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Validates  '&&' Validates  { Validate::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | '(' Validates ')' { $2 }
  ;

Factor -> Validate:
    '(' Factor ')'  { $2 }
  | TextCompare   { $1 }
  | NumberCompare { $1 }
  | BoolValidate { $1 }
  ;

BoolValidate -> Validate:
    Ident '='  Bool { Validate::Eq { span: $span, field: $1, value: Value::Bool($3) } }
  | Ident '<>' Bool { Validate::Ne { span: $span, field: $1, value: Value::Bool($3) } }
  | Ident '!=' Bool { Validate::Ne { span: $span, field: $1, value: Value::Bool($3) } }
  ;

TextCompare -> Validate:
    Ident '='  Text { Validate::Eq { span: $span, field: $1, value: Value::Text($3) } } 
  | Ident '>'  Text { Validate::Gt { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '<'  Text { Validate::Lt { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '>=' Text { Validate::Gte { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '<=' Text { Validate::Lte { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '<>' Text { Validate::Ne { span: $span, field: $1, value: Value::Text($3) } }
  | Ident '!=' Text { Validate::Ne { span: $span, field: $1, value: Value::Text($3) } }
  | Ident 'IN'  TextArray { Validate::In { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' TextArray { Validate::NotIn { span: $span, field: $1, value: $3 } }
  | Ident 'IS'  NUMBER_TYPE { Validate::IsNumber { span: $span, field: $1, value: $3 } }
  | Ident 'IS'  STRING_TYPE { Validate::IsString { span: $span, field: $1, value: $3 } }
  | LEN '(' Ident ')' '=' Number { Validate::LenField { span: $span, field: $3, compare: EQ, value: $6 } }
  | LEN '(' Ident ')' '!=' Number { Validate::LenField { span: $span, field: $3, compare: NE, value: $6 } }
  | LEN '(' Ident ')' '<>' Number { Validate::LenField { span: $span, field: $3, compare: NE, value: $6 } }
  | LEN '(' Ident ')' '>' Number { Validate::LenField { span: $span, field: $3, compare: GT, value: $6 } }
  | LEN '(' Ident ')' '<' Number { Validate::LenField { span: $span, field: $3, compare: LT, value: $6 } }
  | LEN '(' Ident ')' '>=' Number { Validate::LenField { span: $span, field: $3, compare: GTE, value: $6 } }
  | LEN '(' Ident ')' '<=' Number { Validate::LenField { span: $span, field: $3, compare: LTE, value: $6 } }
  ;

NumberCompare -> Validate:
    Ident '='  Number { Validate::Eq { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '>'  Number { Validate::Gt { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<'  Number { Validate::Lt { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '>=' Number { Validate::Gte { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<=' Number { Validate::Lte { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '<>' Number { Validate::Ne { span: $span, field: $1, value: Value::Number($3) } }
  | Ident '!=' Number { Validate::Ne { span: $span, field: $1, value: Value::Number($3) } }
  | Ident 'IN'  IntArray { Validate::In    { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' IntArray { Validate::NotIn { span: $span, field: $1, value: $3 } }
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
NUMBER_TYPE -> bool:
    'NUMBER' { true } ;

STRING_TYPE-> bool:
    'STRING' { true } ;

%%

// use lrpar::Span;
use crate::*;

use serde_json::Number;
use crate::Compare::{EQ,NE,GT,LT,GTE,LTE};
use crate::{Validate,remove_apostrophe};
