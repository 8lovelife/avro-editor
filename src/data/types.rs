use rand::distr::{Alphanumeric, SampleString};
use serde_json::{Map, Value as JsonValue, json};

#[derive(Debug, Clone)]
pub enum EditValue {
    String(String),
    Int(i32),
    Long(i64),
    Double(f64),
    Boolean(bool),
    Enum(usize, Vec<String>),
    Union(usize, Box<EditValue>),
    Array(Vec<EditValue>),
    Record(Vec<(String, EditValue)>),
    Null,
}

impl EditValue {
    pub fn to_json(&self) -> JsonValue {
        match self {
            Self::String(s) => json!(s),
            Self::Int(i) => json!(i),
            Self::Long(l) => json!(l),
            Self::Double(d) => json!(d),
            Self::Boolean(b) => json!(b),
            Self::Enum(idx, syms) => json!(syms[*idx]),
            Self::Union(_, val) => val.to_json(),
            Self::Array(arr) => json!(arr.iter().map(|v| v.to_json()).collect::<Vec<_>>()),
            Self::Record(fields) => {
                let mut map = Map::new();
                for (name, val) in fields {
                    map.insert(name.clone(), val.to_json());
                }
                JsonValue::Object(map)
            }
            Self::Null => JsonValue::Null,
        }
    }
}

pub fn generate_filename() -> String {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let rand_string = Alphanumeric.sample_string(&mut rand::rng(), 6);
    format!("{}_{}.avro", timestamp, rand_string)
}
