use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteMod {
    pub remote_id: String,
    pub provider: String,
    pub name: String,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub latest_version: Option<String>,
    pub picture_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_large_url: Option<String>,
    pub detail_url: String,
    pub endorsement_count: u32,
    pub download_count: u32,
    pub unique_downloads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModSearchResult {
    pub items: Vec<RemoteMod>,
    pub total_count: u32,
    pub offset: u32,
    pub count: u32,
}
