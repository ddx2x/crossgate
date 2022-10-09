use crossgate::object::{decorate, Object};

#[decorate]
struct Local {
    #[serde(default)]
    name: String,
}
