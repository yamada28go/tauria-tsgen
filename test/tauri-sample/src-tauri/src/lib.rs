// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Deserialize;
use serde::Serialize;

#[tauri::command]
fn greet_command(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Serialize, Deserialize)]
struct MultiplicationArg {
    a: i32,
    b: i32,
}

#[tauri::command]
fn multiplication_command(data: MultiplicationArg) -> i32 {
    return data.a * data.b;
}

#[tauri::command(rename_all = "snake_case")]
fn error_handling_command(is_error: bool) -> Result<String, String> {
    if is_error {
        Err("err ".to_string())
    } else {
        Ok("OK".to_string())
    }
}

use tauri::ipc::Response;
#[tauri::command]
fn read_file_command(path: &str) -> Response {
    let data = std::fs::read(path).unwrap();
    tauri::ipc::Response::new(data)
}

#[tauri::command]
fn app_handle_command(app: tauri::AppHandle) -> Result<String, String> {
    // バンドル識別子取得
    let identifier = &app.config().identifier;
    Ok(identifier.to_string())
}

#[tauri::command]
fn call_msg_global(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Emitter;
    let _ = app.emit("global", "msg-global").unwrap();

    Ok("ok".to_string())
}

#[tauri::command]
fn call_msg_main(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Emitter;
    let _ = app.emit_to("main", "core-msg", "Hello!!").unwrap();
    Ok("ok".to_string())
}

#[tauri::command]
async fn webview_window_command(webview_window: tauri::WebviewWindow) -> Result<String, String> {
    println!("WebviewWindow: {}", webview_window.label());
    return Ok(webview_window.label().to_string());
}

#[tauri::command]
async fn webview_window_with_arg_command(
    webview_window: tauri::WebviewWindow,
    name: &str,
) -> Result<String, String> {
    println!("WebviewWindow: {}", webview_window.label());
    return Ok(webview_window.label().to_string() + name);
}

struct Database;

#[derive(serde::Serialize)]
struct CustomResponse {
    message: String,
    other_val: usize,
}

async fn some_other_function() -> Option<String> {
    Some("response".into())
}

#[tauri::command]
async fn complex_state_command(
    window: tauri::Window,
    number: usize,
    database: tauri::State<'_, Database>,
) -> Result<CustomResponse, String> {
    println!("Called from {}", window.label());
    let result: Option<String> = some_other_function().await;
    if let Some(message) = result {
        Ok(CustomResponse {
            message,
            other_val: 42 + number,
        })
    } else {
        Err("No result".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet_command,
            multiplication_command,
            error_handling_command,
            read_file_command,
            webview_window_command,
            webview_window_with_arg_command,
            complex_state_command,
            app_handle_command,
            call_msg_global,
            call_msg_main
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
