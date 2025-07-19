#[tauri::command]
fn read_file1() -> tauri::ipc::Response {
  let data = std::fs::read("path").unwrap();
  tauri::ipc::Response::new(data)
}

use tauri::ipc::Response;
#[tauri::command]
fn read_file2() -> Response {
  let data = std::fs::read("path").unwrap();
  tauri::ipc::Response::new(data)
}