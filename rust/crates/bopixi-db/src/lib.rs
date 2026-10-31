//! Postgres access for the dashboard reads and canvas CRUD: pool, migrations,
//! access queries, and repository writes. The SQL mirrors what the in-memory
//! store exposes after hydration, keeping responses contract-identical with
//! the TypeScript server.

use bopixi_core::{
    default_pages, generate_canvas_id, now_ms, Canvas, CanvasGuide, CanvasPageInfo, CanvasRole,
    Frame, FrameApproval, GuidelineDoc, LinkAccess, MemoryReference,
};
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
pub struct FullCanvasMetaRow {
    pub id: String,
    pub name: String,
    pub owner_id: Option<String>,
    pub link_access: Option<String>,
    pub published_at: Option<i64>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub copy_count: i32,
    pub workspace_id: Option<String>,
    pub pages: Option<sqlx::types::Json<Vec<CanvasPageInfo>>>,
    pub guides: Option<sqlx::types::Json<Vec<CanvasGuide>>>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, FromRow)]
pub struct FrameRow {
    pub id: String,
    pub canvas_id: String,
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
    pub demo: Option<bool>,
    pub hidden: Option<bool>,
    pub locked: Option<bool>,
    pub approval: Option<sqlx::types::Json<FrameApproval>>,
}

impl From<FrameRow> for Frame {
    fn from(r: FrameRow) -> Self {
        Self {
            id: r.id,
            canvas_id: r.canvas_id,
            page_id: r.page_id,
            name: r.name,
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
            html: r.html,
            created_at: r.created_at,
            updated_at: r.updated_at,
            updated_by: r.updated_by,
            demo: r.demo,
            hidden: r.hidden,
            locked: r.locked,
            approval: r.approval.map(|j| j.0),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct GuidelineRow {
    pub canvas_id: String,
    pub name: String,
    pub title: Option<String>,
    pub markdown: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

impl From<GuidelineRow> for GuidelineDoc {
    fn from(r: GuidelineRow) -> Self {
        Self {
            name: r.name,
            title: r.title,
            markdown: r.markdown,
            updated_at: r.updated_at,
            updated_by: r.updated_by,
            x: r.x,
            y: r.y,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct MemoryReferenceRow {
    pub id: String,
    pub canvas_id: String,
    pub frame_id: String,
    pub title: String,
    pub html: String,
    pub width: f64,
    pub height: f64,
    pub pinned_by: String,
    pub pinned_at: i64,
}

impl From<MemoryReferenceRow> for MemoryReference {
    fn from(r: MemoryReferenceRow) -> Self {
        Self {
            id: r.id,
            frame_id: r.frame_id,
            title: r.title,
            html: r.html,
            width: r.width,
            height: r.height,
            pinned_by: r.pinned_by,
            pinned_at: r.pinned_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasAccessCheck {
    pub owner_id: Option<String>,
    pub workspace_id: Option<String>,
    pub role: Option<CanvasRole>,
    pub can_manage: bool,
    pub has_durable_access: bool,
}

#[derive(Debug)]
pub enum GrowableWorkspaceResult {
    NotFoundOrNotMember,
    PlanRequired {
        workspace_id: String,
        workspace_name: String,
    },
    Active(WorkspaceRow),
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

#[derive(Debug, Clone, FromRow)]
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

    /// Check canvas access permissions according to Figma-style rules.
    pub async fn check_canvas_access(
        &self,
        canvas_id: &str,
        user_id: Option<&str>,
    ) -> Result<Option<CanvasAccessCheck>, sqlx::Error> {
        #[derive(FromRow)]
        struct AccessRow {
            owner_id: Option<String>,
            workspace_id: Option<String>,
            link_access: Option<String>,
        }

        let row = sqlx::query_as::<_, AccessRow>(
            "SELECT owner_id, workspace_id, link_access FROM canvases WHERE id = $1",
        )
        .bind(canvas_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let link_access = row.link_access.as_deref().and_then(LinkAccess::parse_level);

        // Ownerless legacy canvas stays open for claiming
        if row.owner_id.is_none() {
            return Ok(Some(CanvasAccessCheck {
                owner_id: None,
                workspace_id: row.workspace_id,
                role: Some(CanvasRole::Edit),
                can_manage: false,
                has_durable_access: false,
            }));
        }

        let Some(uid) = user_id else {
            return Ok(Some(CanvasAccessCheck {
                owner_id: row.owner_id,
                workspace_id: row.workspace_id,
                role: link_access.and_then(LinkAccess::to_role),
                can_manage: false,
                has_durable_access: false,
            }));
        };

        if row.owner_id.as_deref() == Some(uid) {
            return Ok(Some(CanvasAccessCheck {
                owner_id: row.owner_id,
                workspace_id: row.workspace_id,
                role: Some(CanvasRole::Edit),
                can_manage: true,
                has_durable_access: true,
            }));
        }

        let is_member = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM canvas_members WHERE canvas_id = $1 AND user_id = $2)",
        )
        .bind(canvas_id)
        .bind(uid)
        .fetch_one(&self.pool)
        .await?;

        let ws_role = if let Some(ws_id) = &row.workspace_id {
            sqlx::query_scalar::<_, String>(
                "SELECT role FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
            )
            .bind(ws_id)
            .bind(uid)
            .fetch_optional(&self.pool)
            .await?
        } else {
            None
        };

        let is_ws_member = ws_role.is_some();
        let is_ws_admin = matches!(ws_role.as_deref(), Some("owner") | Some("admin"));

        if is_member || is_ws_member {
            Ok(Some(CanvasAccessCheck {
                owner_id: row.owner_id,
                workspace_id: row.workspace_id,
                role: Some(CanvasRole::Edit),
                can_manage: is_ws_admin,
                has_durable_access: true,
            }))
        } else {
            Ok(Some(CanvasAccessCheck {
                owner_id: row.owner_id,
                workspace_id: row.workspace_id,
                role: link_access.and_then(LinkAccess::to_role),
                can_manage: false,
                has_durable_access: false,
            }))
        }
    }

    /// Retrieve the full canvas with frames, pages, guides, guidelines, and references.
    pub async fn get_canvas(&self, canvas_id: &str) -> Result<Option<Canvas>, sqlx::Error> {
        let meta = sqlx::query_as::<_, FullCanvasMetaRow>(
            r#"
            SELECT id, name, owner_id, link_access, published_at, description,
                   category, copy_count, workspace_id, pages, guides, created_at, updated_at
              FROM canvases
             WHERE id = $1
            "#,
        )
        .bind(canvas_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(meta) = meta else {
            return Ok(None);
        };

        let member_ids = sqlx::query_scalar::<_, String>(
            r#"
            SELECT user_id
              FROM canvas_members
             WHERE canvas_id = $1
             ORDER BY added_at
            "#,
        )
        .bind(canvas_id)
        .fetch_all(&self.pool)
        .await?;

        let frames = sqlx::query_as::<_, FrameRow>(
            r#"
            SELECT id, canvas_id, page_id, name, x, y, width, height, html,
                   created_at, updated_at, updated_by, demo, hidden, locked, approval
              FROM frames
             WHERE canvas_id = $1
             ORDER BY created_at
            "#,
        )
        .bind(canvas_id)
        .fetch_all(&self.pool)
        .await?;

        let guidelines = sqlx::query_as::<_, GuidelineRow>(
            r#"
            SELECT canvas_id, name, title, markdown, updated_at, updated_by, x, y
              FROM guidelines
             WHERE canvas_id = $1
             ORDER BY name
            "#,
        )
        .bind(canvas_id)
        .fetch_all(&self.pool)
        .await?;

        let references = sqlx::query_as::<_, MemoryReferenceRow>(
            r#"
            SELECT id, canvas_id, frame_id, title, html, width, height, pinned_by, pinned_at
              FROM memory_references
             WHERE canvas_id = $1
             ORDER BY pinned_at DESC
            "#,
        )
        .bind(canvas_id)
        .fetch_all(&self.pool)
        .await?;

        let pages = meta
            .pages
            .map(|j| j.0)
            .or_else(|| Some(default_pages(&meta.id)));
        let guides = meta.guides.map(|j| j.0);

        Ok(Some(Canvas {
            id: meta.id,
            name: meta.name,
            owner_id: meta.owner_id,
            link_access: meta.link_access,
            role: None,
            member_ids: if member_ids.is_empty() {
                None
            } else {
                Some(member_ids)
            },
            workspace_id: meta.workspace_id,
            published_at: meta.published_at,
            description: meta.description,
            category: meta.category,
            copy_count: if meta.copy_count == 0 {
                None
            } else {
                Some(meta.copy_count)
            },
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            frames: frames.into_iter().map(Frame::from).collect(),
            pages,
            guides,
            guidelines: if guidelines.is_empty() {
                None
            } else {
                Some(guidelines.into_iter().map(GuidelineDoc::from).collect())
            },
            references: if references.is_empty() {
                None
            } else {
                Some(references.into_iter().map(MemoryReference::from).collect())
            },
        }))
    }

    /// Create a new canvas with default pages and 0 frames.
    pub async fn create_canvas(
        &self,
        name: &str,
        owner_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Canvas, sqlx::Error> {
        let id = generate_canvas_id();
        let now = now_ms();
        let pages = default_pages(&id);
        let pages_json = sqlx::types::Json(&pages);

        sqlx::query(
            r#"
            INSERT INTO canvases (id, name, owner_id, workspace_id, pages, copy_count, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 0, $6, $7)
            "#,
        )
        .bind(&id)
        .bind(name)
        .bind(owner_id)
        .bind(workspace_id)
        .bind(pages_json)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(Canvas {
            id,
            name: name.to_string(),
            owner_id: owner_id.map(str::to_string),
            link_access: None,
            role: None,
            member_ids: None,
            workspace_id: workspace_id.map(str::to_string),
            published_at: None,
            description: None,
            category: None,
            copy_count: Some(0),
            created_at: now,
            updated_at: now,
            frames: Vec::new(),
            pages: Some(pages),
            guides: None,
            guidelines: None,
            references: None,
        })
    }

    /// Rename canvas and update `updated_at`.
    pub async fn rename_canvas(&self, canvas_id: &str, name: &str) -> Result<bool, sqlx::Error> {
        let now = now_ms();
        let res = sqlx::query("UPDATE canvases SET name = $1, updated_at = $2 WHERE id = $3")
            .bind(name)
            .bind(now)
            .bind(canvas_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    /// Update link access policy and update `updated_at`.
    pub async fn set_link_access(
        &self,
        canvas_id: &str,
        link_access: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let now = now_ms();
        let res =
            sqlx::query("UPDATE canvases SET link_access = $1, updated_at = $2 WHERE id = $3")
                .bind(link_access)
                .bind(now)
                .bind(canvas_id)
                .execute(&self.pool)
                .await?;
        Ok(res.rows_affected() > 0)
    }

    /// Move a canvas into a workspace or out to personal space (`workspace_id = None`).
    pub async fn move_canvas_workspace(
        &self,
        canvas_id: &str,
        workspace_id: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let now = now_ms();
        let res =
            sqlx::query("UPDATE canvases SET workspace_id = $1, updated_at = $2 WHERE id = $3")
                .bind(workspace_id)
                .bind(now)
                .bind(canvas_id)
                .execute(&self.pool)
                .await?;
        Ok(res.rows_affected() > 0)
    }

    /// Claim an unowned/legacy canvas for the given user.
    pub async fn claim_canvas(&self, canvas_id: &str, user_id: &str) -> Result<bool, sqlx::Error> {
        let now = now_ms();
        let res = sqlx::query(
            "UPDATE canvases SET owner_id = $1, updated_at = $2 WHERE id = $3 AND owner_id IS NULL",
        )
        .bind(user_id)
        .bind(now)
        .bind(canvas_id)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    /// Delete a canvas and cascade-delete all its dependent rows in a transaction.
    pub async fn delete_canvas(&self, canvas_id: &str) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 1. Delete asset_refs for frames belonging to canvas
        sqlx::query(
            r#"
            DELETE FROM asset_refs
             WHERE frame_id IN (SELECT id FROM frames WHERE canvas_id = $1)
            "#,
        )
        .bind(canvas_id)
        .execute(&mut *tx)
        .await?;

        // 2. Delete frames
        sqlx::query("DELETE FROM frames WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 3. Delete tasks
        sqlx::query("DELETE FROM tasks WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 4. Delete feedback
        sqlx::query("DELETE FROM feedback WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 5. Delete comments
        sqlx::query("DELETE FROM comments WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 6. Delete activity
        sqlx::query("DELETE FROM activity WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 7. Delete guidelines
        sqlx::query("DELETE FROM guidelines WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 8. Delete guideline_versions
        sqlx::query("DELETE FROM guideline_versions WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 9. Delete frame_versions
        sqlx::query("DELETE FROM frame_versions WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 10. Delete memory_references
        sqlx::query("DELETE FROM memory_references WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 11. Delete decisions
        sqlx::query("DELETE FROM decisions WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 12. Delete memory_proposals
        sqlx::query("DELETE FROM memory_proposals WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 13. Delete canvas_members
        sqlx::query("DELETE FROM canvas_members WHERE canvas_id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        // 14. Delete canvas
        let res = sqlx::query("DELETE FROM canvases WHERE id = $1")
            .bind(canvas_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(res.rows_affected() > 0)
    }

    /// Duplicate an existing canvas with its pages, frames, guidelines, and memory references.
    pub async fn duplicate_canvas(
        &self,
        source_id: &str,
        user_id: &str,
        target_workspace_id: Option<&str>,
    ) -> Result<Option<Canvas>, sqlx::Error> {
        let Some(source) = self.get_canvas(source_id).await? else {
            return Ok(None);
        };

        let new_canvas_id = generate_canvas_id();
        let now = now_ms();
        let copy_name = format!("{} copy", source.name);

        let mut frame_id_map = std::collections::HashMap::new();
        for f in &source.frames {
            frame_id_map.insert(f.id.clone(), nanoid::nanoid!(10));
        }

        let new_pages = source
            .pages
            .as_ref()
            .map(|pages| {
                pages
                    .iter()
                    .map(|p| {
                        let start = p
                            .start_frame_id
                            .as_deref()
                            .and_then(|id| frame_id_map.get(id).cloned());
                        CanvasPageInfo {
                            id: p.id.clone(),
                            name: p.name.clone(),
                            start_frame_id: start,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| default_pages(&new_canvas_id));

        let pages_json = sqlx::types::Json(&new_pages);
        let guides_json = source.guides.as_ref().map(sqlx::types::Json);

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO canvases (id, name, owner_id, workspace_id, pages, guides, copy_count, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, 0, $7, $8)
            "#,
        )
        .bind(&new_canvas_id)
        .bind(&copy_name)
        .bind(user_id)
        .bind(target_workspace_id)
        .bind(pages_json)
        .bind(guides_json)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let mut new_frames = Vec::new();
        for frame in &source.frames {
            let new_id = frame_id_map
                .get(&frame.id)
                .cloned()
                .unwrap_or_else(|| nanoid::nanoid!(10));
            sqlx::query(
                r#"
                INSERT INTO frames (id, canvas_id, page_id, name, x, y, width, height, html, created_at, updated_at, updated_by, demo, hidden, locked)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                "#,
            )
            .bind(&new_id)
            .bind(&new_canvas_id)
            .bind(&frame.page_id)
            .bind(&frame.name)
            .bind(frame.x)
            .bind(frame.y)
            .bind(frame.width)
            .bind(frame.height)
            .bind(&frame.html)
            .bind(now)
            .bind(now)
            .bind(user_id)
            .bind(frame.demo)
            .bind(frame.hidden)
            .bind(frame.locked)
            .execute(&mut *tx)
            .await?;

            new_frames.push(Frame {
                id: new_id,
                canvas_id: new_canvas_id.clone(),
                page_id: frame.page_id.clone(),
                name: frame.name.clone(),
                x: frame.x,
                y: frame.y,
                width: frame.width,
                height: frame.height,
                html: frame.html.clone(),
                created_at: now,
                updated_at: now,
                updated_by: user_id.to_string(),
                demo: frame.demo,
                hidden: frame.hidden,
                locked: frame.locked,
                approval: None,
            });
        }

        if let Some(guidelines) = &source.guidelines {
            for g in guidelines {
                sqlx::query(
                    r#"
                    INSERT INTO guidelines (canvas_id, name, title, markdown, updated_at, updated_by, x, y)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                )
                .bind(&new_canvas_id)
                .bind(&g.name)
                .bind(&g.title)
                .bind(&g.markdown)
                .bind(now)
                .bind(user_id)
                .bind(g.x)
                .bind(g.y)
                .execute(&mut *tx)
                .await?;
            }
        }

        if let Some(references) = &source.references {
            for r in references {
                let ref_id = nanoid::nanoid!(10);
                let frame_id = frame_id_map
                    .get(&r.frame_id)
                    .cloned()
                    .unwrap_or_else(|| r.frame_id.clone());
                sqlx::query(
                    r#"
                    INSERT INTO memory_references (id, canvas_id, frame_id, title, html, width, height, pinned_by, pinned_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                )
                .bind(&ref_id)
                .bind(&new_canvas_id)
                .bind(&frame_id)
                .bind(&r.title)
                .bind(&r.html)
                .bind(r.width)
                .bind(r.height)
                .bind(user_id)
                .bind(now)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(Some(Canvas {
            id: new_canvas_id,
            name: copy_name,
            owner_id: Some(user_id.to_string()),
            link_access: None,
            role: Some(CanvasRole::Edit),
            member_ids: None,
            workspace_id: target_workspace_id.map(str::to_string),
            published_at: None,
            description: None,
            category: None,
            copy_count: Some(0),
            created_at: now,
            updated_at: now,
            frames: new_frames,
            pages: Some(new_pages),
            guides: source.guides,
            guidelines: source.guidelines,
            references: source.references,
        }))
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

    /// Verify workspace membership and check if workspace is active and able to grow.
    pub async fn get_growable_workspace(
        &self,
        workspace_id: &str,
        user_id: &str,
        billing_enabled: bool,
    ) -> Result<GrowableWorkspaceResult, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkspaceRow>(
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
             WHERE w.id = $2
            "#,
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(GrowableWorkspaceResult::NotFoundOrNotMember);
        };

        let is_active =
            !billing_enabled || matches!(row.status.as_str(), "active" | "trialing" | "past_due");
        if !is_active {
            Ok(GrowableWorkspaceResult::PlanRequired {
                workspace_id: row.id,
                workspace_name: row.name,
            })
        } else {
            Ok(GrowableWorkspaceResult::Active(row))
        }
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
