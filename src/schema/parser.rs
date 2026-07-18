use crate::data::types::EditValue;
use apache_avro::Schema;

const RAW_SCHEMA: &str = include_str!("trade_event.avsc");

pub fn get_schema() -> Schema {
    Schema::parse_str(RAW_SCHEMA).expect("Failed to parse schema from file")
}
pub fn generate_default_value(schema: &Schema) -> EditValue {
    match schema {
        Schema::String => EditValue::String(String::new()),
        Schema::Int => EditValue::Int(0),
        Schema::Long => EditValue::Long(0),
        Schema::Double => EditValue::Double(0.0),
        Schema::Boolean => EditValue::Boolean(false),
        Schema::Enum(e) => EditValue::Enum(0, e.symbols.clone()),
        Schema::Union(u) => EditValue::Union(0, Box::new(generate_default_value(&u.variants()[0]))),
        Schema::Array(_) => EditValue::Array(Vec::new()),
        Schema::Record(r) => {
            let fields = r
                .fields
                .iter()
                .map(|f| (f.name.clone(), generate_default_value(&f.schema)))
                .collect();
            EditValue::Record(fields)
        }
        _ => EditValue::Null,
    }
}
