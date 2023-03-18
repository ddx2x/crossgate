use condition::Validate as Expr;

use super::Unstructed;

pub fn validate_match<'a>(
    src: Option<&'a mut Unstructed>,
    tag: &'a mut Unstructed,
    expr: Expr,
) -> bool {
    match expr.clone() {
        Expr::And { span: _, lhs, rhs } => match src {
            Some(src) => {
                let l_result = validate_match(Some(src), tag, *lhs);
                let r_result = validate_match(Some(src), tag, *rhs);
                return l_result && r_result;
            }
            None => {
                let l_result = validate_match(None, tag, *lhs);
                let r_result = validate_match(None, tag, *rhs);
                return l_result && r_result;
            }
        },
        Expr::Or { span: _, lhs, rhs } => match src {
            Some(src) => {
                let l_result = validate_match(Some(src), tag, *lhs);
                let r_result = validate_match(Some(src), tag, *rhs);
                return l_result || r_result;
            }
            None => {
                let l_result = validate_match(None, tag, *lhs);
                let r_result = validate_match(None, tag, *rhs);
                return l_result || r_result;
            }
        },
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
        } => return tag.get(field.as_str()).is_null(),
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
            if let Some(src) = src {
                return validate_match(None, src, *expr) == tag.get(field.as_str());
            }
            return tag.get(field.as_str()) == value;
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
