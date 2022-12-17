use condition::Expr;
use serde_json::Value;

use super::Unstructed;

pub fn matchs<'a>(
    unstructeds: &'a mut Vec<Unstructed>,
    exprs: &'a [Expr],
) -> anyhow::Result<&'a mut Vec<Unstructed>> {
    let mut remove_indexs = vec![];
    for (index, unstructed) in unstructeds.into_iter().enumerate() {
        for expr in exprs {
            if !_match(unstructed, expr) {
                remove_indexs.push(index);
            }
        }
    }

    for index in remove_indexs {
        unstructeds.remove(index);
    }

    Ok(unstructeds)
}

fn _match(unstructed: &Unstructed, expr: &Expr) -> bool {
    match expr {
        Expr::And { lhs, rhs, .. } => return _match(unstructed, lhs) && _match(unstructed, rhs),
        Expr::Or { lhs, rhs, .. } => return _match(unstructed, lhs) || _match(unstructed, rhs),
        Expr::Eq { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            return s.eq(t);
                        }
                        return false;
                    }
                    condition::Value::Number(t) => {
                        if let Value::Number(s) = s {
                            return s.eq(t);
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::Ne { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            return !s.eq(t);
                        }
                        return false;
                    }
                    condition::Value::Number(t) => {
                        if let Value::Number(s) = s {
                            return !s.eq(t);
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::Gt { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Number(t) => {
                        if let Value::Number(s) = s {
                            return s.as_f64() > t.as_f64();
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::Gte { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Number(t) => {
                        if let Value::Number(s) = s {
                            return s.as_f64() >= t.as_f64();
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::Lt { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Number(t) => {
                        if let Value::Number(s) = s {
                            return s.as_f64() < t.as_f64();
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::Lte { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Number(t) => {
                        if let Value::Number(s) = s {
                            return s.as_f64() <= t.as_f64();
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::Like { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            if let Ok(r) = regex::Regex::new(s) {
                                return r.is_match(t);
                            } else {
                                return false;
                            }
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::NotLike { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            if let Ok(r) = regex::Regex::new(s) {
                                return !r.is_match(t);
                            } else {
                                return false;
                            }
                        }
                        return false;
                    }
                    _ => return false,
                }
            }

            return false;
        }
        Expr::In { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::List(tvs) => {
                        let mut is_match = false;

                        if let Value::Array(vs) = s {
                            for tv in tvs {
                                if is_match {
                                    break;
                                }

                                for v in vs {
                                    if is_match {
                                        break;
                                    }
                                    match tv {
                                        condition::Value::Text(tv) => {
                                            if let Value::String(v) = v {
                                                is_match = v.eq(tv);
                                            } else {
                                                is_match = false;
                                            }
                                        }
                                        condition::Value::Number(tv) => {
                                            if let Value::Number(s) = v {
                                                is_match = s.eq(tv);
                                            } else {
                                                is_match = false;
                                            }
                                        }
                                        _ => is_match = false,
                                    };
                                }
                            }
                        }
                    }
                    _ => return false,
                }
            }
            return false;
        }
        Expr::NotIn { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::List(tvs) => {
                        let mut is_match = false;

                        if let Value::Array(vs) = s {
                            for tv in tvs {
                                if is_match {
                                    break;
                                }

                                for v in vs {
                                    if is_match {
                                        break;
                                    }
                                    match tv {
                                        condition::Value::Text(tv) => {
                                            if let Value::String(v) = v {
                                                is_match = !v.eq(tv);
                                            } else {
                                                is_match = false;
                                            }
                                        }
                                        condition::Value::Number(tv) => {
                                            if let Value::Number(s) = v {
                                                is_match = !s.eq(tv);
                                            } else {
                                                is_match = false;
                                            }
                                        }
                                        _ => is_match = false,
                                    };
                                }
                            }
                        }
                    }
                    _ => return false,
                }
            }
            return false;
        }
        Expr::IsNull { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Bool(t) => {
                        if let Value::Bool(s) = s {
                            return s.eq(t);
                        }
                        return false;
                    }

                    _ => return false,
                }
            }

            return false;
        }
        Expr::IsNotNull { field, value, .. } => {
            if let Some(s) = unstructed.0.get(field) {
                match value {
                    condition::Value::Bool(t) => {
                        if let Value::Bool(s) = s {
                            return !s.eq(t);
                        }
                        return false;
                    }

                    _ => return false,
                }
            }

            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use condition::parse;

    use crate::utils::from_str;

    use super::matchs;

    #[test]
    fn test_basic() {
        match from_str(r#"{"a":123}"#) {
            Ok(u) => {
                let mut us = vec![u];
                let exprs = &vec![parse("a=123").unwrap()];
                match matchs(&mut us, &exprs) {
                    Ok(r) => {
                        if r.len() != 1 {
                            panic!("Inconsistent expected results")
                        }
                    }
                    Err(e) => panic!("simulation data error: {}", e),
                }
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }
}
