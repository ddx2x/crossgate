use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::Result;

use super::Unstructed;

pub fn compare_and_merge<'a, T>(old: &mut T, new: &mut T, fields: Vec<String>) -> Result<T>
where
    T: DeserializeOwned + Serialize,
{
    let mut new_map = value_to_map::<T>(&new)?;
    let mut old_map = value_to_map::<T>(&old)?;

    let mut updated = false;

    for field in fields {
        if compare_and_merge_value(&mut old_map, &mut new_map, &field) {
            updated = true;
        }
    }

    if !updated {
        return Err(anyhow::anyhow!("not update"));
    }

    Ok(serde_json::from_value::<T>(serde_json::to_value(old_map)?)?)
}

pub fn value_to_map<'a, T: DeserializeOwned + Serialize>(
    value: &'a T,
) -> Result<Map<String, Value>> {
    let mut binding = serde_json::to_value::<&T>(value)?;
    let data = binding
        .as_object_mut()
        .context("obj value to json data is none")?;

    return Ok(data.clone());
}

pub fn from_unstructed_to_type<T: DeserializeOwned + Serialize>(
    unstructed: Unstructed,
) -> Result<T> {
    Ok(serde_json::from_value::<T>(serde_json::Value::Object(
        unstructed.0,
    ))?)
}

pub fn from_value_to_unstructed<'a, T: DeserializeOwned + Serialize>(
    value: &'a T,
) -> Result<Unstructed> {
    let mut binding = serde_json::to_value::<&T>(value)?;
    let data = binding
        .as_object_mut()
        .context("value to unstruncted is none")?;

    return Ok(Unstructed(data.clone()));
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

pub fn get(data: &Map<String, Value>, path: &str) -> Value {
    let (head, remain) = shift(path);

    if !data.contains_key(&head) {
        return Value::Null;
    }

    if remain == "" {
        if let Some(value) = data.get(&head) {
            return value.clone();
        };
    }

    if let Some(value) = data.get(&head) {
        if let Some(data) = value.as_object() {
            return get(data, &remain.to_string());
        }
    }

    Value::Null
}

pub fn remove(data: &mut Map<String, Value>, path: &str) {
    let (head, remain) = shift(path);

    if !data.contains_key(&head) {
        return;
    }

    if remain == "" {
        data.remove(&head);
        return;
    }

    if let Some(value) = data.get_mut(&head) {
        if let Some(data) = value.as_object_mut() {
            remove(data, &remain.to_string())
        }
    }
}

pub fn set(data: &mut Map<String, Value>, path: &str, value: &Value) -> Option<Value> {
    let (head, remain) = shift(path);

    if remain == "" {
        data.remove(path);
        data.insert(path.to_string(), value.clone());
        return Some(().into());
    }

    if let Some(field_value) = data.get_mut(&head) {
        if let Some(path_value) = field_value.as_object_mut() {
            return set(path_value, &remain.to_string(), value);
        }
    }

    data.insert(
        head.to_string(),
        serde_json::Value::Object(serde_json::Map::new()),
    );

    return set(
        data.get_mut(&head)?.as_object_mut()?,
        &remain.to_string(),
        value,
    );
}

fn shift(path: &str) -> (String, String) {
    let list: Vec<&str> = path.split(".").collect();
    match list.len() {
        1 => return (list[0].to_string(), "".to_string()),
        _ => return (list[0].to_string(), list[1..list.len()].join(".")),
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::utils::dict::remove;

    use super::{get, set, value_to_map};

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
        let root = serde_json::from_str::<Root>(DATA).unwrap();
        let map = &mut value_to_map::<Root>(&root).unwrap();

        assert_eq!(
            get(map, &"test.aa.aa.cc".to_string()),
            serde_json::Value::Null
        );

        assert_eq!(get(map, &"test.aa".to_string()), "bb",);

        assert_eq!(get(map, &"test.cc.dd".to_string()), "cc",);
    }

    #[test]
    fn test_set() {
        let root = serde_json::from_str::<Root>(DATA).unwrap();
        let map = &mut value_to_map::<Root>(&root).unwrap();

        set(
            map,
            &"test.aa.aa.cc".to_string(),
            &serde_json::to_value(&"new_test_data".to_string()).unwrap(),
        )
        .unwrap();

        assert_eq!(get(map, &"test.aa.aa.cc".to_string()), "new_test_data");
    }

    #[test]

    fn test_remove() {
        let root = serde_json::from_str::<Root>(DATA).unwrap();
        let map = &mut value_to_map::<Root>(&root).unwrap();

        remove(map, &"age".to_string());

        assert_eq!(get(map, &"age".to_string()), Value::Null);

        remove(map, &"test.aa".to_string());

        assert_eq!(get(map, &"test.aa".to_string()), Value::Null);
    }
}
