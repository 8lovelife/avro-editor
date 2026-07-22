pub mod record_editor;
pub mod record_preview;
pub mod schema_explorer;

use crate::data::avro_io;
use crate::data::avro_io::generate_filename;
use crate::data::platform;
use crate::schema::parser;
use crate::state::app_state::{AppState, PendingFileOp};
use apache_avro::Schema;
use eframe::egui;

pub fn render_main_ui(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🛠 Schema-Driven Avro Editor");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾 Export Avro file to...").clicked() {
                    match avro_io::encode_avro_bytes(state) {
                        Ok(data) => {
                            platform::save(
                                generate_filename(),
                                "Avro",
                                &["avro"],
                                data,
                                state.pending_op.clone(),
                                ctx,
                            );
                        }
                        Err(e) => {
                            state.toast_message = Some(format!("❌ Error: {}", e));
                            state.toast_timer = 5.0;
                        }
                    }
                }

                if ui.button("📂 Load Custom Schema (.avsc)").clicked() {
                    platform::pick_and_read(
                        "Avro Schema",
                        &["avsc", "json"],
                        state.pending_op.clone(),
                        |filename, bytes| match String::from_utf8(bytes) {
                            Ok(content) => PendingFileOp::SchemaLoaded { filename, content },
                            Err(e) => PendingFileOp::Failed(format!("Invalid UTF-8: {e}")),
                        },
                        ctx,
                    );
                }

                if ui.button("📥 Load Avro File (.avro)").clicked() {
                    platform::pick_and_read(
                        "Avro Data",
                        &["avro"],
                        state.pending_op.clone(),
                        |filename, bytes| PendingFileOp::RawAvroLoaded { filename, bytes },
                        ctx,
                    );
                }
            });
        });
    });

    egui::SidePanel::left("schema_panel").show(ctx, |ui| {
        schema_explorer::render_schema_panel(ui, state);
    });

    egui::SidePanel::right("preview_panel").show(ctx, |ui| {
        record_preview::render_preview_panel(ui, state);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        record_editor::render_root_list(ui, state);
    });

    let finished_op = state.pending_op.lock().unwrap().take();
    if let Some(op) = finished_op {
        match op {
            PendingFileOp::SchemaLoaded { filename, content } => {
                match Schema::parse_str(&content) {
                    Ok(new_schema) => {
                        let schema_info = parser::build_schema_info(&new_schema);
                        let initial_record =
                            parser::generate_default_value(&new_schema, &schema_info.schema_lookup);
                        state.schema = new_schema;
                        state.root_records = vec![initial_record];
                        state.schema_lookup = schema_info.schema_lookup;
                        state.schema_json_registry = schema_info.schema_json_registry;
                        state.toast_message = Some(format!("✅ Schema Loaded: {}", filename));
                        state.toast_timer = 3.0;
                    }
                    Err(e) => {
                        state.toast_message = Some(format!("❌ Invalid Schema: {}", e));
                        state.toast_timer = 5.0;
                    }
                }
            }
            PendingFileOp::RawAvroLoaded { filename, bytes } => {
                match avro_io::import_from_avro_bytes(state, &bytes) {
                    Ok(count) => {
                        state.toast_message =
                            Some(format!("✅ Avro Loaded: {} ({} records)", filename, count));
                        state.toast_timer = 3.0;
                    }
                    Err(e) => {
                        state.toast_message = Some(format!("❌ Failed to load avro: {}", e));
                        state.toast_timer = 5.0;
                    }
                }
            }
            PendingFileOp::ExportDone { filename } => {
                state.toast_message = Some(format!("✅ Saved: {}", filename));
                state.toast_timer = 3.0;
            }
            PendingFileOp::Failed(e) => {
                state.toast_message = Some(format!("❌ Error: {}", e));
                state.toast_timer = 5.0;
            }
        }
    }

    if let Some(msg) = &state.toast_message {
        egui::Window::new("ToastNotification")
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 30.0))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .frame(egui::Frame::popup(&ctx.style()).inner_margin(10.0))
            .show(ctx, |ui| {
                ui.label(msg);
            });

        state.toast_timer -= ctx.input(|i| i.stable_dt as f64);

        if state.toast_timer <= 0.0 {
            state.toast_message = None;
        } else {
            ctx.request_repaint();
        }
    }
}
