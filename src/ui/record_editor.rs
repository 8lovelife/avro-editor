use crate::data::types::EditValue;
use crate::schema::parser::generate_default_value;
use crate::state::app_state::AppState;
use apache_avro::Schema;
use eframe::egui;

pub fn render_root_list(ui: &mut egui::Ui, state: &mut AppState) {
    // 增加记录按钮
    if ui.button("➕ Add New Record").clicked() {
        // 根据根 Schema 生成一个新的实例
        let new_record = generate_default_value(&state.schema);
        state.root_records.push(new_record);
    }

    ui.separator();

    let mut to_remove = None;

    // 遍历所有记录
    for (idx, record) in state.root_records.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("Record #{}", idx + 1));
            if ui.button("🗑 Delete").clicked() {
                to_remove = Some(idx);
            }
        });

        // 渲染单个 Record 内容
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
        // 1. 基础 String 渲染
        (Schema::String, EditValue::String(s)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                // 使用 add 将 TextEdit 组件注入到 UI 中
                ui.add(
                    egui::TextEdit::singleline(s)
                        .desired_width(120.0) // 更改宽度为固定 120px
                        .hint_text("Enter value...") // 添加提示信息
                        .font(egui::TextStyle::Monospace), // 使用等宽字体
                );
            });
        }

        // 2. 基础 Int 渲染
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
                ui.add(egui::DragValue::new(d).speed(0.1)); // 浮点数调整步长
            });
        }

        // 渲染 Boolean (开关)
        (Schema::Boolean, EditValue::Boolean(b)) => {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", label));
                ui.checkbox(b, "");
            });
        }

        // 渲染 Enum (下拉选择器)
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

        // 3. 复杂 Record 渲染：拉起树形折叠面板
        (Schema::Record(rect_schema), EditValue::Record(fields)) => {
            egui::CollapsingHeader::new(format!("📦 Record: {}", label))
                .default_open(true)
                .show(ui, |ui| {
                    // 双重指针安全迭代（借用检查优化）
                    for (field_schema, (f_name, f_val)) in
                        rect_schema.fields.iter().zip(fields.iter_mut())
                    {
                        render_editor(ui, &field_schema.schema, f_val, f_name);
                    }
                });
        }

        // 4. 高级 Array 渲染：支持动态新增、删除条目
        (Schema::Array(arr_schema), EditValue::Array(items)) => {
            egui::CollapsingHeader::new(format!("📑 Array: {} ({})", label, items.len()))
                .default_open(true)
                .show(ui, |ui| {
                    let mut to_remove = None;

                    // 渲染数组内已有元素
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

                    // 拦截删除事件
                    if let Some(idx) = to_remove {
                        items.remove(idx);
                    }

                    // 💥 关键新增逻辑：点击按钮，动态派生出该 Array item 类型的新默认节点
                    if ui.button("➕ Add Element / Record").clicked() {
                        let new_item = generate_default_value(&arr_schema.items);
                        items.push(new_item);
                    }
                });
        }

        // 5. 高级 Union 切换（如：处理 ["null", "string"] 等可选字段）
        (Schema::Union(union_schema), EditValue::Union(current_idx, inner_val)) => {
            ui.horizontal(|ui| {
                ui.label(format!("⌥ {}:", label));

                let variants = union_schema.variants();
                let mut selected = *current_idx;

                // 渲染一个下拉框供用户自由切换当前 Union 生效的亚型
                egui::ComboBox::from_id_salt(label)
                    .selected_text(format!("{:?}", variants[selected]))
                    .show_ui(ui, |ui| {
                        for (i, var) in variants.iter().enumerate() {
                            ui.selectable_value(&mut selected, i, format!("{:?}", var));
                        }
                    });

                // 如果用户在运行时改变了 Union 的选型，销毁旧数据，生成新类型的默认树
                if selected != *current_idx {
                    *current_idx = selected;
                    *inner_val = Box::new(generate_default_value(&variants[selected]));
                }
            });

            // 往下缩进渲染选中的具体亚型
            ui.indent(label, |ui| {
                render_editor(
                    ui,
                    &union_schema.variants()[*current_idx],
                    inner_val,
                    "Value",
                );
            });
        }

        // 处理 Null / 异常降级 fallback
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
