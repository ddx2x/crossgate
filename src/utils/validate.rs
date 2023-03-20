use super::Unstructed;
use condition::{Compare, Validate as Expr, Value as V1};
use serde_json::Value as V2;

fn to(cv: V1, set: &Unstructed) -> V2 {
    match cv {
        V1::Text(v) => V2::String(v),
        V1::Number(v) => V2::Number(v),
        V1::Bool(v) => V2::Bool(v),
        V1::List(v) => V2::Array(v.iter().map(|item| to(item.clone(), set)).collect()),
        V1::Field(v) => set.get(&v),
        V1::Null => V2::Null,
    }
}

pub fn validate_match<'a>(
    src: Option<&'a Unstructed>, // 表示当前已经持久化的集合
    tag: &'a Unstructed,         // 表示当前待更新的集合
    expr: Expr,
) -> bool {
    match expr {
        Expr::Corss { model, .. } => match src {
            Some(src) => match model {
                condition::Model::Field {
                    src_field,
                    tag_field,
                    compare,
                } => return cmp(&src.get(&src_field), &tag.get(&tag_field), compare),
                condition::Model::Value {
                    field,
                    compare,
                    value,
                    object,
                } => match object {
                    condition::Object::Src => {
                        return cmp(&src.get(&field), &to(value, tag), compare)
                    }
                    condition::Object::Tag => {
                        return cmp(&tag.get(&field), &to(value, tag), compare)
                    }
                },
            },
            None => return false,
        },
        _ => basic_match(tag, &expr),
    }
}

fn cmp(src: &V2, tag: &V2, compare: Compare) -> bool {
    match compare {
        Compare::EQ => src == tag,
        Compare::NE => src != tag,

        // TODO : 实现其他
        // Compare::GT => src > tag,
        // Compare::GTE => src >= tag,
        // Compare::LT => src < tag,
        // Compare::LTE => src <= tag,
        _ => false,
    }
}

fn basic_match(val: &Unstructed, expr: &Expr) -> bool {
    match expr.clone() {
        Expr::And { lhs, rhs, .. } => return basic_match(val, &lhs) && basic_match(val, &rhs),
        Expr::Or { lhs, rhs, .. } => return basic_match(val, &lhs) || basic_match(val, &rhs),
        Expr::Eq { field, value, .. } => return to(value, val) == val.get(&field),
        Expr::Ne { field, value, .. } => return to(value, val) != val.get(&field),
        Expr::Gt { field, value, .. } => {
            let value = to(value, val);
            match val.get(&field) {
                V2::Number(target_number) => {
                    if target_number.is_i64() {
                        return target_number.as_i64() > value.as_i64();
                    }
                    if target_number.is_u64() {
                        return target_number.as_u64() > value.as_u64();
                    }
                    if target_number.is_f64() {
                        return target_number.as_f64() > value.as_f64();
                    }
                    return false;
                }
                _ => return false,
            }
        }
        Expr::Gte { field, value, .. } => {
            let value = to(value, val);
            match val.get(&field) {
                V2::Number(target_number) => {
                    if target_number.is_i64() {
                        return target_number.as_i64() >= value.as_i64();
                    }
                    if target_number.is_u64() {
                        return target_number.as_u64() >= value.as_u64();
                    }
                    if target_number.is_f64() {
                        return target_number.as_f64() >= value.as_f64();
                    }
                    return false;
                }
                _ => return false,
            }
        }
        Expr::Lt { field, value, .. } => {
            let value = to(value, val);
            match val.get(&field) {
                V2::Number(target_number) => {
                    if target_number.is_i64() {
                        return target_number.as_i64() < value.as_i64();
                    }
                    if target_number.is_u64() {
                        return target_number.as_u64() < value.as_u64();
                    }
                    if target_number.is_f64() {
                        return target_number.as_f64() < value.as_f64();
                    }
                    return false;
                }
                _ => return false,
            }
        }
        Expr::Lte { field, value, .. } => {
            let value = to(value, val);
            match val.get(&field) {
                V2::Number(target_number) => {
                    if target_number.is_i64() {
                        return target_number.as_i64() <= value.as_i64();
                    }
                    if target_number.is_u64() {
                        return target_number.as_u64() <= value.as_u64();
                    }
                    if target_number.is_f64() {
                        return target_number.as_f64() <= value.as_f64();
                    }
                    return false;
                }
                _ => return false,
            }
        }
        Expr::In { field, value, .. } => {
            if let Some(value_list) = to(value, val).as_array() {
                return value_list.contains(&val.get(&field));
            };
            return false;
        }
        Expr::NotIn { field, value, .. } => {
            if let Some(value_list) = to(value, val).as_array() {
                return !value_list.contains(&val.get(&field));
            };
            return false;
        }
        Expr::IsNull { field, .. } => return val.get(&field).is_null(),
        Expr::IsNotNull { field, .. } => return !val.get(&field).is_null(),
        Expr::IsNumber { field, .. } => return val.get(&field).is_number(),
        Expr::IsString { field, .. } => return val.get(&field).is_string(),
        Expr::LenField {
            field,
            compare,
            value,
            ..
        } => match compare {
            condition::Compare::EQ => {
                if let Some(value_list) = val.get(&field).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() == required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::NE => {
                if let Some(value_list) = val.get(&field).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() != required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::GT => {
                if let Some(value_list) = val.get(&field).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() > required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::GTE => {
                if let Some(value_list) = val.get(&field).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() >= required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::LT => {
                if let Some(value_list) = val.get(&field).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() < required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::LTE => {
                if let Some(value_list) = val.get(&field).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() <= required_length as usize;
                    }
                }
                return false;
            }
        },
        _ => return false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::from_str;
    use condition::{Compare, Model};
    use lrpar::Span;

    #[test]
    fn test_basic() {
        let src = from_str(r#"{"a":123,"b":"312"}"#).unwrap();
        let span = Span::new(1, 2);

        assert_eq!(
            validate_match(
                None,
                &src,
                Expr::And {
                    span,
                    lhs: Box::new(Expr::IsNumber {
                        field: "a".to_string(),
                        value: true,
                        span,
                    }),
                    rhs: Box::new(Expr::IsString {
                        field: "b".to_string(),
                        value: true,
                        span,
                    }),
                },
            ),
            true
        );
    }

    #[test]
    fn test_is_number() {
        let unstructed = &from_str(r#"{"a":123,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsNumber {
                field: "a".to_string(),
                value: true,
                span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_is_string() {
        let unstructed = &from_str(r#"{"a":"123","b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsString {
                field: "a".to_string(),
                value: true,
                span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_is_null() {
        let unstructed = &from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsNull {
                field: "a".to_string(),
                value: V1::Null,
                span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_is_not_null() {
        let unstructed = &from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsNotNull {
                field: "b".to_string(),
                value: V1::Null,
                span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_gte() {
        let unstructed = &from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::Gte {
                field: "b".to_string(),
                value: V1::Number(serde_json::Number::from(312)),
                span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_lte() {
        let unstructed = &from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::Lte {
                field: "b".to_string(),
                value: V1::Number(serde_json::Number::from(999)),
                span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_in() {
        let unstructed = &mut from_str(r#"{"a":"a","b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::In {
                field: "a".to_string(),
                value: V1::List(vec![V1::Text("a".to_string()), V1::Text("b".to_string())]),
                span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_not_in() {
        let unstructed = from_str(r#"{"a":"c","b":312}"#).unwrap();
        let span = Span::new(1, 2);

        assert!(
            validate_match(
                None,
                &unstructed,
                Expr::NotIn {
                    field: "a".to_string(),
                    value: V1::List(vec![V1::Text("a".to_string()), V1::Text("b".to_string()),]),
                    span
                },
            ) == true
        );

        assert!(
            validate_match(
                None,
                &unstructed,
                Expr::NotIn {
                    field: "a".to_string(),
                    value: V1::List(vec![V1::Text("c".to_string()),]),
                    span
                },
            ) == false
        );
    }

    #[test]
    fn test_len() {
        let unstructed = &mut from_str(r#"{"a":[1,2,3],"b":"312"}"#).unwrap();
        let span = Span::new(1, 2);

        assert!(
            validate_match(
                None,
                unstructed,
                Expr::LenField {
                    field: "a".to_string(),
                    value: serde_json::Number::from(10),
                    compare: Compare::LT,
                    span,
                },
            ) == true
        );

        // TODO: 测试不通过
        assert!(
            validate_match(
                None,
                unstructed,
                Expr::LenField {
                    field: "b".to_string(),
                    value: serde_json::Number::from(10),
                    compare: Compare::LT,
                    span,
                },
            ) == true
        );
    }

    #[test]
    fn test_join() {
        let src = from_str(r#"{"a":[1,2,3],"b":1,"c":2}"#).unwrap();
        let tag = from_str(r#"{"a":[1,2,3],"b":2,"c":1}"#).unwrap();

        let span = Span::new(1, 2);

        // src.b = tag.c
        assert_eq!(
            validate_match(
                Some(&src),
                &tag,
                Expr::Corss {
                    model: Model::Field {
                        src_field: "b".to_string(),
                        tag_field: "c".to_string(),
                        compare: Compare::EQ,
                    },
                    span
                }
            ),
            true
        );

        // src.b = 1
        assert_eq!(
            validate_match(
                Some(&src),
                &tag,
                Expr::Corss {
                    model: Model::Value {
                        compare: Compare::EQ,
                        object: condition::Object::Src,
                        field: "b".to_string(),
                        value: V1::Number(serde_json::Number::from(1)),
                    },
                    span
                }
            ),
            true
        );
    }
}
