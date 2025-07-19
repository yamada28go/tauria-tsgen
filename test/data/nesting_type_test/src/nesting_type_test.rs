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

#[derive(Debug, Serialize, Deserialize)]
pub struct AppendEx{

    pub num : i32 

}


#[derive(Debug, Serialize, Deserialize)]
pub struct Append{

    pub num : i32 ,
    pub appendEx : AppendEx

}


#[derive(Debug, Serialize, Deserialize)]
pub struct Data{

    pub msg : Message ,
    pub append : Append
}

/**
 * @brief Processes a given message.
 * @param msg The message to process.
 * @returns A string indicating the processed message.
 */
#[tauri::command]
pub fn process_message(msg: Data) -> String {
    match msg.msg {
        Message::Quit => "Quit".to_string(),
        Message::Move { x, y } => format!("Move to ({}, {})", x, y),
        Message::Write(text) => format!("Write: {}", text),
        Message::ChangeColor(r, g, b) => format!("Change color to ({}, {}, {})", r, g, b),
    }
}
