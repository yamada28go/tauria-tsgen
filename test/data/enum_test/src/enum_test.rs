use serde::{Deserialize, Serialize};

/**
 * @brief Represents different types of messages.
 */
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Quit the application.
    Quit,
    /// Move to a new position.
    Move { x: i32, y: i32 },
    /// Write a message.
    Write(String),
    /// Change the color.
    ChangeColor(i32, i32, i32),
}

/**
 * @brief Processes a given message.
 * @param msg The message to process.
 * @returns A string indicating the processed message.
 */
#[tauri::command]
pub fn process_message(msg: Message) -> String {
    match msg {
        Message::Quit => "Quit".to_string(),
        Message::Move { x, y } => format!("Move to ({}, {})", x, y),
        Message::Write(text) => format!("Write: {}", text),
        Message::ChangeColor(r, g, b) => format!("Change color to ({}, {}, {})", r, g, b),
    }
}
