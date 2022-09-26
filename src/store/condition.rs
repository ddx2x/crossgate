use super::*;

pub struct Condition<'a> {
    pub db: &'a str,
    pub table: &'a str,
    pub page: usize,
    pub page_size: usize,
    pub sort: Vec<String>,
    pub other: Vec<(String, Value<'a>)>,
}

impl<'a> Condition<'a> {
    pub fn parse(query: Query<&str, Value<'a>>) -> Self {
        let mut db: &str = "";
        let mut table: &str = "";
        let mut page: usize = 0;
        let mut page_size: usize = 0;
        let mut sort = vec![];

        let mut other = vec![];

        for (k, v) in query {
            match k {
                DB => {
                    if let Value::String(s) = v {
                        db = s;
                    }
                }
                TABLE => {
                    if let Value::String(s) = v {
                        table = s
                    }
                }
                PAGE => {
                    if let Value::Number(s) = v {
                        page = s as usize
                    }
                }
                PAGE_SIZE => {
                    if let Value::Number(s) = v {
                        page_size = s as usize
                    }
                }
                SORT => {
                    if let Value::Array(s) = v {
                        for i in s {
                            if let Value::String(v1) = i {
                                sort.push(v1.to_string())
                            }
                        }
                    }
                }
                _ => other.push((k.to_string(), v)),
            }
        }

        Self {
            db,
            table,
            page,
            page_size,
            other,
            sort,
        }
    }
}
