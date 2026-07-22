use crate::data::types::EditValue;
use apache_avro::Schema;
use apache_avro::schema::Name;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub enum PendingFileOp {
    SchemaLoaded { filename: String, content: String },
    RawAvroLoaded { filename: String, bytes: Vec<u8> },
    ExportDone { filename: String },
    Failed(String),
}

pub struct AppState {
    pub schema: Schema,
    pub root_records: Vec<EditValue>,
    pub schema_lookup: HashMap<Name, Schema>,
    pub schema_json_registry: HashMap<String, Value>,
    pub pending_op: Arc<Mutex<Option<PendingFileOp>>>,
    pub toast_message: Option<String>,
    pub toast_timer: f64,
}
