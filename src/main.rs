mod data;
mod schema;
mod state;
mod ui;

use crate::schema::parser::collect_named_schemas;
use crate::schema::parser::generate_default_value;
use crate::schema::parser::*;
use crate::state::app_state::AppState;
use crate::ui::schema_explorer;
use eframe::egui;
use std::collections::HashMap;

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
    let mut lookup = HashMap::new();
    collect_named_schemas(&schema, &mut lookup);
    let initial_record = generate_default_value(&schema, &lookup);
    let root_records = vec![initial_record];

    // Build initial registry for schema explorer panel
    let schema_json = serde_json::to_value(&schema).unwrap_or_default();
    let mut schema_registry = HashMap::new();
    schema_explorer::build_type_registry(&schema_json, &mut schema_registry, "");

    let state = AppState {
        schema,
        root_records,
        schema_lookup: lookup,
        schema_json_registry: schema_registry,
        toast_message: None,
        toast_timer: 0.0,
    };
    eframe::run_native(
        "Avro Editor",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(AvroEditorApp { state }))),
    )
}
