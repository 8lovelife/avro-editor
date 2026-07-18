use crate::data::types;
use crate::data::types::EditValue;
use crate::state::app_state::AppState;
use apache_avro::Writer;
use apache_avro::types::Value;
use std::fs;

pub fn to_avro_value(edit_value: &EditValue) -> Value {
    match edit_value {
        EditValue::String(s) => Value::String(s.clone()),
        EditValue::Int(i) => Value::Int(*i),
        EditValue::Long(l) => Value::Long(*l),
        EditValue::Double(d) => Value::Double(*d),
        EditValue::Boolean(b) => Value::Boolean(*b),
        EditValue::Enum(idx, syms) => Value::Enum(*idx as u32, syms[*idx].clone()),
        EditValue::Union(idx, val) => Value::Union(*idx as u32, Box::new(to_avro_value(val))),
        EditValue::Array(arr) => Value::Array(arr.iter().map(to_avro_value).collect()),
        EditValue::Record(fields) => {
            let avro_fields = fields
                .iter()
                .map(|(name, val)| (name.clone(), to_avro_value(val)))
                .collect();
            Value::Record(avro_fields)
        }
        EditValue::Null => Value::Null,
    }
}

pub fn export_to_avro(state: &AppState) -> Result<String, String> {
    // 1. 生成文件名
    let filename = types::generate_filename();

    // 2. 转为 Avro Values
    let avro_values: Vec<_> = state.root_records.iter().map(to_avro_value).collect();

    // 3. 使用 Writer 写入内存缓存
    let mut writer = Writer::new(&state.schema, Vec::new());
    for val in avro_values {
        writer.append(val).map_err(|e| e.to_string())?;
    }

    let result = writer.into_inner().map_err(|e| e.to_string())?;

    // 4. 保存到文件
    fs::write(&filename, result).map_err(|e| e.to_string())?;

    Ok(filename)
}

#[cfg(test)]
mod tests {
    use apache_avro::Reader;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_read_existing_avro_file() {
        // 1. 指定你的文件路径
        let file_path = "/Users/marycheng/Project/avro_editor/20260717_232546_Ap9Jlv.avro";

        // 2. 打开文件
        let file = File::open(file_path).expect("Failed to open avro file");
        let reader = BufReader::new(file);

        // 3. 使用 Reader 创建 Avro 读取器
        let avro_reader = Reader::new(reader).expect("Failed to create Avro reader");

        // 4. 遍历记录并断言
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
