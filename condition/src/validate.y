%start SerdeExpr
%token STRING NUMBER IDENT '>=' '<=' '>' '<' '<>' '!=' '(' ')' 'BOOL' 'IN' 'NIN' 'NUMBER_ARRAY' 'STRING_ARRAY'
%left '||'
%right '&&'

%%
SerdeExpr -> SerdeExpr:
    Factor { $1 }
  | SerdeExprs  { $1 }
  ;

SerdeExprs -> SerdeExpr:
    Factor '||' Factor { SerdeExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' Factor { SerdeExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }  
  | SerdeExprs  '||' Factor { SerdeExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | SerdeExprs  '&&' Factor { SerdeExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '||' SerdeExprs { SerdeExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | Factor '&&' SerdeExprs { SerdeExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | SerdeExprs  '||' SerdeExprs  { SerdeExpr::Or { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | SerdeExprs  '&&' SerdeExprs  { SerdeExpr::And { span: $span, lhs: Box::new($1), rhs: Box::new($3) }  }
  | '(' SerdeExprs ')' { $2 }
  ;

Factor -> SerdeExpr:
    '(' Factor ')'  { $2 }
  | TextCompare   { $1 }
  | NumberCompare { $1 }
  | BoolSerdeExpr { $1 }
  ;

BoolSerdeExpr -> SerdeExpr:
    Ident '='  Bool { SerdeExpr::Eq { span: $span, field: $1, value: SerdeValue::Bool($3) } }
  | Ident '<>' Bool { SerdeExpr::Ne { span: $span, field: $1, value: SerdeValue::Bool($3) } }
  | Ident '!=' Bool { SerdeExpr::Ne { span: $span, field: $1, value: SerdeValue::Bool($3) } }
  ;

TextCompare -> SerdeExpr:
    Ident '='  Text { SerdeExpr::Eq { span: $span, field: $1, value: SerdeValue::String($3) } } 
  | Ident '>'  Text { SerdeExpr::Gt { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '<'  Text { SerdeExpr::Lt { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '>=' Text { SerdeExpr::Gte { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '<=' Text { SerdeExpr::Lte { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '<>' Text { SerdeExpr::Ne { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident '!=' Text { SerdeExpr::Ne { span: $span, field: $1, value: SerdeValue::String($3) } }
  | Ident 'IN'  TextArray { SerdeExpr::In { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' TextArray { SerdeExpr::NotIn { span: $span, field: $1, value: $3 } }
  ;

NumberCompare -> SerdeExpr:
    Ident '='  Number { SerdeExpr::Eq { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '>'  Number { SerdeExpr::Gt { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '<'  Number { SerdeExpr::Lt { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '>=' Number { SerdeExpr::Gte { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '<=' Number { SerdeExpr::Lte { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '<>' Number { SerdeExpr::Ne { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident '!=' Number { SerdeExpr::Ne { span: $span, field: $1, value: SerdeValue::Number($3) } }
  | Ident 'IN'  IntArray { SerdeExpr::In    { span: $span, field: $1, value: $3 } }
  | Ident 'NIN' IntArray { SerdeExpr::NotIn { span: $span, field: $1, value: $3 } }
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