use super::*;

pub struct Condition<'a> {
    pub db: &'a str,
    pub table: &'a str,
    pub uid: Option<&'a str>,
}

impl<'a> Condition<'a> {
    pub fn parse(query: Query<&str, Value<'a>>) -> Self {
        let mut _db: &str = "";
        let mut _table: &str = "";
        let mut _uid: Option<&str> = None;

        for (k, v) in query {
            match k {
                DB => {
                    if let Value::String(s) = v {
                        _db = s;
                    }
                }
                TABLE => {
                    if let Value::String(s) = v {
                        _table = s
                    }
                }
                UID => {
                    if let Value::String(s) = v {
                        _uid = Some(s)
                    }
                }
                _ => {}
            }
        }
        Self {
            db: _db,
            uid: _uid,
            table: _table,
        }
    }
}
