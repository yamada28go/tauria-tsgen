use tauri::Window;

#[tauri::command]
async fn my_custom_command(window: Window) -> Result<(), String> {
    println!("Called from {}", window.label());
    Err("No result".into())
}