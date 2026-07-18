pub mod record_editor;

use crate::state::app_state::AppState;
use eframe::egui;

pub fn render_main_ui(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🛠 Schema-Driven Avro Editor");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🚀 Export to Avro File").clicked() {
                    match crate::data::avro_io::export_to_avro(state) {
                        Ok(filename) => {
                            println!("Successfully exported to {}", filename);
                        }
                        Err(e) => {
                            eprintln!("Export failed: {}", e);
                        }
                    }
                }
            });
        });
    });

    egui::SidePanel::right("preview_panel").show(ctx, |ui| {
        render_preview_panel(ui, state);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            crate::ui::record_editor::render_root_list(ui, state);
        });
    });
}

fn render_preview_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Preview (.json)");
    ui.separator();

    // 遍历 root_records，对每一条记录调用 to_json()，最后包裹在 serde_json::Value::Array 中
    let json_array =
        serde_json::Value::Array(state.root_records.iter().map(|rec| rec.to_json()).collect());

    let json_data = serde_json::to_string_pretty(&json_array).unwrap_or_default();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add(
            egui::TextEdit::multiline(&mut json_data.as_str())
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY),
        );
    });
}
