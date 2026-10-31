//! The HTTP API surface for Bopixi: `/healthz`, the dashboard bootstrap
//! reads (`/api/auth/get-session`, `/api/me`, `/api/canvases`, `/api/workspaces`),
//! the full Canvas CRUD and Access Control suite (`/api/canvases/:id`,
//! `/api/canvases/:id/workspace`, `/api/canvases/:id/claim`,
//! `/api/canvases/:id/duplicate`), and the built SPA. Every route answers with
//! the same status codes and JSON schemas as the Express server.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::services::ServeDir;

use bopixi_core::{
    local_session, ActiveTask, AgentRef, Canvas, CanvasListEntry, CanvasRole, Env, LinkAccess, Me,
    PersonRef, WorkspaceSummary,
};
use bopixi_db::{Db, GrowableWorkspaceResult, TaskRow, WorkspaceRow};

#[derive(Clone)]
pub struct AppState {
    pub env: Arc<Env>,
    pub db: Db,
}

pub fn router(env: Arc<Env>, db: Db, dist: PathBuf) -> Router {
    let state = AppState { env, db };
    let api = Router::new()
        .route("/auth/get-session", get(get_session))
        .route("/me", get(me))
        .route("/canvases", get(canvases).post(create_canvas))
        .route(
            "/canvases/{id}",
            get(get_canvas_detail)
                .patch(patch_canvas)
                .delete(delete_canvas),
        )
        .route("/canvases/{id}/workspace", put(move_canvas_workspace))
        .route("/canvases/{id}/claim", post(claim_canvas))
        .route("/canvases/{id}/duplicate", post(duplicate_canvas))
        .route("/workspaces", get(workspaces))
        .fallback(api_not_found);

    /* SPA: real files from `dist/`, otherwise index.html for client routes.
    The TS server keeps missing asset paths a 404 — answering a missing
    bundle or source map with HTML breaks module loads and stack maps. */
    let index = dist.join("index.html");
    let spa = ServeDir::new(&dist).fallback(get(move |uri: Uri| {
        let index = index.clone();
        async move {
            if uri.path().starts_with("/assets/") {
                return StatusCode::NOT_FOUND.into_response();
            }
            match tokio::fs::read(&index).await {
                Ok(html) => {
                    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
                }
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }));
    Router::new()
        .route("/healthz", get(healthz))
        .nest("/api", api)
        .fallback_service(spa)
        .with_state(state)
}

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    Forbidden(Value),
    NotFound(String),
    Conflict(String),
    PaymentRequired {
        workspace_id: String,
        message: String,
    },
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
            }
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "unauthorized" })),
            )
                .into_response(),
            ApiError::Forbidden(val) => (StatusCode::FORBIDDEN, Json(val)).into_response(),
            ApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response()
            }
            ApiError::Conflict(msg) => {
                (StatusCode::CONFLICT, Json(json!({ "error": msg }))).into_response()
            }
            ApiError::PaymentRequired {
                workspace_id,
                message,
            } => (
                StatusCode::PAYMENT_REQUIRED,
                Json(json!({
                    "error": "workspace_plan_required",
                    "workspaceId": workspace_id,
                    "message": message,
                })),
            )
                .into_response(),
            ApiError::Internal(message) => {
                tracing::error!("{message}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal error" })),
                )
                    .into_response()
            }
        }
    }
}

fn internal(err: impl std::fmt::Display) -> ApiError {
    ApiError::Internal(err.to_string())
}

fn require_actor(state: &AppState) -> Result<bopixi_auth::Actor, ApiError> {
    bopixi_auth::resolve(&state.env).ok_or(ApiError::Unauthorized)
}

async fn healthz() -> Json<Value> {
    Json(json!({ "ok": true }))
}

/// The SPA probes this on boot. Under `DISABLE_AUTH` it must answer as the
/// local user, or the app renders its sign-in screen anyway.
async fn get_session(State(state): State<AppState>) -> Json<Value> {
    if state.env.disable_auth() {
        Json(serde_json::to_value(local_session()).expect("session serializes"))
    } else {
        Json(Value::Null)
    }
}

async fn me(State(state): State<AppState>) -> Result<Json<Me>, ApiError> {
    let actor = require_actor(&state)?;
    let admin = actor.is_admin();
    let plan = if state.env.billing_enabled()
        && state
            .db
            .has_live_workspace(&actor.id)
            .await
            .map_err(internal)?
    {
        "team"
    } else {
        "free"
    };
    Ok(Json(Me {
        id: actor.id,
        name: actor.name,
        email: actor.email,
        admin,
        plan,
        impersonating: None,
    }))
}

async fn canvases(State(state): State<AppState>) -> Result<Json<Vec<CanvasListEntry>>, ApiError> {
    let actor = require_actor(&state)?;
    let rows = state.db.canvases_for(&actor.id).await.map_err(internal)?;
    let canvas_ids: Vec<String> = rows.iter().map(|row| row.id.clone()).collect();

    let members = state
        .db
        .canvas_members(&canvas_ids)
        .await
        .map_err(internal)?;
    let tasks = state
        .db
        .tasks_for_canvases(&canvas_ids)
        .await
        .map_err(internal)?;

    let mut members_by_canvas: HashMap<&str, Vec<&str>> = HashMap::new();
    for member in &members {
        members_by_canvas
            .entry(member.canvas_id.as_str())
            .or_default()
            .push(member.user_id.as_str());
    }

    /* names for everyone on every listed canvas, one query */
    let mut people_ids: Vec<String> = Vec::new();
    let mut people_seen: HashSet<&str> = HashSet::new();
    for row in &rows {
        if let Some(owner) = row.owner_id.as_deref() {
            if people_seen.insert(owner) {
                people_ids.push(owner.to_string());
            }
        }
        for member in members_by_canvas.get(row.id.as_str()).into_iter().flatten() {
            if people_seen.insert(member) {
                people_ids.push((*member).to_string());
            }
        }
    }
    let names: HashMap<String, String> = state
        .db
        .user_names(&people_ids)
        .await
        .map_err(internal)?
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect();

    let mut tasks_by_canvas: HashMap<&str, Vec<&TaskRow>> = HashMap::new();
    for task in &tasks {
        tasks_by_canvas
            .entry(task.canvas_id.as_str())
            .or_default()
            .push(task);
    }

    let name_of = |id: &str| {
        names
            .get(id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string())
    };

    let entries = rows
        .iter()
        .map(|row| {
            /* owner first, then invited members, each once */
            let mut people: Vec<PersonRef> = Vec::new();
            let mut seen: HashSet<&str> = HashSet::new();
            if let Some(owner) = row.owner_id.as_deref() {
                if seen.insert(owner) {
                    people.push(PersonRef {
                        id: owner.to_string(),
                        name: name_of(owner),
                    });
                }
            }
            for member in members_by_canvas.get(row.id.as_str()).into_iter().flatten() {
                if seen.insert(member) {
                    people.push(PersonRef {
                        id: (*member).to_string(),
                        name: name_of(member),
                    });
                }
            }

            let canvas_tasks = tasks_by_canvas
                .get(row.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or_default();

            /* which agents have worked on this canvas, most recent first */
            let mut agent_seen: HashSet<&str> = HashSet::new();
            let mut agents: Vec<AgentRef> = Vec::new();
            for task in canvas_tasks {
                if agents.len() >= 8 {
                    break;
                }
                if agent_seen.insert(task.agent_name.as_str()) {
                    agents.push(AgentRef {
                        name: task.agent_name.clone(),
                        owner: task.owner.clone(),
                        last_at: task.started_at,
                    });
                }
            }

            let active_tasks: Vec<ActiveTask> = canvas_tasks
                .iter()
                .filter(|task| task.ended_at.is_none() && task.failed_at.is_none())
                .map(|task| ActiveTask {
                    id: task.id.clone(),
                    agent_name: task.agent_name.clone(),
                    status: task.status.clone(),
                    started_at: task.started_at,
                    pipeline: task.pipeline.as_deref().map(|raw| {
                        raw.split(',')
                            .filter(|part| !part.is_empty())
                            .map(str::to_string)
                            .collect()
                    }),
                    stage: task.stage,
                })
                .collect();

            CanvasListEntry {
                id: row.id.clone(),
                name: row.name.clone(),
                owner_id: row.owner_id.clone(),
                shared: (row.owner_id.as_deref() != Some(actor.id.as_str())).then_some(true),
                workspace_id: row.workspace_id.clone(),
                created_at: row.created_at,
                updated_at: row.updated_at,
                frame_count: row.frame_count,
                preview_frame_id: row.preview_frame_id.clone(),
                people,
                agents,
                active_tasks,
            }
        })
        .collect();

    Ok(Json(entries))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCanvasRequest {
    pub name: Option<String>,
    pub workspace_id: Option<String>,
}

async fn create_canvas(
    State(state): State<AppState>,
    Json(payload): Json<CreateCanvasRequest>,
) -> Result<Json<Canvas>, ApiError> {
    let actor = require_actor(&state)?;
    if let Some(ws_id) = payload.workspace_id.as_deref() {
        match state
            .db
            .get_growable_workspace(ws_id, &actor.id, state.env.billing_enabled())
            .await
            .map_err(internal)?
        {
            GrowableWorkspaceResult::NotFoundOrNotMember => {
                return Err(ApiError::NotFound("workspace not found".into()));
            }
            GrowableWorkspaceResult::PlanRequired {
                workspace_id,
                workspace_name,
            } => {
                return Err(ApiError::PaymentRequired {
                    workspace_id,
                    message: format!("\"{workspace_name}\" needs a Team plan before it can grow"),
                });
            }
            GrowableWorkspaceResult::Active(_) => {}
        }
    }

    let name = payload
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "Untitled canvas".to_string());
    let canvas = state
        .db
        .create_canvas(&name, Some(&actor.id), payload.workspace_id.as_deref())
        .await
        .map_err(internal)?;
    Ok(Json(canvas))
}

async fn get_canvas_detail(
    State(state): State<AppState>,
    Path(canvas_id): Path<String>,
) -> Result<Json<Canvas>, ApiError> {
    let actor = require_actor(&state)?;
    let access = state
        .db
        .check_canvas_access(&canvas_id, Some(&actor.id))
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;

    let role = access.role.ok_or_else(|| {
        ApiError::Forbidden(json!({ "error": "this canvas is private — ask the owner for access" }))
    })?;

    if !role.allows(CanvasRole::View) {
        return Err(ApiError::Forbidden(
            json!({ "error": "this canvas is private — ask the owner for access" }),
        ));
    }

    let mut canvas = state
        .db
        .get_canvas(&canvas_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;

    canvas.role = Some(role);
    Ok(Json(canvas))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchCanvasRequest {
    pub name: Option<String>,
    pub link_access: Option<Value>,
}

async fn patch_canvas(
    State(state): State<AppState>,
    Path(canvas_id): Path<String>,
    Json(payload): Json<PatchCanvasRequest>,
) -> Result<Json<Value>, ApiError> {
    let actor = require_actor(&state)?;
    let access = state
        .db
        .check_canvas_access(&canvas_id, Some(&actor.id))
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;

    let role = access.role.ok_or_else(|| {
        ApiError::Forbidden(json!({ "error": "this canvas is private — ask the owner for access" }))
    })?;

    if !role.allows(CanvasRole::Edit) {
        return Err(ApiError::Forbidden(json!({
            "error": "you can view this canvas but not edit it — ask the owner for edit access",
            "role": role.as_str(),
        })));
    }

    if let Some(link_access_val) = &payload.link_access {
        if access.owner_id.as_deref() != Some(&actor.id) {
            return Err(ApiError::Forbidden(
                json!({ "error": "only the owner can change link access" }),
            ));
        }
        let Some(raw_str) = link_access_val.as_str() else {
            return Err(ApiError::BadRequest(
                "linkAccess must be \"none\", \"view\", \"comment\" or \"edit\"".into(),
            ));
        };
        let Some(level) = LinkAccess::parse_level(raw_str) else {
            return Err(ApiError::BadRequest(
                "linkAccess must be \"none\", \"view\", \"comment\" or \"edit\"".into(),
            ));
        };
        let stored_val = match level {
            LinkAccess::None => None,
            other => Some(other.as_str()),
        };
        state
            .db
            .set_link_access(&canvas_id, stored_val)
            .await
            .map_err(internal)?;
    }

    if let Some(name) = &payload.name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            state
                .db
                .rename_canvas(&canvas_id, trimmed)
                .await
                .map_err(internal)?;
        }
    }

    Ok(Json(json!({ "ok": true })))
}

async fn delete_canvas(
    State(state): State<AppState>,
    Path(canvas_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let actor = require_actor(&state)?;
    let access = state
        .db
        .check_canvas_access(&canvas_id, Some(&actor.id))
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;

    if access.owner_id.is_none() {
        return Err(ApiError::Forbidden(json!({ "error": "claim it first" })));
    }

    if !access.can_manage {
        return Err(ApiError::Forbidden(json!({ "error": "not yours" })));
    }

    state.db.delete_canvas(&canvas_id).await.map_err(internal)?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveWorkspaceRequest {
    pub workspace_id: Option<Value>,
}

async fn move_canvas_workspace(
    State(state): State<AppState>,
    Path(canvas_id): Path<String>,
    Json(payload): Json<MoveWorkspaceRequest>,
) -> Result<Json<Value>, ApiError> {
    let actor = require_actor(&state)?;
    let access = state
        .db
        .check_canvas_access(&canvas_id, Some(&actor.id))
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;

    let role = access.role.ok_or_else(|| {
        ApiError::Forbidden(json!({ "error": "this canvas is private — ask the owner for access" }))
    })?;

    if !role.allows(CanvasRole::Edit) {
        return Err(ApiError::Forbidden(json!({
            "error": "you can view this canvas but not edit it — ask the owner for edit access",
            "role": role.as_str(),
        })));
    }

    let target_val = payload.workspace_id.unwrap_or(Value::Null);
    let target_opt = match &target_val {
        Value::Null => None,
        Value::String(s) => Some(s.as_str()),
        _ => {
            return Err(ApiError::BadRequest(
                "workspaceId must be a workspace id or null".into(),
            ));
        }
    };

    if !access.can_manage {
        return Err(ApiError::Forbidden(json!({
            "error": "only the canvas owner or a workspace admin can move it"
        })));
    }

    if let Some(target) = target_opt {
        if Some(target) == access.workspace_id.as_deref() {
            return Ok(Json(json!({ "ok": true })));
        }
        if access.owner_id.as_deref() != Some(&actor.id) {
            return Err(ApiError::Forbidden(json!({
                "error": "only the canvas owner can move it into a workspace"
            })));
        }
        match state
            .db
            .get_growable_workspace(target, &actor.id, state.env.billing_enabled())
            .await
            .map_err(internal)?
        {
            GrowableWorkspaceResult::NotFoundOrNotMember => {
                return Err(ApiError::NotFound("workspace not found".into()));
            }
            GrowableWorkspaceResult::PlanRequired {
                workspace_id,
                workspace_name,
            } => {
                return Err(ApiError::PaymentRequired {
                    workspace_id,
                    message: format!("\"{workspace_name}\" needs a Team plan before it can grow"),
                });
            }
            GrowableWorkspaceResult::Active(_) => {}
        }
    }

    state
        .db
        .move_canvas_workspace(&canvas_id, target_opt)
        .await
        .map_err(internal)?;
    Ok(Json(json!({ "ok": true })))
}

async fn claim_canvas(
    State(state): State<AppState>,
    Path(canvas_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let actor = require_actor(&state)?;
    let claimed = state
        .db
        .claim_canvas(&canvas_id, &actor.id)
        .await
        .map_err(internal)?;

    if !claimed {
        return Err(ApiError::Conflict("not found or already owned".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

async fn duplicate_canvas(
    State(state): State<AppState>,
    Path(canvas_id): Path<String>,
) -> Result<Json<Canvas>, ApiError> {
    let actor = require_actor(&state)?;
    let access = state
        .db
        .check_canvas_access(&canvas_id, Some(&actor.id))
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;

    if !access.has_durable_access {
        return Err(ApiError::Forbidden(json!({ "error": "access denied" })));
    }

    let target_ws = if let Some(ws_id) = &access.workspace_id {
        match state
            .db
            .get_growable_workspace(ws_id, &actor.id, state.env.billing_enabled())
            .await
            .map_err(internal)?
        {
            GrowableWorkspaceResult::Active(_) => Some(ws_id.as_str()),
            _ => None,
        }
    } else {
        None
    };

    let copy = state
        .db
        .duplicate_canvas(&canvas_id, &actor.id, target_ws)
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("not found".into()))?;

    Ok(Json(copy))
}

#[derive(Serialize)]
struct WorkspaceList {
    workspaces: Vec<WorkspaceSummary>,
    billing: BillingFlag,
}

#[derive(Serialize)]
struct BillingFlag {
    enabled: bool,
}

fn summarize(row: &WorkspaceRow, billing_enabled: bool) -> WorkspaceSummary {
    let role = match row.role.as_str() {
        "owner" | "admin" | "member" => row.role.clone(),
        _ => "member".to_string(),
    };
    WorkspaceSummary {
        id: row.id.clone(),
        name: row.name.clone(),
        owner_id: row.owner_id.clone(),
        role,
        status: row.status.clone(),
        active: !billing_enabled
            || matches!(row.status.as_str(), "active" | "trialing" | "past_due"),
        plan: (row.plan.as_deref() == Some("team")).then(|| "team".to_string()),
        interval: match row.interval.as_deref() {
            Some("month") => Some("month".to_string()),
            Some("year") => Some("year".to_string()),
            _ => None,
        },
        seats: row.seats,
        member_count: row.member_count,
        canvas_count: row.canvas_count,
        current_period_end: row.current_period_end.filter(|end| *end != 0),
        cancel_at_period_end: row.cancel_at_period_end,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

async fn workspaces(State(state): State<AppState>) -> Result<Json<WorkspaceList>, ApiError> {
    let actor = require_actor(&state)?;
    let enabled = state.env.billing_enabled();
    let rows = state.db.workspaces_for(&actor.id).await.map_err(internal)?;
    Ok(Json(WorkspaceList {
        workspaces: rows.iter().map(|row| summarize(row, enabled)).collect(),
        billing: BillingFlag { enabled },
    }))
}

async fn api_not_found() -> ApiError {
    ApiError::NotFound("not found".into())
}
