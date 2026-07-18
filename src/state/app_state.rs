use crate::data::types::EditValue;
use apache_avro::Schema;

pub struct AppState {
    pub schema: Schema,
    pub root_records: Vec<EditValue>,
    pub toast_message: Option<String>,
    pub toast_timer: f64,
}
