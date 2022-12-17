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
            if _match(unstructed, expr) {
                break;
            }
            remove_indexs.push(index);
        }
    }

    remove_indexs.sort_by_key(|n| std::usize::MAX - n);

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
                            if let Ok(r) = regex::Regex::new(t) {
                                return r.is_match(s);
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
                            if let Ok(r) = regex::Regex::new(t) {
                                return !r.is_match(s);
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
            if let condition::Value::List(condition_list) = value {
                if let Some(s) = unstructed.0.get(field) {
                    match s {
                        Value::Number(s) => {
                            for item in condition_list {
                                if let condition::Value::Number(v) = item {
                                    if v.eq(s) {
                                        return true;
                                    }
                                }
                            }
                        }
                        Value::String(s) => {
                            for item in condition_list {
                                if let condition::Value::Text(v) = item {
                                    if v.eq(s) {
                                        return true;
                                    }
                                }
                            }
                        }
                        _ => return false,
                    }
                }
                return false;
            }

            return false;
        }
        Expr::NotIn { field, value, .. } => {
            if let condition::Value::List(condition_list) = value {
                if let Some(s) = unstructed.0.get(field) {
                    match s {
                        Value::Number(s) => {
                            for item in condition_list {
                                if let condition::Value::Number(v) = item {
                                    if !v.eq(s) {
                                        return true;
                                    }
                                }
                            }
                        }
                        Value::String(s) => {
                            for item in condition_list {
                                if let condition::Value::Text(v) = item {
                                    if !v.eq(s) {
                                        return true;
                                    }
                                }
                            }
                        }
                        _ => return false,
                    }
                }
                return false;
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
    fn test_eq() {
        let mut datas = vec![
            from_str(r#"{"a":123,"b":312}"#).unwrap(),
            from_str(r#"{"a":123,"d":111}"#).unwrap(),
        ];

        let exprs = &vec![parse("a=123").unwrap()];

        match matchs(&mut datas, &exprs) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_in() {
        let mut datas = vec![
            from_str(r#"{"a":1,"b":2}"#).unwrap(),
            from_str(r#"{"a":3,"d":3}"#).unwrap(),
        ];

        // where a in (1,3)
        let exprs = &vec![parse("a ~ (1,3)").unwrap()];

        match matchs(&mut datas, &exprs) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where a notin (1,3)
        let exprs = &vec![parse("a ~~ (1,3)").unwrap()];

        match matchs(&mut datas, &exprs) {
            Ok(r) => {
                if r.len() == 0 {
                    panic!("Inconsistent expected results")
                }
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_like() {
        let mut datas = vec![
            from_str(r#"{"name":"bobo","src":2}"#).unwrap(),
            from_str(r#"{"name":"bill","src":3}"#).unwrap(),
            from_str(r#"{"name":"alex","src":3}"#).unwrap(),
        ];

        // where name like '^b'
        let exprs = &vec![parse("name ! '^b.'").unwrap()];

        match matchs(&mut datas, &exprs) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }

                println!("like data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where name no like '^b'
        let exprs = &vec![parse("name !! '^b.'").unwrap()];

        match matchs(&mut datas, &exprs) {
            Ok(r) => {
                if r.len() != 0 {
                    panic!("Inconsistent expected results")
                }
                println!("not like data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }
}
