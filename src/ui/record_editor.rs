use crate::data::types::EditValue;
use crate::schema::parser::generate_default_value;
use crate::state::app_state::AppState;
use apache_avro::Schema;
use apache_avro::schema::Name;
use eframe::egui;
use std::collections::HashMap;

pub fn render_root_list(ui: &mut egui::Ui, state: &mut AppState) {
    let mut scroll_to_new_record = false;

    ui.horizontal(|ui| {
        if ui.button("➕ Add New Record").clicked() {
            let new_record = generate_default_value(&state.schema, &state.schema_lookup);
            state.root_records.push(new_record);
            scroll_to_new_record = true;
        }
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let count = state.root_records.len();
            ui.label(
                egui::RichText::new(format!("Total Records: {}", count))
                    .strong()
                    .color(egui::Color32::GRAY),
            );
        });
    });
    ui.separator();

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut to_remove = None;
            let last_idx = state.root_records.len().saturating_sub(1);

            for (idx, record) in state.root_records.iter_mut().enumerate() {
                let header_response = ui
                    .horizontal(|ui| {
                        ui.label(format!("Record #{}", idx + 1));
                        if ui.button("🗑 Delete").clicked() {
                            to_remove = Some(idx);
                        }
                    })
                    .response;

                if scroll_to_new_record && idx == last_idx {
                    header_response.scroll_to_me(Some(egui::Align::TOP));
                }

                render_editor(
                    ui,
                    &state.schema,
                    record,
                    &format!("idx_{}", idx),
                    &state.schema_lookup,
                );
                ui.add_space(10.0);
                ui.separator();
            }

            if let Some(idx) = to_remove {
                state.root_records.remove(idx);
            }
        });
}

pub fn render_editor(
    ui: &mut egui::Ui,
    schema: &Schema,
    value: &mut EditValue,
    label: &str,
    lookup: &HashMap<Name, Schema>,
) {
    ui.push_id(label, |ui| {
        render_editor_body(ui, schema, value, label, lookup);
    });
}

fn render_editor_body(
    ui: &mut egui::Ui,
    schema: &Schema,
    value: &mut EditValue,
    label: &str,
    lookup: &HashMap<Name, Schema>,
) {
    // Attempt to resolve Schema::Ref directly before rendering
    let effective_schema = match schema {
        Schema::Ref { name } => lookup.get(name).unwrap_or(schema),
        _ => schema,
    };

    match (effective_schema, value) {
        (Schema::String, EditValue::String(s)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.add(egui::TextEdit::singleline(s).desired_width(120.0));
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
        (Schema::Float, EditValue::Float(f)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.add(egui::DragValue::new(f).speed(0.1));
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
        (
            Schema::Enum(_),
            EditValue::Enum {
                index,
                value: sym_val,
            },
        ) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                egui::ComboBox::from_id_salt(ui.id())
                    .selected_text(sym_val.as_str())
                    .show_ui(ui, |ui| {
                        if let Schema::Enum(es) = effective_schema {
                            for (i, sym) in es.symbols.iter().enumerate() {
                                if ui.selectable_value(index, i, sym).clicked() {
                                    *sym_val = sym.clone();
                                }
                            }
                        }
                    });
            });
        }

        (
            Schema::Union(union_schema),
            EditValue::Union {
                index,
                inner_schema,
                value: inner_val,
            },
        ) => {
            ui.horizontal(|ui| {
                ui.label(format!("| {}:", label));
                let mut selected = *index;
                egui::ComboBox::from_id_salt(ui.id())
                    .selected_text(format!("{:?}", union_schema.variants()[selected]))
                    .show_ui(ui, |ui| {
                        for (i, var) in union_schema.variants().iter().enumerate() {
                            ui.selectable_value(&mut selected, i, format!("{:?}", var));
                        }
                    });

                if selected != *index {
                    *index = selected;
                    let new_variant_schema = union_schema.variants()[selected].clone();
                    *inner_val = Box::new(generate_default_value(&new_variant_schema, lookup));
                    *inner_schema = new_variant_schema;
                }
            });
            ui.indent(label, |ui| {
                render_editor(ui, inner_schema, inner_val, "Value", lookup);
            });
        }

        (Schema::Record(rect_schema), EditValue::Record(fields)) => {
            egui::CollapsingHeader::new(format!("Record: {}", label))
                .id_salt(ui.id())
                .default_open(true)
                .show(ui, |ui| {
                    for (field_schema, (f_name, f_val)) in
                        rect_schema.fields.iter().zip(fields.iter_mut())
                    {
                        render_editor(ui, &field_schema.schema, f_val, f_name, lookup);
                    }
                });
        }

        (Schema::Array(arr_schema), EditValue::Array(items)) => {
            egui::CollapsingHeader::new(format!("[ ] Array: {} ({})", label, items.len()))
                .id_salt(ui.id())
                .default_open(true)
                .show(ui, |ui| {
                    let mut to_remove = None;
                    for (idx, item) in items.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button("❌").clicked() {
                                to_remove = Some(idx);
                            }
                            render_editor(
                                ui,
                                &arr_schema.items,
                                item,
                                &format!("[{}]", idx),
                                lookup,
                            );
                        });
                    }
                    if let Some(idx) = to_remove {
                        items.remove(idx);
                    }
                    if ui.button("➕ Add").clicked() {
                        items.push(generate_default_value(&arr_schema.items, lookup));
                    }
                });
        }

        (Schema::Map(map_schema), EditValue::Map(kvs)) => {
            egui::CollapsingHeader::new(format!("{ } Map: {}", "{ }", label))
                .default_open(true)
                .id_salt(ui.id())
                .show(ui, |ui| {
                    let mut to_remove = None;
                    for (i, (key, val)) in kvs.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(key);
                            render_editor(ui, &map_schema.types, val, &format!("[{}]", i), lookup);
                            if ui.button("🗑").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }
                    if let Some(i) = to_remove {
                        kvs.remove(i);
                    }
                    if ui.button("➕ Add Entry").clicked() {
                        kvs.push((
                            "new_key".to_string(),
                            generate_default_value(&map_schema.types, lookup),
                        ));
                    }
                });
        }

        (Schema::Bytes, EditValue::Bytes(b)) => {
            ui.label(format!("{}: [Bytes length: {}]", label, b.len()));
        }

        (Schema::Fixed(fixed_schema), EditValue::Fixed(size, b)) => {
            if *size != fixed_schema.size || b.len() != fixed_schema.size {
                *size = fixed_schema.size;
                b.resize(fixed_schema.size, 0);
            }

            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{} (fixed[{}]):", label, size));
                for byte in b.iter_mut() {
                    ui.add(
                        egui::DragValue::new(byte)
                            .range(0..=255)
                            .hexadecimal(2, false, true),
                    );
                }
            });
        }

        (Schema::Null, EditValue::Null) => {
            ui.label(format!("{}:", label));
            ui.weak("null");
        }

        (Schema::Uuid, EditValue::Uuid(s)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{} (uuid):", label));
                ui.add(
                    egui::TextEdit::singleline(s)
                        .desired_width(220.0)
                        .hint_text("00000000-0000-0000-0000-000000000000"),
                );
            });
        }

        (Schema::Date, EditValue::Date(days)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{} (date, epoch days):", label));
                ui.add(egui::DragValue::new(days));
            });
        }

        (Schema::TimeMillis, EditValue::TimeMillis(ms)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{} (time-millis):", label));
                ui.add(egui::DragValue::new(ms));
            });
        }

        (Schema::TimeMicros, EditValue::TimeMicros(us)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{} (time-micros):", label));
                ui.add(egui::DragValue::new(us));
            });
        }

        (Schema::TimestampMillis, EditValue::TimestampMillis(ms)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{} (timestamp-millis):", label));
                ui.add(egui::DragValue::new(ms));
            });
        }

        (Schema::TimestampMicros, EditValue::TimestampMicros(us)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{} (timestamp-micros):", label));
                ui.add(egui::DragValue::new(us));
            });
        }

        (Schema::Duration, EditValue::Duration(bytes)) => {
            ui.vertical(|ui| {
                ui.label(format!("{} (duration, 12 bytes):", label));
                let months = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                let days = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                let millis = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

                let mut m = months;
                let mut d = days;
                let mut ms = millis;

                ui.horizontal(|ui| {
                    ui.label("months:");
                    if ui.add(egui::DragValue::new(&mut m)).changed() {
                        bytes[0..4].copy_from_slice(&m.to_le_bytes());
                    }
                    ui.label("days:");
                    if ui.add(egui::DragValue::new(&mut d)).changed() {
                        bytes[4..8].copy_from_slice(&d.to_le_bytes());
                    }
                    ui.label("millis:");
                    if ui.add(egui::DragValue::new(&mut ms)).changed() {
                        bytes[8..12].copy_from_slice(&ms.to_le_bytes());
                    }
                });
            });
        }

        (Schema::Decimal(decimal_schema), EditValue::Decimal(bytes)) => {
            ui.label(format!(
                "{} (decimal, precision={}, scale={}): [{} bytes]",
                label,
                decimal_schema.precision,
                decimal_schema.scale,
                bytes.len()
            ));
        }

        _ => {
            ui.label(egui::RichText::new("Type Mismatch").color(egui::Color32::YELLOW));
        }
    }
}
