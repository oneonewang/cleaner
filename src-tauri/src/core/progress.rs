//! 进度事件工具

use tauri::{AppHandle, Emitter};

use crate::models::scan_result::ProgressEvent;

pub fn emit_event(app: &AppHandle, event: ProgressEvent) {
    if let Err(e) = app.emit("scan-progress", &event) {
        eprintln!("emit error: {:?}", e);
    }
}

pub fn emit_started(app: &AppHandle, scan_id: &str, scan_type: &str) {
    emit_event(
        app,
        ProgressEvent::Started {
            scan_id: scan_id.to_string(),
            scan_type: scan_type.to_string(),
        },
    );
}

pub fn emit_progress(
    app: &AppHandle,
    scan_id: &str,
    percent: f32,
    category: Option<String>,
    current_path: Option<String>,
) {
    emit_event(
        app,
        ProgressEvent::Progress {
            scan_id: scan_id.to_string(),
            percent,
            category,
            current_path,
        },
    );
}

pub fn emit_category_done(app: &AppHandle, scan_id: &str, category: &str, bytes: u64) {
    emit_event(
        app,
        ProgressEvent::CategoryDone {
            scan_id: scan_id.to_string(),
            category: category.to_string(),
            bytes,
        },
    );
}

pub fn emit_finished(app: &AppHandle, scan_id: &str, total_bytes: u64) {
    emit_event(
        app,
        ProgressEvent::Finished {
            scan_id: scan_id.to_string(),
            total_bytes,
        },
    );
}

pub fn emit_cancelled(app: &AppHandle, scan_id: &str) {
    emit_event(
        app,
        ProgressEvent::Cancelled {
            scan_id: scan_id.to_string(),
        },
    );
}

pub fn emit_error(app: &AppHandle, scan_id: &str, message: &str) {
    emit_event(
        app,
        ProgressEvent::Error {
            scan_id: scan_id.to_string(),
            message: message.to_string(),
        },
    );
}
