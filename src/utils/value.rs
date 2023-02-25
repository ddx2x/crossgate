use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub fn get<T>(value: Value, default: T) -> T
where
    T: DeserializeOwned,
{
    match serde_json::from_value(value.clone()) {
        Ok(v) => v,
        Err(_) => default,
    }
}

pub fn map_get<T: DeserializeOwned>(map: Map<String, Value>, key: &str, default: T) -> T {
    if let Some(value) = map.get(key) {
        return get(value.clone(), default);
    }
    default
}

#[cfg(test)]
mod tests {
    use crate::utils::value::map_get;

    use super::get;
    use serde_json::{json, Map};

    #[test]
    fn test_string() {
        assert_eq!("abc".to_string(), get::<String>(json!("abc"), "".into()));
    }

    #[test]
    fn test_u64() {
        assert_eq!(123, get::<u64>(json!(123), 0));
    }

    #[test]
    fn test_map_get_string() {
        let mut map = Map::new();
        map.insert("name".to_owned(), "abc".into());

        assert_eq!(
            "abc".to_string(),
            map_get::<String>(map, "name", "".to_string())
        );
    }
}
