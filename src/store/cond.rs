use std::{any, fmt::format};

use super::*;
use bson::{doc, Bson, Document};
use condition::parse;

pub enum MongoOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Ne,
}

#[derive(Debug)]
pub struct Condition<'a> {
    pub db: &'a str,
    pub table: &'a str,
    pub page: usize,
    pub page_size: usize,
    pub sort: Vec<String>,

    pub other: Vec<(String, Value<'a>)>,

    filter: Document,
}

impl<'a> Condition<'a> {
    pub fn new() -> Self {
        Self {
            db: Default::default(),
            table: Default::default(),
            page: 0,
            page_size: 10,
            sort: Default::default(),
            other: Default::default(),
            filter: doc! {},
        }
    }
    pub fn db(&mut self, db: &'a str) -> &mut Condition<'a> {
        self.db = db;
        self
    }

    pub fn parse_by_text(&mut self, input: &'a str) -> anyhow::Result<&mut Condition<'a>> {
        let expr = match parse(input) {
            Ok(s) => s,
            Err(e) => return Err(anyhow::anyhow!("{:?}", e)),
        };

        self.filter = self.eval(&[expr])?.into_iter().flatten().collect();

        Ok(self)
    }

    fn gen_doc(k: &str, v: &condition::Value, op: MongoOp) -> anyhow::Result<Document> {
        let mut doc = doc! {};
        let v = match v {
            condition::Value::Text(v) => v.as_str().to_string(),
            condition::Value::Number(v) => format!("{}", v),
            condition::Value::Bool(v) => format!("{}", v),
            _ => return Err(anyhow::anyhow!("unsupport type parse")),
        };

        match op {
            MongoOp::Eq => doc.insert(k, v),
            MongoOp::Gt => doc.insert("$gt", doc! {k:v}),
            MongoOp::Gte => doc.insert("$gte", doc! {k:v}),
            MongoOp::Lt => doc.insert("$lt", doc! {k:v}),
            MongoOp::Lte => doc.insert("$lte", doc! {k:v}),
            MongoOp::Ne => doc.insert("$ne", doc! {k:v}),
        };

        Ok(doc)
    }

    fn eval(&mut self, exprs: &[condition::Expr]) -> anyhow::Result<Vec<bson::Document>> {
        let mut docs = vec![];
        for expr in exprs.into_iter() {
            match expr {
                condition::Expr::And { span: _, lhs, rhs } => {
                    let mut doc = doc! {};
                    doc.insert("$and", self.eval(&[*lhs.clone(), *rhs.clone()])?);
                    docs.push(doc);
                }
                condition::Expr::Or { span: _, lhs, rhs } => {
                    let mut doc = doc! {};
                    doc.insert("$or", self.eval(&[*lhs.clone(), *rhs.clone()])?);
                    docs.push(doc);
                }
                condition::Expr::Eq {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::Eq)?);
                }
                condition::Expr::Ne {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::Ne)?);
                }
                condition::Expr::Gt {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::Gt)?);
                }
                condition::Expr::Gte {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::Gte)?);
                }
                condition::Expr::Lt {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::Lt)?);
                }
                condition::Expr::Lte {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::Lte)?);
                }
                _ => return Err(anyhow::anyhow!("not support op")),
            }
        }

        Ok(docs)
    }

    pub fn parse(query: Query<&str, Value<'a>>) -> Self {
        let mut db: &str = "";
        let mut table: &str = "";
        let mut page: usize = 0;
        let mut page_size: usize = 0;
        let mut sort = vec![];

        let mut other = vec![];

        let filter = doc! {};

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
            filter,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_cond() {
        let sym = "a=1&&b=2||c=1&&b=2";
        let mut cond = Condition::new();
        match cond.parse_by_text(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }
}
