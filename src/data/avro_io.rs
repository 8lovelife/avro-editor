use crate::state::app_state::AppState;
use rand::distr::{Alphanumeric, SampleString};
use std::path::PathBuf;

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
    fn test_read_existing_avro_file() {
        let file_path = "xxx.avro";
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
