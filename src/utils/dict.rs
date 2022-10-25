use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::Result;

pub fn value_to_map<'a, T: DeserializeOwned + Serialize>(
    value: &'a T,
) -> Result<Map<String, Value>> {
    let mut binding = serde_json::to_value::<&T>(value)?;
    let data = binding
        .as_object_mut()
        .context("obj_value to json data is none")?;

    return Ok(data.clone());
}

pub fn compare_and_merge_value(
    old_map: &mut Map<String, Value>,
    new_map: &mut Map<String, Value>,
    field: &String,
) -> bool {
    let old_value = get(old_map, field);
    let new_value = get(new_map, field);
    if !old_value.eq(&new_value) {
        set(old_map, field, &new_value);
        return true;
    }

    return false;
}

fn shift(path: &String) -> (String, String) {
    let data: Vec<&str> = path.split(".").collect();
    if data.len() < 1 {
        return ("".to_string(), "".to_string());
    }

    if data.len() < 2 {
        return (data[0].to_string(), "".to_string());
    }

    let new_data = &data[1..data.len()].join(".");
    return (data[0].to_string(), new_data.to_string());
}

fn get(data: &mut Map<String, Value>, path: &String) -> Value {
    let (head, remain) = shift(path);
    if let Some(value) = data.get_mut(&head) {
        if remain != "" {
            // 获取的field不存在时返回null
            if let Some(value) = value.as_object_mut() {
                return get(value, &remain);
            }
            return Value::Null;
        };

        return value.clone();
    }
    return Value::Null;
}

fn set(data: &mut Map<String, Value>, path: &String, value: &Value) -> Option<Value> {
    let (head, remain) = shift(path);

    if remain == "" {
        data.remove(path);
        data.insert(path.to_string(), value.clone());
        return Some(().into());
    }

    // 获取的field可能为空，也可能不是map
    if let Some(field_value) = data.get_mut(&head) {
        if let Some(path_value) = field_value.as_object_mut() {
            return set(path_value, &remain, value);
        }
    }

    data.insert(
        head.to_string(),
        serde_json::Value::Object(serde_json::Map::new()),
    );
    return set(data.get_mut(&head)?.as_object_mut()?, &remain, value);
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::{compare_and_merge_value, get, set, value_to_map};

    #[derive(Default, Debug, Serialize, Deserialize)]
    pub struct Root {
        pub name: String,
        pub age: i64,
        pub phones: Vec<String>,
        pub test: Test,
    }

    #[derive(Default, Debug, Serialize, Deserialize)]
    pub struct Test {
        pub aa: String,
        pub cc: Cc,
    }

    #[derive(Default, Debug, Serialize, Deserialize)]
    pub struct Cc {
        pub dd: String,
    }

    const DATA: &str = r#"
    {
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ],
        "test":{
            "aa":"bb",
            "cc":{
                "dd":"cc"
            }
        }
    }"#;

    #[test]
    fn test_get() {
        let p = serde_json::from_str::<Root>(DATA).unwrap();
        let p_map = &mut value_to_map::<Root>(&p).unwrap();

        let value = get(p_map, &"test.aa.aa.cc".to_string());
        assert_eq!(value, serde_json::Value::Null);

        let value = get(p_map, &"test.cc.dd".to_string());

        assert_eq!(value, "cc");
    }

    #[test]
    fn test_set() {
        let p = serde_json::from_str::<Root>(DATA).unwrap();
        let p_map = &mut value_to_map::<Root>(&p).unwrap();
        let value = serde_json::to_value(&"new_test_data".to_string()).unwrap();

        set(p_map, &"test.aa.aa.cc".to_string(), &value).unwrap();
        let value = get(p_map, &"test.aa.aa.cc".to_string());
        assert_eq!(value, "new_test_data");
    }
}
