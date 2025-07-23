use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MainPayload {
    pub message: String,
    pub value: u32,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SubPayload {
    pub data: String,
}

#[tauri::command]
fn emit_main_event(app: AppHandle, payload: MainPayload) {
    app.emit_to("main","main_event", MainPayload { message: "test".to_string(), value: 1 }).unwrap();
}

#[tauri::command]
fn emit_sub_event(window: WebviewWindow, payload: SubPayload) {
    window.emit("sub_event", SubPayload { data: "test".to_string() }).unwrap();
}

#[tauri::command]
fn emit_another_main_event(app: AppHandle) {
    app.emit_to("main","another_main_event", "simple string").unwrap();
}