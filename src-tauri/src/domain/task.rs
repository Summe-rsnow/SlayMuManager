use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLogEntry {
    pub id: String,
    pub category: String,
    pub title: String,
    pub detail: Option<String>,
    pub created_at: String,
}
