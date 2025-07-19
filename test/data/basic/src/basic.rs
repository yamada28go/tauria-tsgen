use serde::{Deserialize, Serialize};

/**
 * @brief Greets the user.
 * @param name The name of the user.
 * @returns A greeting message.
 */
#[tauri::command]
fn greet(name: String) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/**
 * @brief Adds two numbers.
 * @param a The first number.
 * @param b The second number.
 * @returns The sum of the two numbers.
 */
#[tauri::command]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/**
 * @brief Gets a user by ID.
 * @param id The ID of the user.
 * @returns The user with the specified ID.
 */
#[tauri::command]
fn get_user(id: u32) -> String {
    format!("User with id: {}", id)
}

/**
 * @brief Updates a user.
 * @param user_name The name of the user to update.
 * @returns A message indicating the user has been updated.
 */
#[tauri::command]
fn update_user(user_name: String) -> String {
    format!("Updated user: {}", user_name)
}