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
    if let Some(old_value) = get(old_map, field) {
        if let Some(new_value) = get(new_map, field) {
            if !old_value.eq(&new_value) {
                set(old_map, field, &new_value);
                return true;
            }
        }
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

fn get(data: &mut Map<String, Value>, path: &String) -> Option<Value> {
    let (head, remain) = shift(path);
    let value = data.get(&head)?;

    let mut value = value.clone();

    if remain != "" {
        let new_map = value.as_object_mut()?;
        return get(new_map, &remain);
    };

    Some(value)
}

fn set(data: &mut Map<String, Value>, path: &String, value: &Value) -> Option<Value> {
    let (head, remain) = shift(path);

    if remain == "" {
        data.remove(&path.clone());
        data.insert(path.to_string(), value.clone());
        return Some(serde_json::Value::Object(data.clone()));
    }

    let mut path_value = data.get(&head)?.clone();
    let path_value = path_value.as_object_mut()?;

    let res = set(path_value, &remain, value)?;
    data.remove(&head);
    data.insert(head, res);

    return Some(().into());
}
