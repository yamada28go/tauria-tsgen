#[tauri::command]
async fn test_webview_window(webview_window: tauri::WebviewWindow) -> Result<String, String>{
  println!("WebviewWindow: {}", webview_window.label());
  return Ok(webview_window.label().to_string());
}
#[tauri::command]
async fn test_webview_window2(webview_window: tauri::WebviewWindow,name: &str) -> Result<String, String>{
  println!("WebviewWindow: {}", webview_window.label());
  return Ok(webview_window.label().to_string()+name);
}

use tauri::WebviewWindow as MyWindow;
#[tauri::command]
async fn test_webview_window3(webview_window: MyWindow,name: &str) -> Result<String, String>{
  println!("WebviewWindow: {}", webview_window.label());
  return Ok(webview_window.label().to_string()+name);
}

use tauri::WebviewWindow;

#[tauri::command]
async fn test_webview_window4(webview_window: WebviewWindow, name: &str) -> Result<String, String> {
    println!("WebviewWindow: {}", webview_window.label());
    Ok(webview_window.label().to_string() + name)
}