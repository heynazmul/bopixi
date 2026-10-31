# Rust Porting Notes: Bopixi Backend

This document records the migration of the Bopixi backend from TypeScript (Node.js/Express/Drizzle) to high-performance idiomatic Rust (Axum/Tokio/SQLx).

---

## 1. Architecture & Crates

The Rust backend is organized as a flat workspace under `rust/`:

| Crate | Purpose |
|---|---|
| [`bopixi-core`](file:///Volumes/1TB%20SSD/motion%20graphics/doop-main/rust/crates/bopixi-core) | Domain models (`Canvas`, `Frame`, `CanvasRole`, `LinkAccess`, `Env`, `Me`, `WorkspaceSummary`) |
| [`bopixi-db`](file:///Volumes/1TB%20SSD/motion%20graphics/doop-main/rust/crates/bopixi-db) | Async PostgreSQL access via SQLx (migrations, queries, atomic cascade deletions) |
| [`bopixi-auth`](file:///Volumes/1TB%20SSD/motion%20graphics/doop-main/rust/crates/bopixi-auth) | Authentication & actor resolution (`DISABLE_AUTH`, Better Auth sessions) |
| [`bopixi-api`](file:///Volumes/1TB%20SSD/motion%20graphics/doop-main/rust/crates/bopixi-api) | Axum HTTP router, endpoint handlers, contract-identical JSON responses |
| [`bopixi-server`](file:///Volumes/1TB%20SSD/motion%20graphics/doop-main/rust/crates/bopixi-server) | Tokio runtime entry point, tracing setup, static SPA file server |

---

## 2. API Mapping

### Milestone 0: Bootstrap & Dashboard Reads
| HTTP Route | TypeScript Source | Rust Handler |
|---|---|---|
| `GET /healthz` | `server/index.ts` | `bopixi_api::healthz` |
| `GET /api/auth/get-session` | `server/index.ts` (Better Auth) | `bopixi_api::get_session` |
| `GET /api/me` | `server/index.ts` | `bopixi_api::me` |
| `GET /api/canvases` | `server/index.ts` | `bopixi_api::canvases` |
| `GET /api/workspaces` | `server/workspaces.ts` | `bopixi_api::workspaces` |
| `/*` (SPA fallback) | `server/index.ts` (Express static) | `bopixi_api::router` (`ServeDir` + fallback) |

### Milestone 1: Canvas CRUD & Access Control
| HTTP Route | TypeScript Source | Rust Handler |
|---|---|---|
| `POST /api/canvases` | `server/index.ts:1129` (`createCanvas`) | `bopixi_api::create_canvas` |
| `GET /api/canvases/:id` | `server/index.ts:1156` (`requireCanvas`) | `bopixi_api::get_canvas_detail` |
| `PATCH /api/canvases/:id` | `server/index.ts:1239` (`renameCanvas`, `linkAccess`) | `bopixi_api::patch_canvas` |
| `DELETE /api/canvases/:id` | `server/index.ts:1227` (`deleteCanvas`) | `bopixi_api::delete_canvas` |
| `PUT /api/canvases/:id/workspace` | `server/index.ts:1187` (`setWorkspace`) | `bopixi_api::move_canvas_workspace` |
| `POST /api/canvases/:id/claim` | `server/index.ts:1208` (`claimCanvas`) | `bopixi_api::claim_canvas` |
| `POST /api/canvases/:id/duplicate` | `server/index.ts:1136` (`duplicateCanvas`) | `bopixi_api::duplicate_canvas` |

---

## 3. Access Control & Authorization Mapping

Canvas authorization follows Figma-style permissions implemented identically to `server/access.ts`:

| Rule | TypeScript | Rust (`bopixi-db::check_canvas_access`) |
|---|---|---|
| Unowned / Legacy Canvas | `!canvas.ownerId => 'edit'` | `owner_id.is_none() => role: Some(CanvasRole::Edit), can_manage: false` |
| Canvas Owner | `canvas.ownerId === userId => 'edit'` | `owner_id == userId => role: Some(CanvasRole::Edit), can_manage: true, durable: true` |
| Invited Member | `canvas.memberIds.includes(userId) => 'edit'` | `is_canvas_member => role: Some(CanvasRole::Edit), can_manage: false, durable: true` |
| Workspace Member | `isWorkspaceMember(canvas.workspaceId, userId) => 'edit'` | `is_ws_member => role: Some(CanvasRole::Edit), durable: true` |
| Workspace Admin / Owner | `hasRole(workspaceId, userId, 'admin')` | `is_ws_admin => can_manage: true` |
| Share Link Level | `canvas.linkAccess in ('view', 'comment', 'edit')` | `link_access.to_role() => role: Some(role), can_manage: false, durable: false` |
| Private / None | `null` | `None => 403 Forbidden` |

---

## 4. Deliberate Differences & Optimizations

1. **Transactional Cascade Deletion**:
   - The TypeScript server used asynchronous fire-and-forget `swallow(db.delete(...))` across 14 tables without a wrapping transaction.
   - The Rust implementation wraps the entire deletion sequence in an atomic PostgreSQL transaction (`BEGIN ... COMMIT`), preventing partial/orphaned records during crashes or connection drops.
2. **Compile-Time Type Safety**:
   - `CanvasRole` and `LinkAccess` are strongly typed Rust enums with `allows()` comparison instead of loose strings and rank dictionaries.
   - `GrowableWorkspaceResult` distinguishes between `NotFoundOrNotMember`, `PlanRequired` (402), and `Active` at the type level.
3. **No Garbage Collection Overhead**:
   - High-throughput endpoints run with zero runtime overhead, async Tokio thread pools, and prepared SQL queries.

---

## 5. Verification Commands

The following commands were run and verified 100% clean:

```bash
# Code formatting check
~/.cargo/bin/cargo fmt --manifest-path rust/Cargo.toml --all -- --check

# Strict Clippy linter with all warnings denied
~/.cargo/bin/cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features -- -D warnings

# Full unit and integration test suite
~/.cargo/bin/cargo test --manifest-path rust/Cargo.toml

# Documentation check
~/.cargo/bin/cargo doc --no-deps --manifest-path rust/Cargo.toml
```
