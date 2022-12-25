use condition::Expr;
use serde_json::Value;

use super::Unstructed;

pub fn matchs<'a>(
    unstructeds: &'a mut Vec<Unstructed>,
    expr: Expr,
) -> anyhow::Result<&'a mut Vec<Unstructed>> {
    let mut remove_indexs = vec![];
    for (index, unstructed) in unstructeds.into_iter().enumerate() {
        if !filter(unstructed, &expr) {
            remove_indexs.push(index);
        }
    }
    remove_indexs.sort_by_key(|n| std::usize::MAX - n);

    for index in remove_indexs {
        unstructeds.remove(index);
    }

    Ok(unstructeds)
}

fn filter(unstructed: &Unstructed, expr: &Expr) -> bool {
    match expr {
        Expr::And { lhs, rhs, .. } => return filter(unstructed, lhs) && filter(unstructed, rhs),
        Expr::Or { lhs, rhs, .. } => return filter(unstructed, lhs) || filter(unstructed, rhs),
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
                                    if !v.eq(s) {
                                        continue;
                                    }
                                    return true;
                                }
                            }
                        }
                        Value::String(s) => {
                            for item in condition_list {
                                if let condition::Value::Text(v) = item {
                                    if !v.eq(s) {
                                        continue;
                                    }
                                    return true;
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
                            let mut hits = vec![];
                            for item in condition_list {
                                if let condition::Value::Number(v) = item {
                                    hits.push(!v.eq(s));
                                }
                            }
                            return hits.iter().fold(true, |acc, e| (acc == *e) && (*e == true));
                        }
                        Value::String(s) => {
                            let mut hits = vec![];
                            for item in condition_list {
                                if let condition::Value::Text(v) = item {
                                    hits.push(!v.eq(s));
                                }
                            }
                            return hits.iter().fold(true, |acc, e| (acc == *e) && (*e == true));
                        }
                        _ => return false,
                    }
                }
                return false;
            }

            return false;
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::matchs;
    use crate::utils::from_str;
    use condition::parse;

    #[test]
    fn test_eq() {
        let datas = vec![
            from_str(r#"{"a":123,"b":312}"#).unwrap(),
            from_str(r#"{"a":123,"d":111}"#).unwrap(),
        ];

        match matchs(&mut datas.clone(), parse("a=123").unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        match matchs(&mut datas.clone(), parse("a!=123").unwrap()) {
            Ok(r) => {
                if r.len() != 0 {
                    panic!("Inconsistent expected results")
                }
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_in() {
        let datas = vec![
            from_str(r#"{"a":1,"b":2}"#).unwrap(),
            from_str(r#"{"a":3,"d":3}"#).unwrap(),
            from_str(r#"{"a":4,"d":3}"#).unwrap(),
        ];

        // where a in (1,3)
        match matchs(&mut datas.clone(), parse("a ~ (1,3)").unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }
                println!("in result {:?}", r)
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where a notin (1,3)
        match matchs(&mut datas.clone(), parse("a ~~ (1,3)").unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
                //expect result {"a":4,"d":3}
                println!("not in result {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_like() {
        let datas = vec![
            from_str(r#"{"name":"bobo","src":2}"#).unwrap(),
            from_str(r#"{"name":"bill","src":3}"#).unwrap(),
            from_str(r#"{"name":"alex","src":3}"#).unwrap(),
        ];

        // where name like '^b'
        match matchs(&mut datas.clone(), parse("name ! '^b.'").unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }

                println!("like data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where name no like '^b'
        match matchs(&mut datas.clone(), parse("name !! '^b.'").unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
                println!("not like data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_compare() {
        let datas = vec![
            from_str(r#"{"name":"bobo","src":2}"#).unwrap(),
            from_str(r#"{"name":"bill","src":3}"#).unwrap(),
            from_str(r#"{"name":"alex","src":3}"#).unwrap(),
        ];

        // where src>2
        match matchs(&mut datas.clone(), parse("src>2").unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }

                println!("gt data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where src>=2
        match matchs(&mut datas.clone(), parse("src>=2").unwrap()) {
            Ok(r) => {
                if r.len() != 3 {
                    panic!("Inconsistent expected results")
                }
                println!("gte data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where src<3
        match matchs(&mut datas.clone(), parse("src<3").unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
                println!("lt data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where src<=2
        match matchs(&mut datas.clone(), parse("src<=2").unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
                println!("lte data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_joint() {
        let datas = vec![
            from_str(r#"{"name":"bobo","src":2}"#).unwrap(),
            from_str(r#"{"name":"bill","src":3}"#).unwrap(),
            from_str(r#"{"name":"alex","src":3}"#).unwrap(),
        ];

        // where name like '^a' or src=3
        match matchs(&mut datas.clone(), parse("name ! '^a.' || src=3").unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }

                println!("joint data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_bool() {
        let datas = vec![
            from_str(r#"{"name":"bobo","active":false}"#).unwrap(),
            from_str(r#"{"name":"bill","active":false}"#).unwrap(),
            from_str(r#"{"name":"alex","active":true}"#).unwrap(),
        ];

        // where active is true
        match matchs(&mut datas.clone(), parse("active = true").unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }

                println!("bool data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }
}
