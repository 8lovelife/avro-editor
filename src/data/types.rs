use apache_avro::Schema;
use apache_avro::types::Value;
use serde_json::{Map, Value as JsonValue, json};

#[derive(Debug, Clone)]
pub enum EditValue {
    Null,
    Boolean(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),

    /// Represents an Avro Bytes type (raw binary data).
    Bytes(Vec<u8>),

    /// Represents an Avro Fixed type (fixed-length binary data).
    Fixed(usize, Vec<u8>),

    /// Represents an Avro Array.
    Array(Vec<EditValue>),

    /// Represents an Avro Record. Contains field names and values.
    Record(Vec<(String, EditValue)>),

    /// Represents an Avro Enum.
    Enum {
        index: usize,
        value: String,
    },

    /// Represents an Avro Union.
    Union {
        index: usize,
        inner_schema: Schema,
        value: Box<EditValue>,
    },

    /// Represents an Avro Map.
    Map(Vec<(String, EditValue)>),

    // ---------- LogicalType Derived Types ----------
    Uuid(String),

    /// Avro `int` + `logicalType: date`. Stores days since 1970-01-01.
    Date(i32),

    /// Avro `int` + `logicalType: time-millis`. Stores milliseconds since midnight.
    TimeMillis(i32),

    /// Avro `long` + `logicalType: time-micros`. Stores microseconds since midnight.
    TimeMicros(i64),

    /// Avro `long` + `logicalType: timestamp-millis`. Stores milliseconds since Unix Epoch.
    TimestampMillis(i64),

    /// Avro `long` + `logicalType: timestamp-micros`. Stores microseconds since Unix Epoch.
    TimestampMicros(i64),

    /// Avro `fixed(12)` + `logicalType: duration`. Fixed 12 bytes,
    /// formatted as months(4 bytes) / days(4 bytes) / milliseconds(4 bytes), little-endian.
    Duration([u8; 12]),

    /// Avro `bytes`/`fixed` + `logicalType: decimal`. Stores the unscaled integer's
    /// big-endian two's complement representation, precision/scale managed by schema.
    Decimal(Vec<u8>),
}

impl EditValue {
    /// Converts the internal EditValue to a serde_json::Value.
    pub fn to_json(&self) -> JsonValue {
        match self {
            Self::Null => JsonValue::Null,
            Self::String(s) => json!(s),
            Self::Int(i) => json!(i),
            Self::Long(l) => json!(l),
            Self::Float(f) => json!(f),
            Self::Double(d) => json!(d),
            Self::Boolean(b) => json!(b),

            // Fixed Enum pattern matching: Extract the 'value' field directly
            Self::Enum { index: _, value } => json!(value),

            // Fixed Union pattern matching: Extract 'value' and recursively call to_json
            Self::Union {
                index: _,
                inner_schema: _,
                value,
            } => value.to_json(),

            // Handle Array type
            Self::Array(arr) => json!(arr.iter().map(|v| v.to_json()).collect::<Vec<_>>()),

            // Handle Record type
            Self::Record(fields) => {
                let mut map = Map::new();
                for (name, val) in fields {
                    map.insert(name.clone(), val.to_json());
                }
                JsonValue::Object(map)
            }

            // Handle Map type (represented as Object in JSON)
            Self::Map(kvs) => {
                let mut map = Map::new();
                for (key, val) in kvs {
                    map.insert(key.clone(), val.to_json());
                }
                JsonValue::Object(map)
            }

            // Handle Bytes and Fixed binary types
            // Ignore size parameter when matching Fixed, take only data 'b'
            Self::Bytes(b) | Self::Fixed(_, b) => json!(b),

            // ---------- LogicalType Derived Types JSON Encoding ----------
            // UUID is underlying Avro string
            Self::Uuid(s) => json!(s),

            // Date / Time-millis are underlying Avro int
            Self::Date(days) => json!(days),
            Self::TimeMillis(ms) => json!(ms),

            // Time-micros / Timestamp-millis / Timestamp-micros are underlying Avro long
            Self::TimeMicros(us) => json!(us),
            Self::TimestampMillis(ms) => json!(ms),
            Self::TimestampMicros(us) => json!(us),

            // Duration is underlying fixed(12), output as raw bytes
            Self::Duration(bytes) => json!(bytes.to_vec()),

            // Decimal is underlying bytes/fixed, output as raw (unscaled integer) bytes
            Self::Decimal(bytes) => json!(bytes),
        }
    }

    /// Converts the internal EditValue into apache_avro::types::Value for final serialization.
    pub fn to_avro_value(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Boolean(b) => Value::Boolean(*b),
            Self::Int(i) => Value::Int(*i),
            Self::Long(l) => Value::Long(*l),
            Self::Float(f) => Value::Float(*f),
            Self::Double(d) => Value::Double(*d),
            Self::String(s) => Value::String(s.clone()),
            Self::Bytes(b) => Value::Bytes(b.clone()),
            Self::Fixed(size, data) => Value::Fixed(*size, data.clone()),
            Self::Enum { index, value } => Value::Enum(*index as u32, value.clone()),
            Self::Union {
                index,
                inner_schema: _,
                value,
            } => Value::Union(*index as u32, Box::new(value.to_avro_value())),

            // Handle Array: Map each element recursively.
            Self::Array(arr) => Value::Array(arr.iter().map(|v| v.to_avro_value()).collect()),

            // Handle Map: Convert Vec<(String, EditValue)> to HashMap<String, Value>.
            Self::Map(kvs) => {
                let map = kvs
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_avro_value()))
                    .collect();
                Value::Map(map)
            }

            // Handle Record: Map fields recursively.
            Self::Record(fields) => {
                let avro_fields = fields
                    .iter()
                    .map(|(name, val)| (name.clone(), val.to_avro_value()))
                    .collect();
                Value::Record(avro_fields)
            }

            // ---------- LogicalType Derived Types ----------

            // Uuid: Need to parse the internal string into uuid::Uuid.
            // If parsing fails, fall back to a standard Value::String to avoid panicking and interrupting serialization.
            Self::Uuid(s) => match apache_avro::Uuid::parse_str(s) {
                Ok(u) => Value::Uuid(u),
                Err(_) => Value::String(s.clone()),
            },

            // Date / Time-millis / Time-micros / Timestamp-millis / Timestamp-micros
            // These map one-to-one to variants in apache_avro::types::Value, so we pass them through directly.
            Self::Date(days) => Value::Date(*days),
            Self::TimeMillis(ms) => Value::TimeMillis(*ms),
            Self::TimeMicros(us) => Value::TimeMicros(*us),
            Self::TimestampMillis(ms) => Value::TimestampMillis(*ms),
            Self::TimestampMicros(us) => Value::TimestampMicros(*us),

            // Duration: Unpack the 12 bytes into months(4), days(4), and millis(4) using little-endian order,
            // and construct an apache_avro::Duration.
            Self::Duration(bytes) => {
                let months = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                let days = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                let millis = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
                Value::Duration(apache_avro::Duration::new(
                    apache_avro::Months::new(months),
                    apache_avro::Days::new(days),
                    apache_avro::Millis::new(millis),
                ))
            }

            // Decimal: Pass the big-endian bytes of the unscaled integer to apache_avro::Decimal.
            Self::Decimal(bytes) => Value::Decimal(apache_avro::Decimal::from(bytes.clone())),
        }
    }
}
