use serde_json::{Map, Value};

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

pub fn get(data: &mut Map<String, Value>, path: &String) -> Option<Value> {
    let (head, remain) = shift(path);
    let value = data.get(&head)?;

    let mut value = value.clone();

    if remain != "" {
        let new_map = value.as_object_mut()?;
        return get(new_map, &remain);
    };

    Some(value)
}

pub fn set(data: &mut Map<String, Value>, path: &String, value: &Value) -> Option<Value> {
    let (head, remain) = shift(path);

    if remain == "" {
        data.remove(&path.clone());
        data.insert(path.to_string(), value.clone());
        return Some(serde_json::Value::Object(data.clone()));
    }

    let path_value = data.get(&head)?;
    let mut binding = path_value.clone();
    let path_value = binding.as_object_mut()?;

    let res = set(path_value, &remain, value)?;
    data.remove(&head);
    data.insert(head, res);

    return Some(serde_json::Value::Object(data.clone()));
}
