use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub mod dict;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Unstructed(Map<String, Value>);

impl Unstructed {
    pub fn set(&mut self, path: &str, value: &Value) {
        dict::set(&mut self.0, &path.to_string(), value);
    }

    pub fn get(&mut self, path: &str) -> Value {
        dict::get(&mut self.0, &path.to_string())
    }

    pub fn unmarshal<'a, T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_value::<T>(serde_json::Value::Object(
            self.0.clone(),
        ))?)
    }
    
    pub fn keys(&self) -> Vec<String> {
        self.0.keys().map(|key| key.to_string()).collect()
    }
}
