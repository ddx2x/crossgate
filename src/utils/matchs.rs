use super::Unstructed;
use condition::Expr;
use serde_json::Value;

pub fn match_by_predicate<'a>(
    unstructeds: &'a mut Vec<Unstructed>,
    predicate: &str,
) -> anyhow::Result<&'a mut Vec<Unstructed>> {
    matchs(unstructeds, condition::parse(predicate)?)
}

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
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            return s.gt(t);
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
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            return s.ge(t);
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
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            return s.lt(t);
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
                    condition::Value::Text(t) => {
                        if let Value::String(s) = s {
                            return s.le(t);
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
        Expr::IsNotNull { field, .. } => {
            if unstructed.get_by_type::<Value>(field, Value::Null) != Value::Null {
                return true;
            }
            return false;
        }
        Expr::IsNull { field, .. } => {
            if unstructed.get_by_type::<Value>(field, Value::Null) == Value::Null {
                return true;
            }
            return false;
        }
        Expr::Len {
            field, cmp, value, ..
        } => {
            let len = match value {
                condition::Value::Number(v) => v.as_i64(),
                _ => return false,
            };
            let real = match unstructed.get_by_type::<Value>(field, Value::Array(vec![])) {
                Value::String(v) => v.len() as i64,
                Value::Array(v) => v.len() as i64,
                Value::Object(v) => v.len() as i64,
                _ => return false,
            };
            match cmp {
                condition::Compare::Eq => len == Some(real),
                condition::Compare::Ne => len != Some(real),
                condition::Compare::Gt => Some(real) > len,
                condition::Compare::Gte => Some(real) >= len,
                condition::Compare::Lt => Some(real) < len,
                condition::Compare::Lte => Some(real) <= len,
            }
        }
        Expr::Belong { field, value, .. } => {
            if let condition::Value::List(rhs_list) = value {
                let lhs_list = unstructed.get_by_type::<Vec<Value>>(&field, vec![]);
                if lhs_list.len() == 0 || rhs_list.len() == 0 {
                    return false;
                }

                let rhs_list = rhs_list
                    .iter()
                    .map(|item| match item {
                        condition::Value::Text(v) => Value::String(v.clone()),
                        condition::Value::Number(v) => Value::Number(v.clone()),
                        _ => Value::Null,
                    })
                    .collect::<Vec<_>>();

                let mut hits = vec![];

                let include = for<'a> |item: &'a Value, set: Vec<Value>| -> bool {
                    return set.contains(&item);
                };

                for lhs in lhs_list {
                    hits.push(include(&lhs, rhs_list.clone()));
                }

                return !hits.contains(&false);
            }

            return false;
        }
        Expr::NoBelong { field, value, .. } => {
            if let condition::Value::List(rhs_list) = value {
                let lhs_list = unstructed.get_by_type::<Vec<Value>>(&field, vec![]);
                if lhs_list.len() == 0 || rhs_list.len() == 0 {
                    return false;
                }

                let rhs_list = rhs_list
                    .iter()
                    .map(|item| match item {
                        condition::Value::Text(v) => Value::String(v.clone()),
                        condition::Value::Number(v) => Value::Number(v.clone()),
                        _ => Value::Null,
                    })
                    .collect::<Vec<_>>();

                let mut hits = vec![];

                let include = for<'a> |item: &'a Value, set: Vec<Value>| -> bool {
                    return set.contains(&item);
                };

                //[1,4] << [1,2,3] => true  左边的元素不属于右边的元素
                for lhs in lhs_list {
                    hits.push(include(&lhs, rhs_list.clone()));
                }
                return hits.contains(&false);
            }

            return true;
        }
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

    #[test]
    fn test_in_string() {
        let datas = vec![
            from_str(r#"{"name":"bobo","active":false}"#).unwrap(),
            from_str(r#"{"name":"bill","active":false}"#).unwrap(),
            from_str(r#"{"name":"alex","active":true}"#).unwrap(),
        ];

        // where name in ("bobo","bill")
        match matchs(
            &mut datas.clone(),
            parse(r#"name ~ ("bobo","bill")"#).unwrap(),
        ) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }

                println!("bool data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_null() {
        let datas = vec![
            from_str(r#"{"name":"bobo","active":null}"#).unwrap(),
            from_str(r#"{"name":"bill","active":false}"#).unwrap(),
            from_str(r#"{"name":"alex","active":true}"#).unwrap(),
        ];

        // where active is null
        match matchs(&mut datas.clone(), parse(r#"active ^ null"#).unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
                println!("data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        // where active is not null
        match matchs(&mut datas.clone(), parse(r#"active ^^ null"#).unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }
                println!("data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_len() {
        let datas = vec![
            from_str(r#"{"name":"bobo","active":null}"#).unwrap(),
            from_str(r#"{"name":"bill","active":false}"#).unwrap(),
            from_str(r#"{"name":"alex","active":true}"#).unwrap(),
        ];

        // where len(name) = 4
        match matchs(&mut datas.clone(), parse(r#"len(name) = 4 "#).unwrap()) {
            Ok(r) => {
                if r.len() != 3 {
                    panic!("Inconsistent expected results")
                }
                println!("data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        let datas = vec![from_str(r#"{"name":"bobo","ids":[1,2,3]}"#).unwrap()];
        // where len(ids) = 3
        match matchs(&mut datas.clone(), parse(r#"len(ids) = 3 "#).unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
                println!("data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        let datas = vec![from_str(r#"{"name":"bobo","obj":{"a":1}}"#).unwrap()];
        // where len(obj) = 1
        match matchs(&mut datas.clone(), parse(r#"len(obj) = 1 "#).unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
                println!("data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_belong() {
        let datas = vec![from_str(r#"{"alist":[1,2]}"#).unwrap()];

        match matchs(&mut datas.clone(), parse(r#"alist<<(1,2,3)"#).unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }

                println!("test_belong data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        match matchs(&mut datas.clone(), parse(r#"alist<<(1,3)"#).unwrap()) {
            Ok(r) => {
                if r.len() != 0 {
                    panic!("Inconsistent expected results")
                }

                println!("test_belong data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_no_belong() {
        // [1,4] >> [1,2,3]  左边的元素不属于右边的元素
        let datas = vec![from_str(r#"{"alist":[1,4]}"#).unwrap()];
        match matchs(&mut datas.clone(), parse(r#"alist >> (1,2,3)"#).unwrap()) {
            Ok(r) => {
                println!("test_nobelong data {:?}", r);
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
            }
            Err(e) => eprintln!("simulation data error: {}", e),
        }

        // [4] >> [1,2,3]  左边的元素不属于右边的元素
        let datas = vec![from_str(r#"{"alist":[4]}"#).unwrap()];
        match matchs(&mut datas.clone(), parse(r#"alist >> (1,2,3)"#).unwrap()) {
            Ok(r) => {
                println!("test_nobelong data {:?}", r);
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_mix() {
        let datas = vec![from_str(
            r#"
        {
            "uid": "Abc",
            "level": 2,
            "parent_id": "汽车养护",
            "full_id": "",
            "nav_status": 1,
            "keywords": [
                "ABC"
            ],
            "description": ""
        }
        "#,
        )
        .unwrap()];

        match matchs(
            &mut datas.clone(),
            parse(r#"level ~ (1,2,3) && nav_status ~ (1,2) && len(keywords) > 0 && len(uid) > 0"#)
                .unwrap(),
        ) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }

                println!("test data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn my_test() {
        let datas = vec![from_str(
            r#"
        {
            "uid": "64535b7c0c28142880602cca",
            "channel_name": "VIP普通用户",
            "level": 1,
            "direct_subordinate": 1,
            "indirect_subordinate": 1,
            "share_ratio": 1,
            "can_purchased": true,
            "price": 1
        }
        "#,
        )
        .unwrap()];

        match matchs(
            &mut datas.clone(),
            parse(
                r#" len(channel_name) >=2 && len(channel_name) <=64 && level>=1 && level <=100 "#,
            )
            .unwrap(),
        ) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }

                println!("test data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        match matchs(&mut datas.clone(), parse(r#" len(uid) > 0"#).unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }

                println!("test data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }

    #[test]
    fn test_string_compare() {
        let datas = vec![
            from_str(
                r#"
        {
            "uid": "2"
        }
        "#,
            )
            .unwrap(),
            from_str(
                r#"
        {
            "uid": "1"
        }
        "#,
            )
            .unwrap(),
        ];

        match matchs(&mut datas.clone(), parse(r#"uid > '1'"#).unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }

                println!("test data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        match matchs(&mut datas.clone(), parse(r#"uid >= '1'"#).unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }

                println!("test data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        match matchs(&mut datas.clone(), parse(r#"uid < '2'"#).unwrap()) {
            Ok(r) => {
                if r.len() != 1 {
                    panic!("Inconsistent expected results")
                }

                println!("test data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }

        match matchs(&mut datas.clone(), parse(r#"uid <= '2'"#).unwrap()) {
            Ok(r) => {
                if r.len() != 2 {
                    panic!("Inconsistent expected results")
                }

                println!("test data {:?}", r);
            }
            Err(e) => panic!("simulation data error: {}", e),
        }
    }
}
