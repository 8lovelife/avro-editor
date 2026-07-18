use crate::data::types::EditValue;
use crate::schema::parser::generate_default_value;
use crate::state::app_state::AppState;
use apache_avro::Schema;
use eframe::egui;

pub fn render_root_list(ui: &mut egui::Ui, state: &mut AppState) {
    if ui.button("➕ Add New Record").clicked() {
        let new_record = generate_default_value(&state.schema);
        state.root_records.push(new_record);
    }

    ui.separator();

    let mut to_remove = None;

    for (idx, record) in state.root_records.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("Record #{}", idx + 1));
            if ui.button("🗑 Delete").clicked() {
                to_remove = Some(idx);
            }
        });

        render_editor(ui, &state.schema, record, &format!("idx_{}", idx));
        ui.add_space(10.0);
        ui.separator();
    }

    if let Some(idx) = to_remove {
        state.root_records.remove(idx);
    }
}

pub fn render_editor(ui: &mut egui::Ui, schema: &Schema, value: &mut EditValue, label: &str) {
    match (schema, value) {
        (Schema::String, EditValue::String(s)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.add(
                    egui::TextEdit::singleline(s)
                        .desired_width(120.0)
                        .hint_text("Enter value...")
                        .font(egui::TextStyle::Monospace),
                );
            });
        }

        (Schema::Int, EditValue::Int(i)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.add(egui::DragValue::new(i));
            });
        }

        (Schema::Long, EditValue::Long(l)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.add(egui::DragValue::new(l));
            });
        }
        (Schema::Double, EditValue::Double(d)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.add(egui::DragValue::new(d).speed(0.1));
            });
        }

        (Schema::Boolean, EditValue::Boolean(b)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.checkbox(b, "");
            });
        }

        (Schema::Enum(_), EditValue::Enum(current_idx, symbols)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                egui::ComboBox::from_id_salt(label)
                    .selected_text(&symbols[*current_idx])
                    .show_ui(ui, |ui| {
                        for (i, sym) in symbols.iter().enumerate() {
                            ui.selectable_value(current_idx, i, sym);
                        }
                    });
            });
        }

        (Schema::Record(rect_schema), EditValue::Record(fields)) => {
            egui::CollapsingHeader::new(format!("📦 Record: {}", label))
                .default_open(true)
                .show(ui, |ui| {
                    for (field_schema, (f_name, f_val)) in
                        rect_schema.fields.iter().zip(fields.iter_mut())
                    {
                        render_editor(ui, &field_schema.schema, f_val, f_name);
                    }
                });
        }

        (Schema::Array(arr_schema), EditValue::Array(items)) => {
            egui::CollapsingHeader::new(format!("📑 Array: {} ({})", label, items.len()))
                .default_open(true)
                .show(ui, |ui| {
                    let mut to_remove = None;

                    for (idx, item) in items.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button("❌").clicked() {
                                to_remove = Some(idx);
                            }
                            ui.vertical(|ui| {
                                render_editor(ui, &arr_schema.items, item, &format!("[{}]", idx));
                            });
                        });
                        ui.separator();
                    }

                    if let Some(idx) = to_remove {
                        items.remove(idx);
                    }

                    if ui.button("➕ Add Element / Record").clicked() {
                        let new_item = generate_default_value(&arr_schema.items);
                        items.push(new_item);
                    }
                });
        }

        (Schema::Union(union_schema), EditValue::Union(current_idx, inner_val)) => {
            ui.horizontal(|ui| {
                ui.label(format!("⌥ {}:", label));

                let variants = union_schema.variants();
                let mut selected = *current_idx;

                egui::ComboBox::from_id_salt(label)
                    .selected_text(format!("{:?}", variants[selected]))
                    .show_ui(ui, |ui| {
                        for (i, var) in variants.iter().enumerate() {
                            ui.selectable_value(&mut selected, i, format!("{:?}", var));
                        }
                    });

                if selected != *current_idx {
                    *current_idx = selected;
                    *inner_val = Box::new(generate_default_value(&variants[selected]));
                }
            });

            ui.indent(label, |ui| {
                render_editor(
                    ui,
                    &union_schema.variants()[*current_idx],
                    inner_val,
                    "Value",
                );
            });
        }

        (Schema::Null, EditValue::Null) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.weak("null");
            });
        }
        _ => {
            ui.label(
                egui::RichText::new("⚠️ State desync with Schema").color(egui::Color32::YELLOW),
            );
        }
    }
}
