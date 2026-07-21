use eframe::egui;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SchemaViewMode {
    Tree,
    Text,
}

pub fn render_schema_panel(ui: &mut egui::Ui, state: &crate::state::app_state::AppState) {
    let view_mode_id = ui.id().with("schema_view_mode");
    let mut view_mode = ui
        .data(|d| d.get_temp::<SchemaViewMode>(view_mode_id))
        .unwrap_or(SchemaViewMode::Tree);

    let schema_json = serde_json::to_value(&state.schema)
        .unwrap_or_else(|_| serde_json::json!({ "error": "Could not serialize schema" }));

    ui.horizontal(|ui| {
        ui.heading("Schema (.avsc)");

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if ui
                .selectable_label(view_mode == SchemaViewMode::Tree, "Tree")
                .clicked()
            {
                view_mode = SchemaViewMode::Tree;
            }
            if ui
                .selectable_label(view_mode == SchemaViewMode::Text, "Text")
                .clicked()
            {
                view_mode = SchemaViewMode::Text;
            }
            ui.data_mut(|d| d.insert_temp(view_mode_id, view_mode));

            ui.separator();

            if ui.button("📋 Copy").clicked() {
                let text = serde_json::to_string_pretty(&schema_json).unwrap_or_default();
                ui.ctx().copy_text(text);
            }
        });
    });
    ui.separator();

    match view_mode {
        SchemaViewMode::Tree => {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_schema_tree(ui, "Root", &schema_json, &state.schema_json_registry, "");
                });
        }
        SchemaViewMode::Text => {
            let text = serde_json::to_string_pretty(&schema_json).unwrap_or_default();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(ui.available_width()),
                    );
                });
        }
    }
}

/// Recursively renders the schema tree.
/// Named types are rendered as entry points to avoid infinite recursion.
fn render_schema_tree(
    ui: &mut egui::Ui,
    label: &str,
    value: &Value,
    registry: &HashMap<String, Value>,
    current_namespace: &str,
) {
    match value {
        Value::Object(map) => {
            let next_namespace = map
                .get("namespace")
                .and_then(|v| v.as_str())
                .unwrap_or(current_namespace);

            // Identify named type definitions (Record, Enum, Fixed).
            // We only trigger this logic if this is not a Tooltip definition view.
            if let (Some(Value::String(name)), Some(Value::String(type_str))) =
                (map.get("name"), map.get("type"))
            {
                if (type_str == "record" || type_str == "enum" || type_str == "fixed")
                    && label != "Definition"
                {
                    let fqn = if next_namespace.is_empty() || name.contains('.') {
                        name.clone()
                    } else {
                        format!("{}.{}", next_namespace, name)
                    };

                    let header = format!("{}: {} ({})", label, name, type_str);
                    let header_resp =
                        egui::CollapsingHeader::new(header)
                            .default_open(true)
                            .show(ui, |ui| {
                                for (k, v) in map {
                                    render_schema_tree(ui, k, v, registry, next_namespace);
                                }
                            });

                    // Jump-to-definition logic:
                    if let Some(target) =
                        ui.data(|d| d.get_temp::<String>(egui::Id::new("scroll_target")))
                    {
                        if target == fqn {
                            header_resp
                                .header_response
                                .scroll_to_me(Some(egui::Align::Center));
                            ui.data_mut(|d| {
                                d.remove_temp::<String>(egui::Id::new("scroll_target"))
                            });
                        }
                    }
                    return;
                }
            }

            // Normal rendering for non-named objects
            let header = if label == "fields" || label == "items" {
                format!("{}: [{} items]", label, map.len())
            } else {
                label.to_string()
            };

            egui::CollapsingHeader::new(header)
                .default_open(true)
                .show(ui, |ui| {
                    for (k, v) in map {
                        render_schema_tree(ui, k, v, registry, next_namespace);
                    }
                });
        }
        Value::Array(arr) => {
            egui::CollapsingHeader::new(format!("{}: [{} items]", label, arr.len()))
                .default_open(true)
                .show(ui, |ui| {
                    for (i, v) in arr.iter().enumerate() {
                        render_schema_tree(
                            ui,
                            &format!("Item {}", i),
                            v,
                            registry,
                            current_namespace,
                        );
                    }
                });
        }
        Value::String(s) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                if let Some(resolved_type) = registry.get(s) {
                    let response =
                        ui.colored_label(egui::Color32::LIGHT_GREEN, format!("\"{}\" ℹ", s));
                    response.on_hover_ui(|ui| {
                        ui.heading(format!("Definition of {}", s));
                        ui.separator();

                        egui::ScrollArea::vertical()
                            // .max_height(400.0) // Set a reasonable maximum height
                            .show(ui, |ui| {
                                render_schema_tree(ui, "Definition", resolved_type, registry, "");
                            });
                    });
                    if ui
                        .button("🔗")
                        .on_hover_text("Jump to definition")
                        .clicked()
                    {
                        ui.data_mut(|d| d.insert_temp(egui::Id::new("scroll_target"), s.clone()));
                    }
                } else {
                    ui.colored_label(egui::Color32::from_rgb(173, 216, 230), format!("\"{}\"", s));
                }
            });
        }
        Value::Number(n) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
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
