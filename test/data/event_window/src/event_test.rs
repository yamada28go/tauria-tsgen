#[derive(Clone, serde::Serialize)]
pub struct EventPayload {
    pub message: String,
}

#[tauri::command]
fn event_test_command(app: tauri::AppHandle) {
    use tauri::Emitter;
    app.emit_to(
        "main",
        "window-event",
        EventPayload {
            message: "payload-struct".to_string(),
        },
    )
    .unwrap();
}
