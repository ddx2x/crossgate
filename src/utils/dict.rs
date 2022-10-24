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

pub fn set(data: &mut Map<String, Value>, path: &String, value: &Value) -> Option<()> {
    let (head, remain) = shift(path);

    let value = data.get(&head)?;



    return Some(());
}
