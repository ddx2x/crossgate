use condition::Validate as Expr;

use super::Unstructed;

pub fn validate_match<'a>(
    src: &Option<&'a mut Unstructed>, // 表示当前已经持久化的集合
    tag: &'a mut Unstructed,          // 表示当前待更新的集合
    expr: Expr,
) -> bool {
    match expr {
        Expr::And { span: _, lhs, rhs } => {
            return validate_match(src, tag, *lhs) && validate_match(src, tag, *rhs)
        }
        Expr::Or { span: _, lhs, rhs } => {
            return validate_match(src, tag, *lhs) || validate_match(src, tag, *rhs)
        }
        Expr::Eq {
            span: _,
            field,
            value,
        } => return value == tag.get(field.as_str()),
        Expr::Ne {
            span: _,
            field,
            value,
        } => {
            return value != tag.get(field.as_str());
        }
        Expr::Gt {
            span: _,
            field,
            value,
        } => {
            if let (Some(compare_value), Some(tag_num)) =
                (value.as_f64(), tag.get(field.as_str()).as_f64())
            {
                return tag_num > compare_value;
            }
            return false;
        }
        Expr::Gte {
            span: _,
            field,
            value,
        } => {
            if let (Some(compare_value), Some(tag_num)) =
                (value.as_f64(), tag.get(field.as_str()).as_f64())
            {
                return tag_num >= compare_value;
            }
            return false;
        }
        Expr::Lt {
            span: _,
            field,
            value,
        } => {
            if let (Some(compare_value), Some(tag_num)) =
                (value.as_f64(), tag.get(field.as_str()).as_f64())
            {
                return tag_num < compare_value;
            }
            return false;
        }
        Expr::Lte {
            span: _,
            field,
            value,
        } => {
            if let (Some(compare_value), Some(tag_num)) =
                (value.as_f64(), tag.get(field.as_str()).as_f64())
            {
                return tag_num <= compare_value;
            }
            return false;
        }
        Expr::In {
            span: _,
            field,
            value,
        } => {
            if let Some(value_list) = value.as_array() {
                return value_list.contains(&tag.get(field.as_str()));
            };
            return false;
        }
        Expr::NotIn {
            span: _,
            field,
            value,
        } => {
            if let Some(value_list) = value.as_array() {
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
            field,
            value,
        } => {
            if src.is_none() {
                return false;
            }

            // 例如 src.a = b 时，需要从 src 中找到 b 字段对应的值， 这里的表达式只能够是标准的比较
            match expr {
                // Expr::And { span: _, lhs, rhs } => todo!(),
                // Expr::Or { span: _, lhs, rhs } => todo!(),
                // Expr::Eq {
                //     span: _,
                //     field,
                //     value,
                // } => todo!(),
                // Expr::Ne {
                //     span: _,
                //     field,
                //     value,
                // } => todo!(),
                // Expr::Gt {
                //     span: _,
                //     field,
                //     value,
                // } => todo!(),
                // Expr::Gte {
                //     span: _,
                //     field,
                //     value,
                // } => todo!(),
                // Expr::Lt {
                //     span: _,
                //     field,
                //     value,
                // } => todo!(),
                // Expr::Lte {
                //     span: _,
                //     field,
                //     value,
                // } => todo!(),
                // Expr::In {
                //     span: _,
                //     field,
                //     value,
                // } => todo!(),
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
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_number() {}

    #[test]
    fn test_is_string() {}

    #[test]
    fn test_is_null() {}

    #[test]
    fn test_is_not_null() {}

    #[test]
    fn test_join() {}
}
