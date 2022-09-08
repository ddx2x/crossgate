use super::*;

pub struct Condition<'a> {
    pub db: &'a str,
    pub table: &'a str,
    pub other: Vec<(String, Value<'a>)>,
}

impl<'a> Condition<'a> {
    pub fn parse(query: Query<&str, Value<'a>>) -> Self {
        let mut db: &str = "";
        let mut table: &str = "";

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
                _ => other.push((k.to_string(), v)),
            }
        }

        Self { db, table, other }
    }
}
