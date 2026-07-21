mod data;
mod schema;
mod state;
mod ui;

use crate::schema::parser;
use crate::schema::parser::generate_default_value;
use crate::schema::parser::*;
use crate::state::app_state::AppState;
use eframe::egui;

struct AvroEditorApp {
    state: AppState,
}

impl eframe::App for AvroEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render_main_ui(ctx, &mut self.state);
    }
}

fn main() -> eframe::Result<()> {
    let schema = get_schema();
    let schema_info = parser::build_schema_info(&schema);
    let initial_record = generate_default_value(&schema, &schema_info.schema_lookup);
    let root_records = vec![initial_record];

    let state = AppState {
        schema,
        root_records,
        schema_lookup: schema_info.schema_lookup,
        schema_json_registry: schema_info.schema_json_registry,
        toast_message: None,
        toast_timer: 0.0,
    };
    eframe::run_native(
        "Avro Editor",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(AvroEditorApp { state }))),
    )
}
