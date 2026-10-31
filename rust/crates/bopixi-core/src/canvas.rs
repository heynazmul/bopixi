//! Wire shapes for the dashboard canvas list (`GET /api/canvases`) and full
//! canvas detail and mutations (`/api/canvases/:id`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CanvasRole {
    View,
    Comment,
    Edit,
}

impl CanvasRole {
    #[must_use]
    pub fn allows(self, need: CanvasRole) -> bool {
        self >= need
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            CanvasRole::View => "view",
            CanvasRole::Comment => "comment",
            CanvasRole::Edit => "edit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkAccess {
    None,
    View,
    Comment,
    Edit,
}

impl LinkAccess {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            LinkAccess::None => "none",
            LinkAccess::View => "view",
            LinkAccess::Comment => "comment",
            LinkAccess::Edit => "edit",
        }
    }

    #[must_use]
    pub fn to_role(self) -> Option<CanvasRole> {
        match self {
            LinkAccess::None => None,
            LinkAccess::View => Some(CanvasRole::View),
            LinkAccess::Comment => Some(CanvasRole::Comment),
            LinkAccess::Edit => Some(CanvasRole::Edit),
        }
    }

    #[must_use]
    pub fn parse_level(s: &str) -> Option<Self> {
        match s {
            "none" => Some(LinkAccess::None),
            "view" => Some(LinkAccess::View),
            "comment" => Some(LinkAccess::Comment),
            "edit" => Some(LinkAccess::Edit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasPageInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasGuide {
    pub id: String,
    pub page_id: String,
    pub axis: String,
    pub at: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameApproval {
    pub status: String,
    pub approved_by: String,
    pub approved_by_user_id: String,
    pub approved_at: i64,
    pub approved_frame_updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub id: String,
    pub canvas_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_id: Option<String>,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub html: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub updated_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<FrameApproval>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidelineDoc {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub markdown: String,
    pub updated_at: i64,
    pub updated_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryReference {
    pub id: String,
    pub frame_id: String,
    pub title: String,
    pub html: String,
    pub width: f64,
    pub height: f64,
    pub pinned_by: String,
    pub pinned_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_access: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<CanvasRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_count: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
    pub frames: Vec<Frame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Vec<CanvasPageInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guides: Option<Vec<CanvasGuide>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidelines: Option<Vec<GuidelineDoc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<MemoryReference>>,
}

#[must_use]
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[must_use]
pub fn generate_canvas_id() -> String {
    nanoid::nanoid!(10)
}

#[must_use]
pub fn default_pages(canvas_id: &str) -> Vec<CanvasPageInfo> {
    vec![CanvasPageInfo {
        id: format!("{canvas_id}:page1"),
        name: "Page 1".to_string(),
        start_frame_id: None,
    }]
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_role_hierarchy() {
        assert!(CanvasRole::Edit.allows(CanvasRole::View));
        assert!(CanvasRole::Edit.allows(CanvasRole::Comment));
        assert!(CanvasRole::Edit.allows(CanvasRole::Edit));

        assert!(CanvasRole::Comment.allows(CanvasRole::View));
        assert!(CanvasRole::Comment.allows(CanvasRole::Comment));
        assert!(!CanvasRole::Comment.allows(CanvasRole::Edit));

        assert!(CanvasRole::View.allows(CanvasRole::View));
        assert!(!CanvasRole::View.allows(CanvasRole::Comment));
        assert!(!CanvasRole::View.allows(CanvasRole::Edit));
    }

    #[test]
    fn link_access_parsing() {
        assert_eq!(LinkAccess::parse_level("none"), Some(LinkAccess::None));
        assert_eq!(LinkAccess::parse_level("view"), Some(LinkAccess::View));
        assert_eq!(
            LinkAccess::parse_level("comment"),
            Some(LinkAccess::Comment)
        );
        assert_eq!(LinkAccess::parse_level("edit"), Some(LinkAccess::Edit));
        assert_eq!(LinkAccess::parse_level("invalid"), None);

        assert_eq!(LinkAccess::None.to_role(), None);
        assert_eq!(LinkAccess::View.to_role(), Some(CanvasRole::View));
        assert_eq!(LinkAccess::Comment.to_role(), Some(CanvasRole::Comment));
        assert_eq!(LinkAccess::Edit.to_role(), Some(CanvasRole::Edit));
    }

    #[test]
    fn generate_canvas_id_format() {
        let id = generate_canvas_id();
        assert_eq!(id.len(), 10);
    }
}
