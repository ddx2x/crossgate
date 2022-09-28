use crate::store::Filter;
use bson::{doc, Document};
use condition::parse;
use mongodb::bson::Bson;

use super::GetFilter;

pub enum MongoOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Ne,
    Like,
    NotLike, // { item: { $not: { $regex: "^p.*" } } }
    In,
    NotIn,
}
#[derive(Clone, Debug)]
pub struct MongoFilter(pub Document);

impl MongoFilter {
    fn gen_doc(k: &str, v: &condition::Value, op: MongoOp) -> anyhow::Result<Document> {
        let op = match op {
            MongoOp::Eq => "$eq",
            MongoOp::Gt => "$gt",
            MongoOp::Gte => "$gt",
            MongoOp::Lt => "$lt",
            MongoOp::Lte => "$lte",
            MongoOp::Ne => "$ne",
            MongoOp::Like => "$regex",
            MongoOp::In => "$in",
            MongoOp::NotIn => "$nin",

            MongoOp::NotLike => {
                //{ item: { $not: { $regex: "^p.*" } } }
                panic!("notlike not supported")
            }
        };

        let mut doc = doc! {};

        if op == "$in" || op == "$nin" {
            let mut str_vec = vec![];
            let mut int_vec = vec![];

            if let condition::Value::List(vs) = v {
                for v in vs {
                    match v {
                        condition::Value::Text(v) => {
                            str_vec.push(v.as_str().to_string());
                        }
                        condition::Value::Number(v) => {
                            int_vec.push(Bson::Int64(*v as i64));
                        }
                        _ => return Err(anyhow::anyhow!("in op unsupport non int or charts")),
                    }
                }
            } else {
                return Err(anyhow::anyhow!("in op just only support list"));
            }

            if str_vec.len() > 0 && int_vec.len() > 0 {
                return Err(anyhow::anyhow!(
                    "only supports the same type of int or charts in the list"
                ));
            }

            if str_vec.len() > 0 {
                doc.insert(k, doc! {op:str_vec});
            } else {
                doc.insert(k, doc! {op:int_vec});
            }

            return Ok(doc);
        }

        // like
        if op == "$regex" {
            if let condition::Value::Text(s) = v {
                doc.insert(k, doc! {op:format!("/{}/",s.as_str())});
            } else {
                return Err(anyhow::anyhow!("in op just only support text"));
            }
            return Ok(doc);
        }

        doc = match v {
            condition::Value::Text(v) => doc! {k:doc! {op:v.as_str().to_string()}},
            condition::Value::Number(v) => doc! {k:doc!{op:Bson::Int64(*v as i64)}},
            condition::Value::Bool(v) => doc! {k:doc!{op:Bson::Boolean(*v)}},
            _ => return Err(anyhow::anyhow!("unsupport type parse")),
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
                condition::Expr::Like {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::Like)?);
                }
                condition::Expr::NotLike {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::NotLike)?);
                }
                condition::Expr::In {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::In)?);
                }
                condition::Expr::NotIn {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(field.as_str(), value, MongoOp::NotIn)?);
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
        let sym = "a=1 && b=2 || c=1 && b=2";
        let mut mf = MongoFilter(doc! {});
        match mf.parse(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_parse_cond2() {
        let sym = "a=1 && (b=2||c=1) && b=2";
        let mut mf = MongoFilter(doc! {});
        match mf.parse(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_parse_cond3() {
        let sym = "a=1 && (b=2 || c=1 && b=2)";
        let mut mf = MongoFilter(doc! {});
        match mf.parse(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_parse_in_notin() {
        let mut mf = MongoFilter(doc! {});
        match mf.parse("a ~ (1,2,3,4)") {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };

        match mf.parse("a ~~ (1,2,3,4)") {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_parse_like() {
        let mut mf = MongoFilter(doc! {});
        match mf.parse("a ! '^1.2'") {
            // prefix 1.2
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };
    }
}
