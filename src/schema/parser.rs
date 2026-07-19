use crate::data::types::EditValue;
use apache_avro::Schema;
use apache_avro::types::Value;

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

pub fn from_avro_value(value: &Value, schema: &Schema) -> EditValue {
    match (schema, value) {
        (Schema::Null, Value::Null) => EditValue::Null,
        (Schema::Boolean, Value::Boolean(b)) => EditValue::Boolean(*b),
        (Schema::Int, Value::Int(i)) => EditValue::Int(*i),
        (Schema::Long, Value::Long(l)) => EditValue::Long(*l),
        (Schema::Float, Value::Float(f)) => EditValue::Float(*f),
        (Schema::Double, Value::Double(d)) => EditValue::Double(*d),
        (Schema::String, Value::String(s)) => EditValue::String(s.clone()),
        (Schema::Bytes, Value::Bytes(b)) => EditValue::Bytes(b.clone()),

        (Schema::Fixed(fixed_schema), Value::Fixed(size, b)) => {
            let _ = fixed_schema;
            EditValue::Fixed(*size, b.clone())
        }

        (Schema::Enum(_), Value::Enum(idx, sym)) => EditValue::Enum {
            index: *idx as usize,
            value: sym.clone(),
        },

        (Schema::Union(union_schema), Value::Union(idx, inner)) => {
            let variant_schema = union_schema.variants()[*idx as usize].clone();
            EditValue::Union {
                index: *idx as usize,
                value: Box::new(from_avro_value(inner, &variant_schema)),
                inner_schema: variant_schema,
            }
        }

        (Schema::Array(arr_schema), Value::Array(items)) => {
            let converted = items
                .iter()
                .map(|v| from_avro_value(v, &arr_schema.items))
                .collect();
            EditValue::Array(converted)
        }

        (Schema::Map(map_schema), Value::Map(kvs)) => {
            let converted = kvs
                .iter()
                .map(|(k, v)| (k.clone(), from_avro_value(v, &map_schema.types)))
                .collect();
            EditValue::Map(converted)
        }

        (Schema::Record(record_schema), Value::Record(fields)) => {
            let mut result = Vec::with_capacity(record_schema.fields.len());
            for field_schema in &record_schema.fields {
                let converted = fields
                    .iter()
                    .find(|(name, _)| name == &field_schema.name)
                    .map(|(_, v)| from_avro_value(v, &field_schema.schema))
                    .unwrap_or_else(|| generate_default_value(&field_schema.schema));
                result.push((field_schema.name.clone(), converted));
            }
            EditValue::Record(result)
        }

        (Schema::Uuid, Value::Uuid(u)) => EditValue::Uuid(u.to_string()),
        (Schema::Date, Value::Date(d)) => EditValue::Date(*d),
        (Schema::TimeMillis, Value::TimeMillis(t)) => EditValue::TimeMillis(*t),
        (Schema::TimeMicros, Value::TimeMicros(t)) => EditValue::TimeMicros(*t),
        (Schema::TimestampMillis, Value::TimestampMillis(t)) => EditValue::TimestampMillis(*t),
        (Schema::TimestampMicros, Value::TimestampMicros(t)) => EditValue::TimestampMicros(*t),

        (Schema::Duration, Value::Duration(d)) => {
            let months: u32 = d.months().into();
            let days: u32 = d.days().into();
            let millis: u32 = d.millis().into();
            let mut bytes = [0u8; 12];
            bytes[0..4].copy_from_slice(&months.to_le_bytes());
            bytes[4..8].copy_from_slice(&days.to_le_bytes());
            bytes[8..12].copy_from_slice(&millis.to_le_bytes());
            EditValue::Duration(bytes)
        }

        (Schema::Decimal(_), Value::Decimal(d)) => {
            eprintln!(
                "Warning: Decimal import is a placeholder, verify apache_avro::Decimal API: {:?}",
                d
            );
            EditValue::Decimal(Vec::new())
        }

        (schema, _) => {
            eprintln!(
                "Warning: schema/value mismatch while importing avro file, schema={:?}",
                schema
            );
            generate_default_value(schema)
        }
    }
}
