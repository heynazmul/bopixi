//! Integration tests for Canvas CRUD and access control routes.
//! Gated on TEST_DATABASE_URL (admin connection string capable of creating
//! scratch databases for test isolation).

use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tower::ServiceExt;

use bopixi_api::router;
use bopixi_core::Env;
use bopixi_db::{Db, MIGRATOR};

async fn setup_test_db(admin_url: &str) -> (Db, String, sqlx::postgres::PgPool) {
    let admin_options =
        PgConnectOptions::from_str(admin_url).expect("TEST_DATABASE_URL must be a valid URL");
    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(admin_options.clone())
        .await
        .expect("connect to admin database");

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    let scratch_name = format!("bopixi_test_{}_{stamp}", std::process::id());

    sqlx::query(&format!(r#"create database "{scratch_name}""#))
        .execute(&admin)
        .await
        .expect("create scratch database");

    let scratch_url = format!("{}/{}", admin_url.trim_end_matches('/'), scratch_name);
    let db = Db::connect(&scratch_url)
        .await
        .expect("connect to scratch db");
    MIGRATOR.run(db.pool()).await.expect("run migrations");

    (db, scratch_name, admin)
}

async fn teardown_test_db(admin: sqlx::postgres::PgPool, scratch_name: &str) {
    sqlx::query(&format!(
        r#"drop database if exists "{scratch_name}" with (force)"#
    ))
    .execute(&admin)
    .await
    .expect("drop scratch database");
    admin.close().await;
}

#[tokio::test]
async fn test_canvas_crud_and_access_lifecycle() {
    let Ok(admin_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("TEST_DATABASE_URL is unset; test_canvas_crud_and_access_lifecycle skipped");
        return;
    };

    let (db, scratch_name, admin) = setup_test_db(&admin_url).await;

    // Seed local user in scratch db
    sqlx::query(
        r#"
        INSERT INTO "user" (id, name, email, email_verified, created_at, updated_at)
        VALUES ('local-user', 'Local Developer', 'dev@local.bopixi', true, 1000, 1000)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .execute(db.pool())
    .await
    .expect("seed local user");

    let env = Arc::new(
        Env::parse_from([("NODE_ENV", "development"), ("DISABLE_AUTH", "true")])
            .expect("valid env"),
    );

    let temp_dist = std::env::temp_dir().join(format!("dist_{}", scratch_name));
    std::fs::create_dir_all(&temp_dist).expect("create temp dist dir");
    std::fs::write(temp_dist.join("index.html"), "<html></html>").expect("write index.html");

    let app = router(env, db.clone(), temp_dist.clone());

    // 1. POST /api/canvases - create canvas
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/canvases")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "name": "Sprint 1 Board" }).to_string()))
        .expect("create request");

    let res = app.clone().oneshot(create_req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("bytes");
    let created: Value = serde_json::from_slice(&bytes).expect("parse json");
    let canvas_id = created["id"].as_str().expect("id").to_string();
    assert_eq!(created["name"], "Sprint 1 Board");
    assert_eq!(created["ownerId"], "local-user");
    assert!(created["pages"].as_array().is_some());

    // 2. GET /api/canvases/:id - read back full detail
    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/api/canvases/{canvas_id}"))
        .body(Body::empty())
        .expect("get request");

    let res = app.clone().oneshot(get_req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("bytes");
    let detail: Value = serde_json::from_slice(&bytes).expect("parse json");
    assert_eq!(detail["id"], canvas_id);
    assert_eq!(detail["role"], "edit");

    // 3. PATCH /api/canvases/:id - rename and set linkAccess
    let patch_req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/canvases/{canvas_id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "Sprint 1 Board (Renamed)",
                "linkAccess": "view"
            })
            .to_string(),
        ))
        .expect("patch request");

    let res = app.clone().oneshot(patch_req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);

    // Verify rename and linkAccess
    let get_req2 = Request::builder()
        .method("GET")
        .uri(format!("/api/canvases/{canvas_id}"))
        .body(Body::empty())
        .expect("get request");
    let res = app.clone().oneshot(get_req2).await.expect("response");
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("bytes");
    let updated: Value = serde_json::from_slice(&bytes).expect("parse json");
    assert_eq!(updated["name"], "Sprint 1 Board (Renamed)");
    assert_eq!(updated["linkAccess"], "view");

    // 4. POST /api/canvases/:id/duplicate - duplicate canvas
    let dup_req = Request::builder()
        .method("POST")
        .uri(format!("/api/canvases/{canvas_id}/duplicate"))
        .body(Body::empty())
        .expect("dup request");

    let res = app.clone().oneshot(dup_req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("bytes");
    let copy: Value = serde_json::from_slice(&bytes).expect("parse json");
    let copy_id = copy["id"].as_str().expect("copy id").to_string();
    assert_ne!(copy_id, canvas_id);
    assert_eq!(copy["name"], "Sprint 1 Board (Renamed) copy");

    // 5. DELETE /api/canvases/:id - delete copy
    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/canvases/{copy_id}"))
        .body(Body::empty())
        .expect("delete request");

    let res = app.clone().oneshot(delete_req).await.expect("response");
    assert_eq!(res.status(), StatusCode::OK);

    // 6. GET /api/canvases/:id on deleted canvas -> 404
    let get_deleted = Request::builder()
        .method("GET")
        .uri(format!("/api/canvases/{copy_id}"))
        .body(Body::empty())
        .expect("get deleted");
    let res = app.clone().oneshot(get_deleted).await.expect("response");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Cleanup scratch db & temp files
    let _ = std::fs::remove_dir_all(&temp_dist);
    teardown_test_db(admin, &scratch_name).await;
}
