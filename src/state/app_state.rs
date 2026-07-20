use crate::data::types::EditValue;
use apache_avro::Schema;
use apache_avro::schema::Name;
use serde_json::Value;
use std::collections::HashMap;

pub struct AppState {
    pub schema: Schema,
    pub root_records: Vec<EditValue>,
    pub schema_lookup: HashMap<Name, Schema>,
    pub schema_json_registry: HashMap<String, Value>,
    pub toast_message: Option<String>,
    pub toast_timer: f64,
}
