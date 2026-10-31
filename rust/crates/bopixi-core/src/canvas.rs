//! Wire shapes for the dashboard canvas list (`GET /api/canvases`), matching
//! `store.toMeta` plus the people/agents/tasks decoration added by the route.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PersonRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub last_at: i64,
}

/// A still-open agent task: the dashboard status strip only. Prompts,
/// ownership and completed history stay on the server.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTask {
    pub id: String,
    pub agent_name: String,
    pub status: String,
    pub started_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasListEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    /// `true` only when the canvas belongs to someone else; the key is omitted
    /// for the viewer's own canvases, exactly like `toMeta`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub frame_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_frame_id: Option<String>,
    pub people: Vec<PersonRef>,
    pub agents: Vec<AgentRef>,
    pub active_tasks: Vec<ActiveTask>,
}
