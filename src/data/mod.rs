pub mod avro_io;
pub mod platform;
pub mod types;

#[cfg(test)]
mod tests {
    use apache_avro::Writer;
    use apache_avro::types::Value;
    use apache_avro::{Days, Millis, Months};
    use apache_avro::{Decimal, Duration, Schema};
    use rand::{Rng, RngExt};
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use uuid::Uuid;

    #[test]
    fn test_generate_and_save_avro_records() {
        let schema = load_schema_from_file("src/schema/sample_schema.avsc");
        let mut writer = Writer::new(&schema, Vec::new());
        let mut rng = rand::rng();

        for _ in 0..10 {
            let record = generate_random_organization(&mut rng);
            writer.append(record).expect("Failed to append record");
        }

        let encoded = writer.into_inner().expect("Failed to flush writer");
        let mut file = File::create("123output.avro").expect("Failed to create avro file");

        file.write_all(&encoded).expect("Failed to write to file");

        println!("Successfully generated and saved 10 fully nested records to output.avro");
    }

    pub fn load_schema_from_file(file_path: &str) -> Schema {
        // 1. Read file content into a string
        let schema_str = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read schema file: {}", file_path));

        // 2. Parse the JSON string into an Avro Schema
        Schema::parse_str(&schema_str).expect("Failed to parse schema JSON content")
    }

    pub fn generate_random_address(rng: &mut impl Rng) -> Value {
        Value::Record(vec![
            (
                "street".into(),
                Value::String(format!("{} Main St", rng.random_range(1..9999))),
            ),
            ("city".into(), Value::String("Random City".into())),
            (
                "zipcode".into(),
                Value::String(format!("{:05}", rng.random_range(10000..99999))),
            ),
            (
                "geo".into(),
                Value::Record(vec![
                    ("lat".into(), Value::Double(rng.random_range(-90.0..90.0))),
                    ("lon".into(), Value::Double(rng.random_range(-180.0..180.0))),
                    (
                        "timezone".into(),
                        Value::Union(1, Box::new(Value::String("UTC".into()))),
                    ),
                ]),
            ),
            ("previousAddresses".into(), Value::Array(vec![])),
        ])
    }

    pub fn generate_random_employee(rng: &mut impl Rng) -> Value {
        let mut metadata_map = HashMap::new();
        metadata_map.insert("level".into(), Value::String("senior".into()));

        Value::Record(vec![
            (
                "name".into(),
                Value::String(format!("Employee {}", rng.random_range(1..1000))),
            ),
            ("age".into(), Value::Int(rng.random_range(1..9999))),
            (
                "salary".into(),
                Value::Float(rng.random_range(30000.0..150000.0)),
            ),
            (
                "hireDate".into(),
                Value::Date(rng.random_range(15000..19000)),
            ),
            (
                "dailyStartTime".into(),
                Value::TimeMillis(rng.random_range(28800000..32400000)),
            ),
            (
                "lastLogin".into(),
                Value::Union(
                    1,
                    Box::new(Value::TimestampMillis(
                        1600000000000 + rng.random_range(0..10000000),
                    )),
                ),
            ),
            (
                "performanceRating".into(),
                Value::Union(1, Box::new(Value::Double(rng.random_range(1.0..5.0)))),
            ),
            ("employeeType".into(), Value::Enum(0, "FULL_TIME".into())),
            (
                "badgeId".into(),
                Value::Fixed(8, (0..8).map(|_| rng.random::<u8>()).collect()),
            ),
            (
                "skills".into(),
                Value::Array(vec![
                    Value::String("Rust".into()),
                    Value::String("Avro".into()),
                ]),
            ),
            ("metadata".into(), Value::Map(metadata_map)),
            ("address".into(), generate_random_address(rng)),
        ])
    }

    pub fn generate_random_project(rng: &mut impl Rng) -> Value {
        Value::Record(vec![
            (
                "projectName".into(),
                Value::String(format!("Project {}", rng.random_range(1..100))),
            ),
            (
                "deadline".into(),
                Value::Union(
                    1,
                    Box::new(Value::TimestampMillis(
                        1700000000000 + rng.random_range(0..10000000),
                    )),
                ),
            ),
            (
                "tags".into(),
                Value::Array(vec![Value::String("urgent".into())]),
            ),
            (
                "isConfidential".into(),
                Value::Boolean(rng.random_bool(0.5)),
            ),
            (
                "budgetDecimal".into(),
                Value::Decimal(Decimal::from(vec![0x01, 0x02, 0x03])),
            ),
            ("priority".into(), Value::Enum(2, "HIGH".into())),
        ])
    }

    pub fn generate_random_team(rng: &mut impl Rng) -> Value {
        let mut projects_map = HashMap::new();
        projects_map.insert("alpha".into(), generate_random_project(rng));

        Value::Record(vec![
            (
                "teamName".into(),
                Value::String(format!("Team {}", rng.random_range(1..50))),
            ),
            (
                "teamUuid".into(),
                Value::Fixed(16, (0..16).map(|_| rng.random::<u8>()).collect()),
            ),
            (
                "members".into(),
                Value::Array(vec![generate_random_employee(rng)]),
            ),
            (
                "teamLead".into(),
                Value::Union(1, Box::new(generate_random_employee(rng))),
            ),
            ("projects".into(), Value::Map(projects_map)),
        ])
    }

    pub fn generate_random_department(rng: &mut impl Rng) -> Value {
        Value::Record(vec![
            (
                "name".into(),
                Value::String(format!("Dept {}", rng.random_range(1..20))),
            ),
            (
                "budget".into(),
                Value::Double(rng.random_range(100000.0..5000000.0)),
            ),
            (
                "departmentCode".into(),
                Value::Fixed(4, (0..4).map(|_| rng.random::<u8>()).collect()),
            ),
            (
                "establishedDate".into(),
                Value::Date(rng.random_range(10000..18000)),
            ),
            ("manager".into(), generate_random_employee(rng)),
            (
                "teams".into(),
                Value::Array(vec![generate_random_team(rng)]),
            ),
        ])
    }

    pub fn generate_random_organization(rng: &mut impl Rng) -> Value {
        let mut org_metadata = HashMap::new();
        org_metadata.insert("region".into(), Value::String("North America".into()));

        Value::Record(vec![
            ("orgId".into(), Value::Uuid(Uuid::new_v4())),
            ("isActive".into(), Value::Boolean(rng.random_bool(0.8))),
            (
                "employeeCount".into(),
                Value::Int(rng.random_range(50..5000)),
            ),
            (
                "totalRevenue".into(),
                Value::Long(rng.random_range(1000000..100000000)),
            ),
            (
                "growthRate".into(),
                Value::Float(rng.random_range(0.01..0.5)),
            ),
            (
                "marketCap".into(),
                Value::Double(rng.random_range(5000000.0..500000000.0)),
            ),
            ("logo".into(), Value::Bytes(vec![0x89, 0x50, 0x4E, 0x47])),
            ("nullField".into(), Value::Null),
            (
                "orgUuid".into(),
                Value::Fixed(16, (0..16).map(|_| rng.random::<u8>()).collect()),
            ),
            (
                "durationField".into(),
                Value::Duration(Duration::new(Months::new(0), Days::new(0), Millis::new(0))),
            ),
            (
                "foundedTimestamp".into(),
                Value::TimestampMillis(1600000000000),
            ),
            ("status".into(), Value::Enum(0, "ACTIVE".into())),
            (
                "departments".into(),
                Value::Array(vec![generate_random_department(rng)]),
            ),
            (
                "contactInfo".into(),
                Value::Union(
                    1,
                    Box::new(Value::Record(vec![
                        ("email".into(), Value::String("contact@org.com".into())),
                        (
                            "phone".into(),
                            Value::Union(1, Box::new(Value::String("555-0199".into()))),
                        ),
                        ("fax".into(), Value::Union(0, Box::new(Value::Null))),
                    ])),
                ),
            ),
            ("orgMetadata".into(), Value::Map(org_metadata)),
            (
                "alternateIds".into(),
                Value::Array(vec![Value::Long(rng.random()), Value::Long(rng.random())]),
            ),
            (
                "mixedUnion".into(),
                Value::Union(7, Box::new(Value::String("mixed_data".into()))),
            ),
        ])
    }
}
