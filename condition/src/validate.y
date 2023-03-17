%start ValidateExpr
%token STRING NUMBER IDENT '>=' '<=' '>' '<' '<>' '!=' '(' ')' 'BOOL' 'IN' 'NIN' 'NUMBER_ARRAY' 'STRING_ARRAY'
%left '||'
%right '&&'

%%
ValidateExpr -> ValidateExpr:
    Factor { $1 }
  | ValidateExprs  { $1 }
  ;

ValidateExprs -> ValidateExpr:
    Factor '||' Factor { ValidateExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' Factor { ValidateExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }  
  | ValidateExprs  '||' Factor { ValidateExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | ValidateExprs  '&&' Factor { ValidateExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '||' ValidateExprs { ValidateExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' ValidateExprs { ValidateExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | ValidateExprs  '||' ValidateExprs  { ValidateExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | ValidateExprs  '&&' ValidateExprs  { ValidateExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | '(' ValidateExprs ')' { $2 }
  ;

Factor -> ValidateExpr:
    '(' Factor ')'  { $2 }
  | TextCompare   { $1 }
  | NumberCompare { $1 }
  | BoolValidateExpr { $1 }
  ;

BoolValidateExpr -> ValidateExpr:
    Ident '='  Bool { ValidateExpr::Eq { span: $span, field: $1, value: SerdeValue::Bool($3) } }
  | Ident '<>' Bool { ValidateExpr::Ne { span: $span, field: $1, value: SerdeValue::Bool($3) } }
  | Ident '!=' Bool { ValidateExpr::Ne { span: $span, field: $1, value: SerdeValue::Bool($3) } }
  ;

TextCompare -> ValidateExpr:
    Ident '='  Text { ValidateExpr::Eq { span: $span, field: $1, value: SerdeValue::String($3) } } 
  | Ident '>'  Text { ValidateExpr::Gt { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '<'  Text { ValidateExpr::Lt { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '>=' Text { ValidateExpr::Gte { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '<=' Text { ValidateExpr::Lte { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '<>' Text { ValidateExpr::Ne { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '!=' Text { ValidateExpr::Ne { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident 'IN'  TextArray { ValidateExpr::In { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' TextArray { ValidateExpr::NotIn { span: $span, field: $1, value: $3 } }
  ;

NumberCompare -> ValidateExpr:
    Ident '='  Number { ValidateExpr::Eq { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '>'  Number { ValidateExpr::Gt { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '<'  Number { ValidateExpr::Lt { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '>=' Number { ValidateExpr::Gte { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '<=' Number { ValidateExpr::Lte { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '<>' Number { ValidateExpr::Ne { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '!=' Number { ValidateExpr::Ne { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident 'IN'  IntArray { ValidateExpr::In    { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' IntArray { ValidateExpr::NotIn { span: $span, field: $1, value: $3 } }
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
IntArray -> SerdeValue:
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
          rs.push(SerdeValue::Number(item.parse::<Number>().unwrap()));
      }
      SerdeValue::Array(rs)
  }
  ;
TextArray -> SerdeValue:
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
          rs.push(SerdeValue::String(remove_apostrophe(item.parse::<String>().unwrap())));
      }
      SerdeValue::Array(rs)
  }
  ;

%%

// use lrpar::Span;
use crate::*;
use serde_json::Number;