use serde::{Deserialize, Serialize};

/// Mirrors the Unity UserDetails class.
/// FileName is the primary key (GUID as string).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDetails {
    pub email: String,
    pub consent: bool,
    pub save_location: String,
    pub file_name: String,
}
