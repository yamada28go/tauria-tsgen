struct MyState(String);

#[tauri::command]
fn test_state(state: tauri::State<MyState>) {
  assert_eq!(state.0 == "some state value", true);
}

#[tauri::command]
fn test_state2(state: tauri::State<MyState>,name: &str) {
  assert_eq!(state.0 == "some state value", true);
}

use tauri::State as MyWindow;

#[tauri::command]
fn test_state3(state: MyWindow<MyState>,name: &str) {
  assert_eq!(state.0 == "some state value", true);
}

// use tauri::State;
// fn test_state4(state: State<MyState>,name: &str) {
//   assert_eq!(state.0 == "some state value", true);
// }

