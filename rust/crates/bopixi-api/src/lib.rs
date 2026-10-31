//! The M0 HTTP surface: `/healthz`, the dashboard reads the React client
//! boots with (`/api/auth/get-session`, `/api/me`, `/api/canvases`,
//! `/api/workspaces`), and the built SPA. Every route answers with the same
//! status codes and JSON bodies as the Express server so the existing client
//! cannot tell which backend it is talking to.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{json, Value};
use tower_http::services::ServeDir;

use bopixi_core::{
    local_session, ActiveTask, AgentRef, CanvasListEntry, Env, Me, PersonRef, WorkspaceSummary,
};
use bopixi_db::{Db, TaskRow, WorkspaceRow};

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
        .route("/canvases", get(canvases))
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
    Unauthorized,
    NotFound,
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "unauthorized" })),
            )
                .into_response(),
            ApiError::NotFound => {
                (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response()
            }
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
    ApiError::NotFound
}
