use super::Unstructed;
use condition::Validate as Expr;
use serde_json::Value;

// 需要传入target，因为可能出现 a=b 的情况
fn condition_value_to_serde_value<'a>(value: condition::Value, tag: &'a mut Unstructed) -> Value {
    match value {
        condition::Value::Text(str) => serde_json::Value::String(str),
        condition::Value::Number(num) => serde_json::Value::Number(num),
        condition::Value::Bool(flag) => serde_json::Value::Bool(flag),
        condition::Value::List(list) => {
            let mut arr = Vec::new();
            for val in list {
                arr.push(condition_value_to_serde_value(val, tag));
            }
            serde_json::Value::Array(arr)
        }
        condition::Value::Field(field) => tag.get(field.as_str()),
        condition::Value::Null => serde_json::Value::Null,
    }
}

pub fn validate_match<'a>(
    src: Option<&'a mut Unstructed>, // 表示当前已经持久化的集合
    tag: &'a mut Unstructed,         // 表示当前待更新的集合
    expr: Expr,
) -> bool {
    match expr {
        Expr::And { span: _, lhs, rhs } => match src {
            Some(src) => {
                return validate_match(Some(src), tag, *lhs) && validate_match(Some(src), tag, *rhs)
            }
            None => return validate_match(None, tag, *lhs) && validate_match(None, tag, *rhs),
        },
        Expr::Or { span: _, lhs, rhs } => match src {
            Some(src) => {
                return validate_match(Some(src), tag, *lhs) || validate_match(Some(src), tag, *rhs)
            }
            None => return validate_match(None, tag, *lhs) || validate_match(None, tag, *rhs),
        },
        Expr::Eq {
            span: _,
            field,
            value,
        } => return condition_value_to_serde_value(value, tag) == tag.get(field.as_str()),
        Expr::Ne {
            span: _,
            field,
            value,
        } => return condition_value_to_serde_value(value, tag) != tag.get(field.as_str()),
        Expr::Gt {
            span: _,
            field,
            value,
        } => {
            let value = condition_value_to_serde_value(value, tag);

            match tag.get(field.as_str()) {
                Value::Number(target_number) => {
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
        Expr::Gte {
            span: _,
            field,
            value,
        } => {
            let value = condition_value_to_serde_value(value, tag);
            match tag.get(field.as_str()) {
                Value::Number(target_number) => {
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
        Expr::Lt {
            span: _,
            field,
            value,
        } => {
            let value = condition_value_to_serde_value(value, tag);
            match tag.get(field.as_str()) {
                Value::Number(target_number) => {
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
        Expr::Lte {
            span: _,
            field,
            value,
        } => {
            let value = condition_value_to_serde_value(value, tag);
            match tag.get(field.as_str()) {
                Value::Number(target_number) => {
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
        Expr::In {
            span: _,
            field,
            value,
        } => {
            if let Some(value_list) = condition_value_to_serde_value(value, tag).as_array() {
                return value_list.contains(&tag.get(field.as_str()));
            };
            return false;
        }
        Expr::NotIn {
            span: _,
            field,
            value,
        } => {
            if let Some(value_list) = condition_value_to_serde_value(value, tag).as_array() {
                return !value_list.contains(&tag.get(field.as_str()));
            };
            return false;
        }
        Expr::IsNull {
            span: _,
            field,
            value: _,
        } => return tag.get(field.as_str()).is_null(),
        Expr::IsNotNull {
            span: _,
            field,
            value: _,
        } => return !tag.get(field.as_str()).is_null(),
        Expr::IsNumber {
            span: _,
            field,
            value: _,
        } => return tag.get(field.as_str()).is_number(),
        Expr::IsString {
            span: _,
            field,
            value: _,
        } => return tag.get(field.as_str()).is_string(),
        Expr::Join {
            from: _,
            expr,
            field: _,
            value: _,
        } => {
            if src.is_none() {
                return false;
            }
            let src = src.unwrap();

            // 例如 src.a = b 时，需要从 src 中找到 b 字段对应的值， 这里的表达式只能够是标准的比较
            match *expr {
                Expr::And { span: _, lhs, rhs } => {
                    return validate_match(None, src, *lhs) && validate_match(None, src, *rhs)
                }
                Expr::Or { span: _, lhs, rhs } => {
                    return validate_match(None, src, *lhs) || validate_match(None, src, *rhs)
                }
                Expr::Eq {
                    span: _,
                    field,
                    value,
                } => return src.get(field.as_str()) == condition_value_to_serde_value(value, src),
                Expr::Ne {
                    span: _,
                    field,
                    value,
                } => return src.get(field.as_str()) != condition_value_to_serde_value(value, src),
                Expr::Gt {
                    span: _,
                    field,
                    value,
                } => {
                    let value = condition_value_to_serde_value(value, src);
                    match src.get(field.as_str()) {
                        Value::Number(src_number) => {
                            if src_number.is_i64() {
                                return src_number.as_i64() > value.as_i64();
                            }
                            if src_number.is_u64() {
                                return src_number.as_u64() > value.as_u64();
                            }
                            if src_number.is_f64() {
                                return src_number.as_f64() > value.as_f64();
                            }
                            return false;
                        }
                        _ => return false,
                    }
                }
                Expr::Gte {
                    span: _,
                    field,
                    value,
                } => {
                    let value = condition_value_to_serde_value(value, src);
                    match src.get(field.as_str()) {
                        Value::Number(src_number) => {
                            if src_number.is_i64() {
                                return src_number.as_i64() >= value.as_i64();
                            }
                            if src_number.is_u64() {
                                return src_number.as_u64() >= value.as_u64();
                            }
                            if src_number.is_f64() {
                                return src_number.as_f64() >= value.as_f64();
                            }
                            return false;
                        }
                        _ => return false,
                    }
                }
                Expr::Lt {
                    span: _,
                    field,
                    value,
                } => {
                    let value = condition_value_to_serde_value(value, src);
                    match src.get(field.as_str()) {
                        Value::Number(src_number) => {
                            if src_number.is_i64() {
                                return src_number.as_i64() < value.as_i64();
                            }
                            if src_number.is_u64() {
                                return src_number.as_u64() < value.as_u64();
                            }
                            if src_number.is_f64() {
                                return src_number.as_f64() < value.as_f64();
                            }
                            return false;
                        }
                        _ => return false,
                    }
                }
                Expr::Lte {
                    span: _,
                    field,
                    value,
                } => {
                    let value = condition_value_to_serde_value(value, src);
                    match src.get(field.as_str()) {
                        Value::Number(src_number) => {
                            if src_number.is_i64() {
                                return src_number.as_i64() <= value.as_i64();
                            }
                            if src_number.is_u64() {
                                return src_number.as_u64() <= value.as_u64();
                            }
                            if src_number.is_f64() {
                                return src_number.as_f64() <= value.as_f64();
                            }
                            return false;
                        }
                        _ => return false,
                    }
                }
                Expr::In {
                    span: _,
                    field,
                    value,
                } => {
                    if let Some(value_list) = condition_value_to_serde_value(value, src).as_array()
                    {
                        return value_list.contains(&src.get(field.as_str()));
                    };
                    return false;
                }
                Expr::NotIn {
                    span: _,
                    field,
                    value,
                } => {
                    if let Some(value_list) = condition_value_to_serde_value(value, src).as_array()
                    {
                        return !value_list.contains(&src.get(field.as_str()));
                    };
                    return false;
                }
                _ => return false,
            }
        }
        Expr::LenField {
            span: _,
            field,
            compare,
            value,
        } => match compare {
            condition::Compare::EQ => {
                if let Some(value_list) = tag.get(field.as_str()).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() == required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::NE => {
                if let Some(value_list) = tag.get(field.as_str()).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() != required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::GT => {
                if let Some(value_list) = tag.get(field.as_str()).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() > required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::GTE => {
                if let Some(value_list) = tag.get(field.as_str()).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() >= required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::LT => {
                if let Some(value_list) = tag.get(field.as_str()).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() < required_length as usize;
                    }
                }
                return false;
            }
            condition::Compare::LTE => {
                if let Some(value_list) = tag.get(field.as_str()).as_array() {
                    if let Some(required_length) = value.as_i64() {
                        return value_list.len() <= required_length as usize;
                    }
                }
                return false;
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::from_str;
    use condition::Compare;
    use lrpar::Span;

    #[test]
    fn test_is_number() {
        let unstructed = &mut from_str(r#"{"a":123,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsNumber {
                field: "a".to_string(),
                value: true,
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_is_string() {
        let unstructed = &mut from_str(r#"{"a":"123","b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsString {
                field: "a".to_string(),
                value: true,
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_is_null() {
        let unstructed = &mut from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsNull {
                field: "a".to_string(),
                value: condition::Value::Null,
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_is_not_null() {
        let unstructed = &mut from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::IsNotNull {
                field: "b".to_string(),
                value: condition::Value::Null,
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_gte() {
        let unstructed = &mut from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::Gte {
                field: "b".to_string(),
                value: condition::Value::Number(serde_json::Number::from(312)),
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_lte() {
        let unstructed = &mut from_str(r#"{"a":null,"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::Lte {
                field: "b".to_string(),
                value: condition::Value::Number(serde_json::Number::from(999)),
                span: span,
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
                value: condition::Value::List(vec![
                    condition::Value::Text("a".to_string()),
                    condition::Value::Text("b".to_string()),
                ]),
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_not_in() {
        let unstructed = &mut from_str(r#"{"a":"c","b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::NotIn {
                field: "a".to_string(),
                value: condition::Value::List(vec![
                    condition::Value::Text("a".to_string()),
                    condition::Value::Text("b".to_string()),
                ]),
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_len() {
        let unstructed = &mut from_str(r#"{"a":[1,2,3],"b":312}"#).unwrap();
        let span = Span::new(1, 2);
        let res = validate_match(
            None,
            unstructed,
            Expr::LenField {
                field: "a".to_string(),
                value: serde_json::Number::from(10),
                compare: Compare::LT,
                span: span,
            },
        );
        assert!(res == true);
    }

    #[test]
    fn test_join() {
        let unstructed = &mut from_str(r#"{"a":[1,2,3],"b":312,"c":312}"#).unwrap();
        let span = Span::new(1, 2);

        let condition = Box::new(Expr::Gte {
            field: "b".to_string(),
            value: condition::Value::Field("c".to_string()),
            span: span,
        });

        let res = validate_match(
            Some(&mut unstructed.clone()),
            unstructed,
            Expr::Join {
                field: "".to_string(),
                from: "".to_string(),
                expr: condition,
                value: condition::Value::Null,
            },
        );
        assert!(res == true);
    }
}
