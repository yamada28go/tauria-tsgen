use tauri::Window;

#[tauri::command]
fn app_handle_command(app: tauri::AppHandle) -> Result<String, String> {

  use tauri::Emitter;
let _ = app.emit("global", "msg-global").unwrap();

  // バンドル識別子取得
    let identifier = &app.config().identifier;
  Ok(identifier.to_string())
}
