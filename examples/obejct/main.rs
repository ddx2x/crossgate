use crossgate::object::{decorate, Object};

#[decorate(id)]
struct A {}

fn main() {
    let a = A {
        uid: "Abc".to_string(),
        version: 0,
        kind: get_kind(),
    };

    let aa = serde_json::to_string(&a).unwrap();

    println!("{}", aa);

    let s = r#"{ "id": "Abc", "version": 0, "kind": "A" }"#;

    let aaa: A = serde_json::from_str(s).unwrap();

    println!("{:?}", aaa);
}
