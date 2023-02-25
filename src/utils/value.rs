use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn get<T>(value: Value, default: T) -> T
where
    T: DeserializeOwned,
{
    match serde_json::from_value(value.clone()) {
        Ok(v) => v,
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::get;
    use serde_json::json;

    #[test]
    fn test_string() {
        assert_eq!("abc".to_string(), get::<String>(json!("abc"), "".into()));
    }

    #[test]
    fn test_u64() {
        assert_eq!(123, get::<u64>(json!(123), 0));
    }
}
