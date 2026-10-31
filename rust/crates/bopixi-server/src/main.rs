//! The `bopixi-server` binary: environment, Postgres, migrations, and the axum
//! app on `PORT` (4400 by default).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use bopixi_core::Env;
use bopixi_db::Db;
use tokio::net::TcpListener;

/// The repository root — env files, `dist/` — resolved from this crate, not
/// the process working directory, so the server can start from anywhere.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[tokio::main]
async fn main() -> Result<()> {
    let root = repo_root();
    /* the TS server reads the same file via tsx --env-file-if-exists */
    let _ = dotenvy::from_path(root.join(".env"));
    init_tracing();

    let env = Env::parse().map_err(|err| anyhow!(err))?;
    if env.disable_auth() {
        tracing::warn!("DISABLE_AUTH is on: every request acts as the local admin");
    }

    /* production already refused a missing DATABASE_URL during parse */
    let database_url = env
        .database_url()
        .map(str::to_string)
        .unwrap_or_else(|| bopixi_db::DEFAULT_DEV_DATABASE_URL.to_string());

    let db = Db::connect(&database_url)
        .await
        .context("connecting to Postgres (set DATABASE_URL, or start the dev Postgres)")?;
    db.migrate().await.context("applying migrations")?;

    let port = env.port();
    let app = bopixi_api::router(Arc::new(env), db, root.join("dist"));

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .with_context(|| format!("binding 0.0.0.0:{port}"))?;
    tracing::info!("bopixi-server listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await.context("serving HTTP")?;
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("bopixi_server=info,bopixi_api=info,bopixi_db=info,tower_http=info,info")
    });
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
