// events-api: Activity event API over SQLite (commit platform, pkgs/events-api)
//! Events API: stores repository activity events (pushes, PRs, CI runs) in
//! SQLite. `webhooks` forwards GitHub events here; dashboards read from it.
//!
//!   GET  /health                   -> {"ok": true}
//!   GET  /events?repo=owner/name   -> [{"id", "repo", "kind", "actor"}]
//!   POST /events {repo,kind,actor} -> 201 {"id", ...}

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

const KINDS: &[&str] = &["push", "pull_request", "workflow_run", "release"];

#[derive(Serialize, Deserialize, Debug, PartialEq, sqlx::FromRow)]
struct Event {
    id: i64,
    repo: String,
    kind: String,
    actor: String,
}

#[derive(Deserialize)]
struct NewEvent {
    repo: String,
    kind: String,
    actor: String,
}

#[derive(Deserialize)]
struct Filter {
    repo: Option<String>,
}

async fn connect(url: &str) -> sqlx::Result<SqlitePool> {
    let pool = SqlitePool::connect(url).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY, repo TEXT NOT NULL, kind TEXT NOT NULL, actor TEXT NOT NULL)",
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true })) }),
        )
        .route("/events", get(list_events).post(create_event))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}

async fn list_events(
    State(pool): State<SqlitePool>,
    Query(filter): Query<Filter>,
) -> Result<Json<Vec<Event>>, StatusCode> {
    sqlx::query_as::<_, Event>(
        "SELECT id, repo, kind, actor FROM events WHERE ?1 IS NULL OR repo = ?1 ORDER BY id",
    )
    .bind(filter.repo)
    .fetch_all(&pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_event(
    State(pool): State<SqlitePool>,
    Json(new): Json<NewEvent>,
) -> Result<(StatusCode, Json<Event>), StatusCode> {
    let valid_repo = new
        .repo
        .split_once('/')
        .is_some_and(|(o, n)| !o.is_empty() && !n.is_empty());
    if !valid_repo || !KINDS.contains(&new.kind.as_str()) || new.actor.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    sqlx::query_as::<_, Event>(
        "INSERT INTO events (repo, kind, actor) VALUES (?, ?, ?) RETURNING id, repo, kind, actor",
    )
    .bind(new.repo)
    .bind(new.kind)
    .bind(new.actor)
    .fetch_one(&pool)
    .await
    .map(|event| (StatusCode::CREATED, Json(event)))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".into());
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app(connect(&url).await?)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn call(app: &Router, method: &str, uri: &str, body: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    async fn test_app() -> Router {
        app(connect("sqlite::memory:").await.unwrap())
    }

    #[tokio::test]
    async fn health_is_ok() {
        let app = test_app().await;
        assert_eq!(
            call(&app, "GET", "/health", "").await,
            (StatusCode::OK, r#"{"ok":true}"#.into())
        );
    }

    #[tokio::test]
    async fn records_and_filters_events() {
        let app = test_app().await;
        let (status, body) = call(
            &app,
            "POST",
            "/events",
            r#"{"repo":"o/a","kind":"push","actor":"ada"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body, r#"{"id":1,"repo":"o/a","kind":"push","actor":"ada"}"#);
        call(
            &app,
            "POST",
            "/events",
            r#"{"repo":"o/b","kind":"release","actor":"bob"}"#,
        )
        .await;

        let (_, body) = call(&app, "GET", "/events?repo=o/b", "").await;
        assert_eq!(
            body,
            r#"[{"id":2,"repo":"o/b","kind":"release","actor":"bob"}]"#
        );
        let (_, body) = call(&app, "GET", "/events", "").await;
        assert_eq!(body.matches("\"id\"").count(), 2);
    }

    #[tokio::test]
    async fn rejects_invalid_events() {
        let app = test_app().await;
        for body in [
            r#"{"repo":"no-slash","kind":"push","actor":"ada"}"#,
            r#"{"repo":"o/a","kind":"deploy","actor":"ada"}"#,
            r#"{"repo":"o/a","kind":"push","actor":" "}"#,
        ] {
            assert_eq!(
                call(&app, "POST", "/events", body).await.0,
                StatusCode::UNPROCESSABLE_ENTITY,
                "{body}"
            );
        }
    }
}
