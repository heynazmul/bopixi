//! `GET /api/me` response (`server/index.ts`).

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Me {
    pub id: String,
    pub name: String,
    pub email: String,
    pub admin: bool,
    /// `free` or `team` — the account menu's plan line.
    pub plan: &'static str,
    /// Present only while an admin is viewing as another user (never under
    /// `DISABLE_AUTH`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonating: Option<Impersonation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Impersonation {
    pub by_name: String,
}
