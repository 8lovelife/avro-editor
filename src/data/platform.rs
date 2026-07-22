use crate::state::app_state::PendingFileOp;
use eframe::egui;
use std::sync::{Arc, Mutex};

type PendingOp = Arc<Mutex<Option<PendingFileOp>>>;

// ---------------- Native ----------------
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_and_read(
    filter_name: &'static str,
    filter_exts: &'static [&'static str],
    pending_op: PendingOp,
    make_op: impl FnOnce(String, Vec<u8>) -> PendingFileOp,
    _ctx: &egui::Context,
) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter_name, filter_exts)
        .pick_file()
    {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let result = match std::fs::read(&path) {
            Ok(bytes) => make_op(filename, bytes),
            Err(e) => PendingFileOp::Failed(format!("Failed to read file: {e}")),
        };
        *pending_op.lock().unwrap() = Some(result);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(
    filename: String,
    filter_name: &'static str,
    filter_exts: &'static [&'static str],
    data: Vec<u8>,
    pending_op: PendingOp,
    _ctx: &egui::Context,
) {
    if let Some(path) = rfd::FileDialog::new()
        .set_file_name(&filename)
        .add_filter(filter_name, filter_exts)
        .save_file()
    {
        let result = match std::fs::write(&path, &data) {
            Ok(_) => PendingFileOp::ExportDone {
                filename: path.to_string_lossy().into_owned(),
            },
            Err(e) => PendingFileOp::Failed(format!("Failed to write file: {e}")),
        };
        *pending_op.lock().unwrap() = Some(result);
    }
}

// ---------------- Web (wasm32) ----------------
#[cfg(target_arch = "wasm32")]
pub fn pick_and_read(
    filter_name: &'static str,
    filter_exts: &'static [&'static str],
    pending_op: PendingOp,
    make_op: impl FnOnce(String, Vec<u8>) -> PendingFileOp + Send + 'static,
    ctx: &egui::Context,
) {
    let task = rfd::AsyncFileDialog::new()
        .add_filter(filter_name, filter_exts)
        .pick_file();
    let ctx = ctx.clone();

    wasm_bindgen_futures::spawn_local(async move {
        if let Some(file) = task.await {
            let filename = file.file_name();
            let bytes = file.read().await;
            *pending_op.lock().unwrap() = Some(make_op(filename, bytes));
            ctx.request_repaint();
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn save(
    filename: String,
    filter_name: &'static str,
    filter_exts: &'static [&'static str],
    data: Vec<u8>,
    pending_op: PendingOp,
    ctx: &egui::Context,
) {
    let task = rfd::AsyncFileDialog::new()
        .set_file_name(&filename)
        .add_filter(filter_name, filter_exts)
        .save_file();
    let ctx = ctx.clone();

    wasm_bindgen_futures::spawn_local(async move {
        if let Some(file) = task.await {
            let result = match file.write(&data).await {
                Ok(_) => PendingFileOp::ExportDone { filename },
                Err(e) => PendingFileOp::Failed(format!("{e:?}")),
            };
            *pending_op.lock().unwrap() = Some(result);
            ctx.request_repaint();
        }
    });
}
