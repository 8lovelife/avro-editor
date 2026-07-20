use crate::schema::parser;
use crate::state::app_state::AppState;
use crate::ui::schema_explorer;
use apache_avro::Reader;
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub fn import_from_avro_at_path(state: &mut AppState, path: PathBuf) -> Result<String, String> {
    let file = File::open(&path).map_err(|e| format!("Failed to open avro file: {}", e))?;
    let reader = Reader::new(BufReader::new(file))
        .map_err(|e| format!("Failed to read avro file: {}", e))?;

    let schema = reader.writer_schema().clone();

    // Build the schema lookup map to resolve recursive/reference structures
    let mut lookup = HashMap::new();
    parser::collect_named_schemas(&schema, &mut lookup);

    let mut records = Vec::new();
    for value_result in reader {
        let value = value_result.map_err(|e| format!("Failed to read avro records: {}", e))?;
        records.push(parser::from_avro_value(&value, &schema, &lookup));
    }

    // Convert schema to json value and build the registry for UI rendering
    let schema_json = serde_json::to_value(&schema).unwrap_or_default();
    let mut registry = HashMap::new();
    schema_explorer::build_type_registry(&schema_json, &mut registry, "");

    let count = records.len();
    state.schema = schema;
    state.root_records = records;
    state.schema_lookup = lookup;
    state.schema_json_registry = registry;

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(format!("{} ({} records)", file_name, count))
}

pub fn export_to_avro_at_path(state: &AppState, path: PathBuf) -> Result<String, String> {
    let avro_values: Vec<_> = state
        .root_records
        .iter()
        .map(|record| record.to_avro_value())
        .collect();
    let mut writer = apache_avro::Writer::new(&state.schema, Vec::new());
    for val in avro_values {
        writer.append(val).map_err(|e| e.to_string())?;
    }
    let result = writer.into_inner().map_err(|e| e.to_string())?;
    std::fs::write(&path, result).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

pub fn generate_filename() -> String {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let rand_string = Alphanumeric.sample_string(&mut rand::rng(), 6);
    format!("{}_{}.avro", timestamp, rand_string)
}

#[cfg(test)]
mod tests {
    use apache_avro::Reader;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_extract_avro_file_schema() {
        let test_avro = "test_random.avro";

        // Open the Avro file and wrap it in a BufReader for performance
        let file = File::open(test_avro).expect("Failed to open file");
        let reader = BufReader::new(file);

        // Create the Avro reader. This parses the file header and extracts the schema.
        let avro_reader = Reader::new(reader).expect("Failed to create Reader");

        // Extract the embedded schema from the file
        let schema = avro_reader.writer_schema();

        // Serialize the Schema object into a formatted JSON string (AVSC format)
        // Note: You must have `serde_json` in your Cargo.toml dependencies
        let avsc_string =
            serde_json::to_string_pretty(schema).expect("Failed to serialize schema to JSON");

        println!("Successfully extracted Schema:\n{}", avsc_string);

        // Assert that the schema is valid and contains typical JSON characters
        assert!(
            avsc_string.contains("{"),
            "The extracted schema should be a valid JSON object"
        );

        // Optional: Write the extracted schema directly to an .avsc file
        std::fs::write("extracted_schema.avsc", &avsc_string).expect("Failed to write avsc file");
    }

    #[test]
    fn test_read_existing_avro_file() {
        let file_path = "test_random.avro";
        let file = File::open(file_path).expect("Failed to open avro file");
        let reader = BufReader::new(file);
        let avro_reader = Reader::new(reader).expect("Failed to create Avro reader");
        let mut count = 0;
        for record in avro_reader {
            match record {
                Ok(val) => {
                    println!("Record {}: {:?}", count, val);
                    count += 1;
                }
                Err(e) => {
                    panic!("Record {} is invalid: {:?}", count, e);
                }
            }
        }

        println!("Successfully read {} records from {}", count, file_path);
        assert!(count > 0, "No records found in file");
    }
}
