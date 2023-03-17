use condition::parse;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use self::value::map_get;

pub mod dict;
pub mod matchs;
pub mod value;
pub mod validate;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Unstructed(Map<String, Value>);

impl Unstructed {
    pub fn set(&mut self, path: &str, value: &Value) {
        dict::set(&mut self.0, &path.to_string(), value);
    }

    pub fn get(&mut self, path: &str) -> Value {
        dict::get(&mut self.0, &path.to_string())
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
}

pub fn from_str(s: &str) -> anyhow::Result<Unstructed> {
    Ok(serde_json::from_str::<Unstructed>(s)?)
}

pub fn empty_unstructed() -> Unstructed {
    from_str("{}").unwrap()
}

#[cfg(test)]
mod tests {

    use super::from_str;

    #[test]
    fn test_basic() {
        let unstructed = match from_str(r#"{"abc":123,"name":"lijim"}"#) {
            Ok(item) => item,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(unstructed.get_by_type::<u64>("abc", 0), 123);

        assert_eq!(unstructed.match_by_predicate("abc=123").unwrap(), true);

        assert_eq!(
            unstructed.match_by_predicate("name='lijimin'").unwrap(),
            false
        );

        assert_eq!(
            unstructed
                .match_by_predicate("name='lijim' && abc=123")
                .unwrap(),
            true
        );
    }
}
