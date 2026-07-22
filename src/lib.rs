mod data;
mod schema;
mod state;
mod ui;

use eframe::egui;
use schema::parser;
use state::app_state::AppState;
use std::sync::{Arc, Mutex};

pub struct AvroEditorApp {
    state: AppState,
}

impl AvroEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let schema = parser::get_schema();
        let schema_info = parser::build_schema_info(&schema);
        let initial_record = parser::generate_default_value(&schema, &schema_info.schema_lookup);
        let root_records = vec![initial_record];

        let state = AppState {
            schema,
            root_records,
            schema_lookup: schema_info.schema_lookup,
            schema_json_registry: schema_info.schema_json_registry,
            pending_op: Arc::new(Mutex::new(None)),
            toast_message: None,
            toast_timer: 0.0,
        };

        Self { state }
    }
}

impl eframe::App for AvroEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render_main_ui(ctx, &mut self.state);
    }
}
