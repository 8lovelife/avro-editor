use crate::state::app_state::AppState;

pub fn render_preview_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Preview (.json)");
    ui.separator();

    let json_array =
        serde_json::Value::Array(state.root_records.iter().map(|rec| rec.to_json()).collect());
    let json_data = serde_json::to_string_pretty(&json_array).unwrap_or_default();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut json_data.as_str())
                    .font(egui::TextStyle::Monospace)
                    .desired_width(ui.available_width()),
            );
        });
}
