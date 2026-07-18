use eframe::egui;
use serde_json::Value;

/// Renders the Schema panel as a collapsible tree structure.
pub fn render_schema_panel(ui: &mut egui::Ui, state: &crate::state::app_state::AppState) {
    ui.heading("Schema (.avsc)");
    ui.separator();

    // 1. Convert the current Schema into a serde_json::Value for tree traversal.
    let schema_json = serde_json::to_value(&state.schema)
        .unwrap_or_else(|_| serde_json::json!({ "error": "Could not serialize schema" }));

    // 2. Use a ScrollArea to manage the vertical space for complex, nested schemas.
    egui::ScrollArea::vertical().show(ui, |ui| {
        render_schema_tree(ui, "Root", &schema_json);
    });
}

/// Recursively renders a JSON structure as a collapsible tree for egui.
fn render_schema_tree(ui: &mut egui::Ui, label: &str, value: &Value) {
    match value {
        Value::Object(map) => {
            let header = if label == "fields" || label == "items" {
                format!("{}: [{} items]", label, map.len())
            } else {
                label.to_string()
            };

            // Set default_open(true) to ensure every level starts fully expanded
            egui::CollapsingHeader::new(header)
                .default_open(true)
                .show(ui, |ui| {
                    for (k, v) in map {
                        render_schema_tree(ui, k, v);
                    }
                });
        }
        Value::Array(arr) => {
            // Set default_open(true) for arrays as well
            egui::CollapsingHeader::new(format!("{}: [{} items]", label, arr.len()))
                .default_open(true)
                .show(ui, |ui| {
                    for (i, v) in arr.iter().enumerate() {
                        render_schema_tree(ui, &format!("Item {}", i), v);
                    }
                });
        }
        Value::String(s) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                // Use light blue for string values to distinguish them from keys.
                ui.colored_label(egui::Color32::from_rgb(173, 216, 230), format!("\"{}\"", s));
            });
        }
        Value::Number(n) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                // Use orange for numeric values (like size or precision).
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), n.to_string());
            });
        }
        Value::Bool(b) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.colored_label(egui::Color32::YELLOW, b.to_string());
            });
        }
        Value::Null => {
            ui.label(format!("{}: null", label));
        }
    }
}
