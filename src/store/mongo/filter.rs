use std::str::FromStr;

use crate::store::Filter;
use bson::{doc, oid::ObjectId, Document};
use condition::yacc_parse as parse;
use mongodb::bson::Bson;

use super::{convert_to_mongodb_time, GetFilter};

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
pub struct MongoFilter(pub Document, pub String, pub bool);

impl MongoFilter {
    fn gen_doc(
        enable_convert: &bool,
        k: &str,
        v: &condition::Value,
        op: MongoOp,
    ) -> anyhow::Result<Document> {
        let op = match op {
            MongoOp::Eq => "$eq",
            MongoOp::Gt => "$gt",
            MongoOp::Gte => "$gte",
            MongoOp::Lt => "$lt",
            MongoOp::Lte => "$lte",
            MongoOp::Ne => "$ne",
            MongoOp::Like => "$regex",
            MongoOp::In => "$in",
            MongoOp::NotIn => "$nin",

            MongoOp::NotLike => {
                //{ item: { $not: { $regex: "^p.*" } } }
                return Err(anyhow::anyhow!("notlike not supported"));
            }
        };

        let mut doc = doc! {};

        if op == "$in" || op == "$nin" {
            let mut str_vec = vec![];
            let mut object_id_vec = vec![];
            let mut number_vec = vec![];
            let mut iosdate_vec = vec![];

            if let condition::Value::List(vs) = v {
                for v in vs {
                    match v {
                        condition::Value::Text(v) => {
                            log::debug!("filter enable_convert: {}, k: {}", enable_convert, k);
                            if *enable_convert && k.eq("_id") {
                                object_id_vec.push(ObjectId::from_str(v.as_str())?);
                                continue;
                            }

                            if v.starts_with("ISODate(") && v.ends_with(")") {
                                // ISODate(2022-06-13 16:00:00) => mongodb: 2022-06-13T16:00:00Z
                                let new_v = v.replace("ISODate(", "").replace(")", "");
                                match convert_to_mongodb_time(&new_v) {
                                    Ok(dt) => iosdate_vec.push(Bson::from(dt)),
                                    Err(e) => return Err(anyhow::anyhow!("{:?}", e)),
                                }
                                continue;
                            }

                            str_vec.push(v.as_str().to_string());
                        }
                        condition::Value::Number(v) => {
                            if v.is_f64() {
                                if let Some(v) = v.as_f64() {
                                    number_vec.push(Bson::from(v));
                                }
                            } else if v.is_i64() {
                                if let Some(v) = v.as_i64() {
                                    number_vec.push(Bson::from(v));
                                }
                            } else if v.is_u64() {
                                if let Some(v) = v.as_u64() {
                                    number_vec.push(Bson::from(v as i64));
                                }
                            }
                        }
                        _ => return Err(anyhow::anyhow!("in op unsupport non int or charts")),
                    }
                }
            } else {
                return Err(anyhow::anyhow!("in op just only support list"));
            }

            if (str_vec.len() > 0 && number_vec.len() > 0)
                || (str_vec.len() > 0 && object_id_vec.len() > 0)
                || (number_vec.len() > 0 && object_id_vec.len() > 0)
            {
                return Err(anyhow::anyhow!(
                    "only supports the same type of int or charts in the list"
                ));
            }

            if object_id_vec.len() > 0 {
                doc.insert(k, doc! {op:object_id_vec});
            } else if str_vec.len() > 0 {
                doc.insert(k, doc! {op:str_vec});
            } else if number_vec.len() > 0 {
                doc.insert(k, doc! {op:number_vec});
            } else if iosdate_vec.len() > 0 {
                doc.insert(k, doc! {op:iosdate_vec});
            }

            return Ok(doc);
        }

        // like
        if op == "$regex" {
            if let condition::Value::Text(s) = v {
                doc.insert(k, doc! {op:format!("{}",s.as_str())});
            } else {
                return Err(anyhow::anyhow!("in op just only support text"));
            }
            return Ok(doc);
        }

        doc = match v {
            condition::Value::Text(v) => {
                if *enable_convert && k.eq("_id") {
                    println!("filter enable_convert: {}, k: {}", enable_convert, k);
                    doc! {k:doc! {op:ObjectId::from_str(v.as_str())?}}
                } else {
                    if v.starts_with("ISODate(") && v.ends_with(")") {
                        // ISODate(2022-06-13 16:00:00) => mongodb: 2022-06-13T16:00:00Z
                        let new_v = v.replace("ISODate(", "").replace(")", "");
                        match convert_to_mongodb_time(&new_v) {
                            Ok(dt) => doc! {k:doc! {op:Bson::from(dt)}},
                            Err(e) => return Err(anyhow::anyhow!("{:?}", e)),
                        }
                    } else {
                        doc! {k:doc! {op:v.as_str().to_string()}}
                    }
                }
            }
            condition::Value::Number(v) => {
                let mut value: Bson = Bson::Null;
                if v.is_f64() {
                    if let Some(v) = v.as_f64() {
                        value = Bson::from(v);
                    }
                } else if v.is_i64() {
                    if let Some(v) = v.as_i64() {
                        value = Bson::from(v);
                    }
                } else if v.is_u64() {
                    if let Some(v) = v.as_u64() {
                        value = Bson::from(v as i64);
                    }
                }
                doc! {k:doc!{op:value}}
            }
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
                    docs.push(Self::gen_doc(&self.2, field.as_str(), value, MongoOp::Eq)?);
                }
                condition::Expr::Ne {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(&self.2, field.as_str(), value, MongoOp::Ne)?);
                }
                condition::Expr::Gt {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(&self.2, field.as_str(), value, MongoOp::Gt)?);
                }
                condition::Expr::Gte {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(&self.2, field.as_str(), value, MongoOp::Gte)?);
                }
                condition::Expr::Lt {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(&self.2, field.as_str(), value, MongoOp::Lt)?);
                }
                condition::Expr::Lte {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(&self.2, field.as_str(), value, MongoOp::Lte)?);
                }
                condition::Expr::Like {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(
                        &self.2,
                        field.as_str(),
                        value,
                        MongoOp::Like,
                    )?);
                }
                condition::Expr::NotLike {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(
                        &self.2,
                        field.as_str(),
                        value,
                        MongoOp::NotLike,
                    )?);
                }
                condition::Expr::In {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(&self.2, field.as_str(), value, MongoOp::In)?);
                }
                condition::Expr::NotIn {
                    span: _,
                    field,
                    value,
                } => {
                    docs.push(Self::gen_doc(
                        &self.2,
                        field.as_str(),
                        value,
                        MongoOp::NotIn,
                    )?);
                }

                _ => return Err(anyhow::anyhow!("not support op")),
            }
        }
        Ok(docs)
    }
}

impl Filter for MongoFilter {
    fn enable_convert(&mut self) -> &mut Self {
        self.2 = true;
        self
    }

    fn parse<S: ToString + ?Sized>(&mut self, input: &S) -> anyhow::Result<Box<Self>> {
        let expr = match parse(input) {
            Ok(s) => s,
            Err(e) => return Err(anyhow::anyhow!("{:?}", e)),
        };
        self.0 = self.eval(&[expr])?.into_iter().flatten().collect();
        self.1 = input.to_string();

        Ok(Box::new(self.clone()))
    }
}

impl GetFilter for MongoFilter {
    fn get_doc(self) -> Document {
        self.0
    }

    fn get_src(self) -> String {
        self.1
    }

    fn get(&self) -> (Document, String) {
        (self.0.clone(), self.1.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_cond() {
        let sym = "a=1 && b=2 || c=1 && b=2";
        let mut mf = MongoFilter(doc! {}, sym.into(), false);
        match mf.parse(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_parse_cond2() {
        let sym = "a=1 && (b=2||c=1) && b=2";
        let mut mf = MongoFilter(doc! {}, sym.to_string(), false);
        match mf.parse(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_parse_cond3() {
        let sym = r#"a=1 && (b="2" || c=1 && b='2')"#;
        let mut mf = MongoFilter(doc! {}, sym.to_string(), false);
        match mf.parse(sym) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_parse_in_notin() {
        let mut mf = MongoFilter(doc! {}, "".to_string(), false);
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
        let mut mf = MongoFilter(doc! {}, "".to_string(), false);
        match mf.parse("a ! '^1.2'") {
            // prefix 1.2
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_parse_f64() {
        let mut mf = MongoFilter(doc! {}, "".to_string(), false);
        match mf.parse("a = 1.2") {
            // prefix 1.2
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_parse_strings() {
        let mut mf = MongoFilter(doc! {}, "".to_string(), false);
        match mf.parse(r#"a = 1.2 || b = 'abc' ||c="cde""#) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_parse_in_strings() {
        let mut mf = MongoFilter(doc! {}, "".to_string(), false);
        match mf.parse(r#"a ~ ("1","2")"#) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };

        let mut mf = MongoFilter(doc! {}, "".to_string(), false);
        match mf.parse(r#"a ~ ('1','2')"#) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn test_parse_isodate() {
        let mut mf = MongoFilter(doc! {}, "".to_string(), false);
        match mf.parse(r#"a >= 'ISODate(2021-01-01 00:00:00)'"#) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        };

        // a in ( ISODate(2021-01-01 00:00:00),ISODate(2021-02-01 00:00:00))
        match mf.parse(r#"a ~ ('ISODate(2021-01-01 00:00:00)','ISODate(2021-02-01 00:00:00)')"#) {
            Ok(c) => println!("{:?}", c),
            Err(e) => panic!("{}", e),
        }
    }
}
