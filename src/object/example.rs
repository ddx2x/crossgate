use super::*;

#[decorate]
struct Example {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_object() {
        let object = Example::new("1", 321, "example");
        let object2 = object.clone();
        assert_eq!(object.uid(), object2.uid());
    }
}
