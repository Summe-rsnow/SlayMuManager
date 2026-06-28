use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopMod {
    pub id: u64,
    pub name: String,
    pub author: String,
    pub description: String,
    pub preview_url: Option<String>,
    pub tags: Vec<String>,
    pub subscribers: u32,
    pub votes_up: u32,
    pub votes_down: u32,
    pub subscribed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopSearchResult {
    pub items: Vec<WorkshopMod>,
    pub total_count: u32,
}
