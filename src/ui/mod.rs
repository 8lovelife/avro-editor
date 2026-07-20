pub mod record_editor;
pub mod record_preview;
pub mod schema_explorer;

use crate::data::avro_io;
use crate::data::avro_io::generate_filename;
use crate::schema::{self, parser};
use crate::state::app_state::AppState;
use apache_avro::Schema;
use eframe::egui;
use rfd::FileDialog;
use std::collections::HashMap;
use std::fs;

pub fn render_main_ui(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🛠 Schema-Driven Avro Editor");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾 Export Avro file to...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .set_file_name(generate_filename())
                        .add_filter("Avro", &["avro"])
                        .save_file()
                    {
                        match avro_io::export_to_avro_at_path(state, path) {
                            Ok(p) => {
                                state.toast_message = Some(format!("✅ Saved to: {}", p));
                                state.toast_timer = 3.0;
                            }
                            Err(e) => {
                                state.toast_message = Some(format!("❌ Error: {}", e));
                                state.toast_timer = 5.0;
                            }
                        }
                    }
                }

                if ui.button("📂 Load Custom Schema (.avsc)").clicked() {
                    // 1. Pop up a file picker, restricted to .avsc or .json files
                    if let Some(path) = FileDialog::new()
                        .add_filter("Avro Schema", &["avsc", "json"])
                        .pick_file()
                    {
                        // 2. Attempt to read the file content
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                // 3. Attempt to parse into apache_avro::Schema
                                match Schema::parse_str(&content) {
                                    Ok(new_schema) => {
                                        // Successfully parsed! Update the application state
                                        state.schema = new_schema.clone();
                                        let mut lookup = HashMap::new();
                                        parser::collect_named_schemas(&new_schema, &mut lookup);
                                        let initial_record =
                                            parser::generate_default_value(&state.schema, &lookup);
                                        state.root_records = vec![initial_record];

                                        // Notify user of success
                                        let file_name =
                                            path.file_name().unwrap_or_default().to_string_lossy();
                                        state.toast_message =
                                            Some(format!("✅ Schema Loaded: {}", file_name));
                                        state.toast_timer = 3.0;
                                    }
                                    Err(e) => {
                                        // Parsing failed (e.g., malformed JSON or violates Avro specs)
                                        state.toast_message =
                                            Some(format!("❌ Invalid Schema: {}", e));
                                        state.toast_timer = 5.0;
                                    }
                                }
                            }
                            Err(e) => {
                                // File read failed (e.g., permission issues)
                                state.toast_message = Some(format!("Failed to read file: {}", e));
                                state.toast_timer = 5.0;
                            }
                        }
                    }
                }

                if ui.button("📥 Load Avro File (.avro)").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Avro Data", &["avro"])
                        .pick_file()
                    {
                        match avro_io::import_from_avro_at_path(state, path) {
                            Ok(summary) => {
                                state.toast_message = Some(format!("✅ Avro Loaded: {}", summary));
                                state.toast_timer = 3.0;
                            }
                            Err(e) => {
                                state.toast_message =
                                    Some(format!("❌ Failed to load avro: {}", e));
                                state.toast_timer = 5.0;
                            }
                        }
                    }
                }
            });
        });
    });

    egui::SidePanel::left("schema_panel")
        // .min_width(300.0)
        .show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    schema_explorer::render_schema_panel(ui, state);
                });
        });

    egui::SidePanel::right("preview_panel").show(ctx, |ui| {
        record_preview::render_preview_panel(ui, state);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            record_editor::render_root_list(ui, state);
        });
    });

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
