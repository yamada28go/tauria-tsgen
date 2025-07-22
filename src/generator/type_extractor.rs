use serde_json;

#[derive(Debug, Clone, serde::Serialize)]
pub struct EventInfo {
    pub event_name: String,
    pub payload_type: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WindowEventInfo {
    pub window_name: String,
    pub event_name: String,
    pub payload_type: String,
}

pub struct ExtractedTypeInfo {
    pub name: String,
    pub ts_interface: serde_json::Value,
    pub is_serializable: bool,
    pub is_deserializable: bool,
}