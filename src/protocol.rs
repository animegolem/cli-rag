use serde::{Deserialize, Serialize};

// Stable protocol types for JSON/NDJSON outputs. Keep additions backward compatible.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub file: std::path::PathBuf,
    pub tags: Vec<String>,
    pub status: Option<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCount {
    pub topic: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    pub groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateHeader {
    pub ok: bool,
    pub doc_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateIssue {
    #[serde(rename = "type")]
    pub kind: String, // "error" | "warning"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMember {
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMinimal {
    pub root: String,
    pub nodes: Vec<ClusterMember>,
    pub edges: Vec<Edge>,
}
