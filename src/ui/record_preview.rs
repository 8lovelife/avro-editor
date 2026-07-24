use crate::state::app_state::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Table,
    Tree,
    Json,
}

pub fn render_preview_panel(ui: &mut egui::Ui, state: &AppState) {
    let view_mode_id = ui.id().with("preview_view_mode");
    let mut view_mode = ui
        .data(|d| d.get_temp::<ViewMode>(view_mode_id))
        .unwrap_or(ViewMode::Tree);

    let json_array =
        serde_json::Value::Array(state.root_records.iter().map(|rec| rec.to_json()).collect());

    ui.horizontal(|ui| {
        ui.heading("Preview");

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if ui
                .selectable_label(view_mode == ViewMode::Tree, "Tree")
                .clicked()
            {
                view_mode = ViewMode::Tree;
            }

            if ui
                .selectable_label(view_mode == ViewMode::Json, "JSON")
                .clicked()
            {
                view_mode = ViewMode::Json;
            }

            if ui
                .selectable_label(view_mode == ViewMode::Table, "Table")
                .clicked()
            {
                view_mode = ViewMode::Table;
            }

            ui.data_mut(|d| d.insert_temp(view_mode_id, view_mode));
        });
    });

    ui.separator();

    match view_mode {
        ViewMode::Table => render_table_view(ui, &json_array),
        ViewMode::Tree => render_tree_view(ui, &json_array),
        ViewMode::Json => render_json_text_view(ui, &json_array),
    }
}

// ========== Table view ==========
fn render_table_view(ui: &mut egui::Ui, json_array: &serde_json::Value) {
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(records) = json_array.as_array() {
                if records.is_empty() {
                    ui.label("No records to display.");
                } else {
                    let mut keys = Vec::new();
                    if let Some(first_obj) = records[0].as_object() {
                        keys = first_obj.keys().cloned().collect();
                    }

                    egui::Grid::new("preview_grid")
                        .striped(true)
                        .min_col_width(120.0)
                        .show(ui, |ui| {
                            for key in &keys {
                                ui.label(egui::RichText::new(key).strong());
                            }
                            ui.end_row();

                            for record in records {
                                if let Some(obj) = record.as_object() {
                                    for key in &keys {
                                        if let Some(val) = obj.get(key) {
                                            let display_str = match val {
                                                serde_json::Value::String(s) => s.clone(),
                                                serde_json::Value::Null => "null".to_string(),
                                                _ => val.to_string(),
                                            };
                                            ui.label(display_str);
                                        } else {
                                            ui.label("");
                                        }
                                    }
                                }
                                ui.end_row();
                            }
                        });
                }
            }
        });
}

// ========== Raw text JSON view ==========
fn render_json_text_view(ui: &mut egui::Ui, json_array: &serde_json::Value) {
    let json_data = serde_json::to_string_pretty(json_array).unwrap_or_default();

    ui.horizontal(|ui| {
        if ui.button("📋 Copy JSON (Pretty)").clicked() {
            ui.ctx().copy_text(json_data.clone());
        }

        if ui.button("📋 Copy JSON (Lines)").clicked() {
            let compressed_data = if let Some(arr) = json_array.as_array() {
                arr.iter()
                    .filter_map(|val| serde_json::to_string(val).ok())
                    .collect::<Vec<String>>()
                    .join("\n")
            } else {
                serde_json::to_string(json_array).unwrap_or_default()
            };

            ui.ctx().copy_text(compressed_data);
        }
    });

    ui.add_space(4.0);

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

// ========== Collapsible / expandable tree view ==========
fn render_tree_view(ui: &mut egui::Ui, json_array: &serde_json::Value) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            render_json_node(ui, "Root", json_array);
        });
}

/// Recursively renders a `serde_json::Value` as a collapsible tree.
fn render_json_node(ui: &mut egui::Ui, label: &str, value: &serde_json::Value) {
    ui.push_id(label, |ui| {
        render_json_node_body(ui, label, value);
    });
}

fn render_json_node_body(ui: &mut egui::Ui, label: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            egui::CollapsingHeader::new(format!("{{}} {} ({} fields)", label, map.len()))
                .id_salt(ui.id())
                .default_open(true)
                .show(ui, |ui| {
                    for (key, val) in map {
                        render_json_node(ui, key, val);
                    }
                });
        }
        serde_json::Value::Array(arr) => {
            egui::CollapsingHeader::new(format!("[] {} ({} items)", label, arr.len()))
                .id_salt(ui.id())
                .default_open(true)
                .show(ui, |ui| {
                    for (i, val) in arr.iter().enumerate() {
                        render_json_node(ui, &format!("[{}]", i), val);
                    }
                });
        }
        serde_json::Value::String(s) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.colored_label(egui::Color32::from_rgb(173, 216, 230), format!("\"{}\"", s));
            });
        }
        serde_json::Value::Number(n) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), n.to_string());
            });
        }
        serde_json::Value::Bool(b) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.colored_label(egui::Color32::from_rgb(79, 193, 255), b.to_string());
            });
        }
        serde_json::Value::Null => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.weak("null");
            });
        }
    }
}
