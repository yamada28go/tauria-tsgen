#[tauri::command]
async fn test_app_handle(webview_window: tauri::AppHandle) -> Result<String, String>{
  println!("WebviewWindow: {}", webview_window.label());
  return Ok(webview_window.label().to_string());
}
#[tauri::command]
async fn test_app_handle2(webview_window: tauri::AppHandle,name: &str) -> Result<String, String>{
  println!("WebviewWindow: {}", webview_window.label());
  return Ok(webview_window.label().to_string()+name);
}

use tauri::AppHandle as MyWindow;
#[tauri::command]
async fn test_app_handle3(webview_window: MyWindow,name: &str) -> Result<String, String>{
  println!("WebviewWindow: {}", webview_window.label());
  return Ok(webview_window.label().to_string()+name);
}

use tauri::AppHandle;

#[tauri::command]
async fn test_app_handle4(webview_window: AppHandle, name: &str) -> Result<String, String> {
    println!("WebviewWindow: {}", webview_window.label());
    Ok(webview_window.label().to_string() + name)
}