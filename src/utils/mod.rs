use condition::parse;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use self::value::map_get;

pub mod dict;
pub mod local;
pub mod matchs;
pub mod retry;
pub mod time;
pub mod value;

pub use local::{ErrorLocation, Location};

#[macro_export]
macro_rules! here {
    () => {
        &Location {
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Unstructed(Map<String, Value>);

impl Unstructed {
    pub fn new() -> Unstructed {
        Unstructed(Map::new())
    }

    pub fn set(&mut self, path: &str, value: &Value) {
        dict::set(&mut self.0, &path.to_string(), value);
    }

    pub fn get(&self, path: &str) -> Value {
        dict::get(&self.0, &path.to_string())
    }

    pub fn remove(&mut self, path: &str) {
        dict::remove(&mut self.0, path)
    }

    pub fn unmarshal<'a, T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_value::<T>(serde_json::Value::Object(
            self.0.clone(),
        ))?)
    }

    pub fn keys(&self) -> Vec<String> {
        self.0.keys().map(|key| key.to_string()).collect()
    }

    pub fn get_by_type<T: DeserializeOwned>(&self, key: &str, default: T) -> T {
        map_get::<T>(&self.0, key, default)
    }

    pub fn match_by_predicate(&self, predicate: &str) -> anyhow::Result<bool> {
        if matchs::matchs(&mut vec![self.clone()], parse(predicate)?)?.len() > 0 {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn cut(&self, keys: Vec<String>) -> Unstructed {
        let mut map = Map::new();
        for key in keys {
            if let Some(value) = self.0.get(&key) {
                map.insert(key, value.clone());
            }
        }
        Unstructed(map)
    }

    pub fn remove_fields<'a>(&'a mut self, fields: &'a [&str]) -> &'a mut Unstructed {
        for field in fields {
            self.remove(field)
        }
        self
    }

    pub fn change_fields<'a>(&'a mut self, fields: &'a [(&str, &str)]) -> &'a mut Unstructed {
        for field in fields {
            self.set(&field.1, &self.get(&field.0));
            self.remove(field.0);
        }
        self
    }

    pub fn copy_field<'a>(&'a mut self, dst: &str, src: &str) -> &'a mut Unstructed {
        self.set(&dst, &self.get(&src));
        self
    }
}

// E: When using this macro, be careful to quote serde_json
// C: 使用这个宏时，请注意引用serde_json
#[macro_export]
macro_rules! unstructed {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(unstructed!(@single $rest)),*]));
    ($($key:expr => $value:expr,)+) => { unstructed!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let mut item = $crate::utils::Unstructed::new();
            $(
                item.set($key, &serde_json::json!($value));
            )*
            item
        }
    };
}

// 参数检查
// item: &mut Unstructed
// rules: [(&str, &str, bool)]
// 例如：规则字段， 0：条件， 1：错误信息， 2：是否取反
// 用例： item : {"a":1,"b":2,"c":3}, rules: [
//    ("a=1", "a 必须等于1", true),
//    ("a!=1","a 必须等于1", false), // 与上面的规则相反
// ]
pub fn validate(item: &Unstructed, rules: &[(&str, &str, bool)]) -> anyhow::Result<(), String> {
    for (rule, resp, and_non) in rules {
        match item.match_by_predicate(rule) {
            Ok(rs) => {
                if (*and_non && rs) || (!and_non && !rs) {
                    continue;
                }
                return Err(resp.to_owned().to_owned());
            }
            Err(e) => return Err(format!("validate failed: {}", e.to_string())),
        }
    }
    Ok(())
}

pub fn validates(items: &[(&Unstructed, &str, &str, bool)]) -> anyhow::Result<(), String> {
    for (item, rule, resp, and_non) in items {
        match item.match_by_predicate(rule) {
            Ok(rs) => {
                if (*and_non && rs) || (!and_non && !rs) {
                    continue;
                }
                return Err(resp.to_owned().to_owned());
            }
            Err(e) => return Err(format!("validate failed: {}", e.to_string())),
        }
    }
    Ok(())
}

pub fn from_map(map: Map<String, Value>) -> Unstructed {
    Unstructed(map)
}

pub fn from_str(s: &str) -> anyhow::Result<Unstructed> {
    Ok(serde_json::from_str::<Unstructed>(s)?)
}

pub fn empty_unstructed() -> Unstructed {
    from_str("{}").unwrap()
}

#[cfg(test)]
mod tests {

    use super::{from_str, validate};

    #[test]
    fn test_basic() {
        let mut item = match from_str(r#"{"abc":123,"name":"lijim"}"#) {
            Ok(item) => item,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(item.get_by_type::<u64>("abc", 0), 123);

        assert_eq!(item.match_by_predicate("abc=123").unwrap(), true);

        assert_eq!(item.match_by_predicate("name='lijimin'").unwrap(), false);

        assert_eq!(
            item.match_by_predicate("name='lijim' && abc=123").unwrap(),
            true
        );

        item.copy_field("cde", "abc");

        assert_eq!(item.get_by_type::<u32>("cde", 0), 123);

        assert_eq!(
            item.change_fields(&[("name", "name2")])
                .get_by_type::<String>("name2", "".into()),
            "lijim".to_string()
        );
    }

    #[test]
    fn test_validate() {
        let mut item = from_str(r#"{"a":1,"b":2}"#).unwrap();

        validate(
            &mut item,
            &[("a=1", "a 必须等于1", true), ("a!=1", "a 必须等于1", false)],
        )
        .unwrap();
    }

    #[test]
    fn test_macro() {
        let item = unstructed! {
            "a" => 1,
            "b" => 2,
            "g" => "abc",
            "z" => vec![1,2,3]
        };

        assert_eq!(item.get_by_type::<u64>("a", 0), 1);
        assert_eq!(item.get_by_type::<String>("g", "".into()), "abc");
        assert_eq!(item.get_by_type::<Vec<i32>>("z", vec![]), vec![1, 2, 3]);
    }
}
