use crossgate::object::{decorate, Object};

#[decorate]
struct Gps {
    #[serde(default)]
    gps: Vec<Vec<f64>>,
}
