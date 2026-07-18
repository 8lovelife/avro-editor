use crate::data::types::EditValue;
use apache_avro::Schema;

const RAW_SCHEMA: &str = include_str!("deep_nested_all_types.avsc");

pub fn get_schema() -> Schema {
    Schema::parse_str(RAW_SCHEMA).expect("Failed to parse schema from file")
}
/// Generates a default `EditValue` based on the provided Avro Schema.
pub fn generate_default_value(schema: &Schema) -> EditValue {
    match schema {
        // --- 1. Primitive Types ---
        Schema::Null => EditValue::Null,
        Schema::Boolean => EditValue::Boolean(false),
        Schema::Int => EditValue::Int(0),
        Schema::Long => EditValue::Long(0),
        Schema::Float => EditValue::Float(0.0),
        Schema::Double => EditValue::Double(0.0),
        Schema::String => EditValue::String(String::new()),
        Schema::Bytes => EditValue::Bytes(Vec::new()),

        // --- 2. Complex Types ---
        // Handle Fixed: Initialize a byte vector with the exact required size.
        Schema::Fixed(fixed_schema) => {
            EditValue::Fixed(fixed_schema.size, vec![0; fixed_schema.size])
        }

        // Handle Array: Default to an empty vector.
        Schema::Array(_) => EditValue::Array(Vec::new()),

        // Handle Map: Default to an empty list of key-value pairs.
        Schema::Map(_) => EditValue::Map(Vec::new()),

        // Handle Record: Recursively generate default values for all fields.
        Schema::Record(record_schema) => {
            let mut fields = Vec::new();
            for field in &record_schema.fields {
                let default_val = generate_default_value(&field.schema);
                fields.push((field.name.clone(), default_val));
            }
            EditValue::Record(fields)
        }

        // Handle Enum: Default to the first symbol defined in the schema.
        Schema::Enum(enum_schema) => {
            let default_symbol = enum_schema.symbols.first().cloned().unwrap_or_default();
            EditValue::Enum {
                index: 0,
                value: default_symbol,
            }
        }

        // Handle Union: Default to the first branch of the union.
        Schema::Union(union_schema) => {
            let variants = union_schema.variants();
            let first_schema = &variants[0];
            EditValue::Union {
                index: 0,
                inner_schema: first_schema.clone(),
                value: Box::new(generate_default_value(first_schema)),
            }
        }

        // --- 3. Logical Types ---
        Schema::Uuid => EditValue::Uuid(String::new()),
        Schema::Date => EditValue::Date(0),
        Schema::TimeMillis => EditValue::TimeMillis(0),
        Schema::TimeMicros => EditValue::TimeMicros(0),
        Schema::TimestampMillis => EditValue::TimestampMillis(0),
        Schema::TimestampMicros => EditValue::TimestampMicros(0),
        Schema::Duration => EditValue::Duration([0u8; 12]),
        Schema::Decimal(_) => EditValue::Decimal(Vec::new()),

        // --- 4. Fallback ---
        _ => {
            // If there's an unrecognized or highly nested logical type, fall back safely.
            eprintln!("Warning: Unhandled schema type: {:?}", schema);
            EditValue::Null
        }
    }
}
