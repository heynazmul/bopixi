//! Postgres access for the dashboard reads: pool, migrations, and the
//! repository queries behind the M0 routes. The SQL mirrors what the
//! in-memory store exposes after hydration, so responses stay contract-
//! identical while writes still go through the TypeScript server.

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;

/// The bopixi drizzle migrations, applied verbatim. sqlx keeps its own
/// `_sqlx_migrations` journal, so this expects a database it created (or a
/// fresh one); a database migrated by drizzle would need baselining.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../../server/db/migrations");

pub const DEFAULT_DEV_DATABASE_URL: &str = "postgres://bopixi:bopixi@127.0.0.1:5432/bopixi";

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
pub struct CanvasRow {
    pub id: String,
    pub name: String,
    pub owner_id: Option<String>,
    pub workspace_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub frame_count: i64,
    pub preview_frame_id: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct CanvasMemberRow {
    pub canvas_id: String,
    pub user_id: String,
}

#[derive(Debug, FromRow)]
pub struct UserNameRow {
    pub id: String,
    pub name: String,
}

/// The task columns the dashboard strip needs; prompts and payloads stay in
/// the database.
#[derive(Debug, FromRow)]
pub struct TaskRow {
    pub canvas_id: String,
    pub id: String,
    pub agent_name: String,
    pub owner: Option<String>,
    pub status: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub failed_at: Option<i64>,
    pub pipeline: Option<String>,
    pub stage: Option<i32>,
}

#[derive(Debug, FromRow)]
pub struct WorkspaceRow {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub status: String,
    pub plan: Option<String>,
    pub interval: Option<String>,
    pub seats: i32,
    pub current_period_end: Option<i64>,
    pub cancel_at_period_end: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub role: String,
    pub member_count: i64,
    pub canvas_count: i64,
}

impl Db {
    pub async fn connect(database_url: &str) -> Result<Self, DbError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Canvases visible to a user: their own, ones they were invited to, and
    /// every canvas of every workspace they belong to (`listCanvases`).
    pub async fn canvases_for(&self, user_id: &str) -> Result<Vec<CanvasRow>, sqlx::Error> {
        sqlx::query_as::<_, CanvasRow>(
            r#"
            SELECT c.id,
                   c.name,
                   c.owner_id,
                   c.workspace_id,
                   c.created_at,
                   c.updated_at,
                   (SELECT count(*) FROM frames f WHERE f.canvas_id = c.id) AS frame_count,
                   (SELECT f.id FROM frames f
                     WHERE f.canvas_id = c.id
                     ORDER BY f.updated_at DESC, f.id
                     LIMIT 1) AS preview_frame_id
              FROM canvases c
             WHERE c.owner_id = $1
                OR EXISTS (SELECT 1 FROM canvas_members cm
                            WHERE cm.canvas_id = c.id AND cm.user_id = $1)
                OR (c.workspace_id IS NOT NULL
                    AND EXISTS (SELECT 1 FROM workspace_members wm
                                 WHERE wm.workspace_id = c.workspace_id AND wm.user_id = $1))
             ORDER BY c.updated_at DESC, c.id
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Invited members of the given canvases (owners are not listed here).
    pub async fn canvas_members(
        &self,
        canvas_ids: &[String],
    ) -> Result<Vec<CanvasMemberRow>, sqlx::Error> {
        if canvas_ids.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<_, CanvasMemberRow>(
            r#"
            SELECT canvas_id, user_id
              FROM canvas_members
             WHERE canvas_id = ANY($1)
             ORDER BY canvas_id, added_at, user_id
            "#,
        )
        .bind(canvas_ids)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn user_names(&self, user_ids: &[String]) -> Result<Vec<UserNameRow>, sqlx::Error> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<_, UserNameRow>(r#"SELECT id, name FROM "user" WHERE id = ANY($1)"#)
            .bind(user_ids)
            .fetch_all(&self.pool)
            .await
    }

    /// Newest first, matching `actions.getTasks` after hydration.
    pub async fn tasks_for_canvases(
        &self,
        canvas_ids: &[String],
    ) -> Result<Vec<TaskRow>, sqlx::Error> {
        if canvas_ids.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT canvas_id, id, agent_name, owner, status, started_at,
                   ended_at, failed_at, pipeline, stage
              FROM tasks
             WHERE canvas_id = ANY($1)
             ORDER BY started_at DESC, id
            "#,
        )
        .bind(canvas_ids)
        .fetch_all(&self.pool)
        .await
    }

    /// Workspaces the user belongs to, oldest first (`workspaces.listFor`).
    pub async fn workspaces_for(&self, user_id: &str) -> Result<Vec<WorkspaceRow>, sqlx::Error> {
        sqlx::query_as::<_, WorkspaceRow>(
            r#"
            SELECT w.id,
                   w.name,
                   w.owner_id,
                   w.status,
                   w.plan,
                   w.interval,
                   w.seats,
                   w.current_period_end,
                   w.cancel_at_period_end,
                   w.created_at,
                   w.updated_at,
                   m.role,
                   (SELECT count(*) FROM workspace_members x WHERE x.workspace_id = w.id) AS member_count,
                   (SELECT count(*) FROM canvases c WHERE c.workspace_id = w.id) AS canvas_count
              FROM workspaces w
              JOIN workspace_members m ON m.workspace_id = w.id AND m.user_id = $1
             ORDER BY w.created_at ASC, w.id
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    /// A live paid membership (`workspaces.planFor`, billing enabled).
    pub async fn has_live_workspace(&self, user_id: &str) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
              SELECT 1
                FROM workspace_members m
                JOIN workspaces w ON w.id = m.workspace_id
               WHERE m.user_id = $1
                 AND w.status IN ('active', 'trialing', 'past_due')
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
}
