use apache_avro::Schema;
use apache_avro::schema::Name;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub schema_lookup: HashMap<Name, Schema>,
    pub schema_json_registry: HashMap<String, Value>,
}
