use crate::store::{Condition, Filter};
use bson::{doc, Document};
use condition::parse;

use super::GetFilter;

pub enum MongoOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Ne,
}
#[derive(Clone, Debug)]
pub struct MongoFilter(pub Document);

impl MongoFilter {
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

    fn eval(&self, exprs: &[condition::Expr]) -> anyhow::Result<Vec<bson::Document>> {
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
}

impl Filter for MongoFilter {
    fn parse(&mut self, input: &str) -> anyhow::Result<Box<Self>> {
        let expr = match parse(input) {
            Ok(s) => s,
            Err(e) => return Err(anyhow::anyhow!("{:?}", e)),
        };
        self.0 = self.eval(&[expr])?.into_iter().flatten().collect();

        Ok(Box::new(self.clone()))
    }
}

impl GetFilter for MongoFilter {
    fn get(self) -> Document {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_cond() {
        let sym = "a=1&&b=2||c=1&&b=2";
        let mut mf = MongoFilter(doc! {});
        match mf.parse(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }
}