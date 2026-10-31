//! Wire shapes for `GET /api/workspaces`, matching `workspaces.summaryFor`.

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    /// The viewer's role; `member` when the membership row is unrecognised.
    pub role: String,
    pub status: String,
    /// No billing on this server, or a live subscription.
    pub active: bool,
    pub plan: Option<String>,
    pub interval: Option<String>,
    pub seats: i32,
    pub member_count: i64,
    pub canvas_count: i64,
    /// Only present when truthy, exactly like the spread in `summaryFor`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_period_end: Option<i64>,
    pub cancel_at_period_end: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
