use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;

use crate::capture;
use crate::config::Config;
use crate::diff;
use crate::dom_index;
use crate::spine::SpineClient;

pub struct AppState {
    pub config: Config,
    pub spine: SpineClient,
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    tracing::info!("Starting agent-eyes daemon...");

    let spine = SpineClient::new(&config.spine.url, "agent-eyes", env!("CARGO_PKG_VERSION"));
    spine.register().await?;

    let spine_clone = spine.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let _ = spine_clone.heartbeat().await;
        }
    });

    let port = config.server.port;
    let state = Arc::new(AppState { config, spine });

    let app = Router::new()
        .route("/health", get(health))
        .route("/capture", post(capture_url))
        .route("/diff", post(pixel_diff))
        .route("/dom/index", post(dom_index_url))
        .route("/dom/stats", get(dom_stats))
        .route("/dom/search", get(dom_search))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(_): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

#[derive(serde::Deserialize)]
struct CaptureRequest {
    url: String,
    output: Option<String>,
}

async fn capture_url(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CaptureRequest>,
) -> Json<serde_json::Value> {
    let output = req
        .output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("screenshot.png"));

    match capture::capture_url(&req.url, &output).await {
        Ok(_) => {
            let _ = state
                .spine
                .publish(
                    "eyes.captured",
                    &serde_json::json!({
                        "url": req.url,
                        "output": output.display().to_string(),
                    }),
                )
                .await;
            Json(serde_json::json!({
                "success": true,
                "url": req.url,
                "output": output.display().to_string()
            }))
        }
        Err(e) => Json(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}

#[derive(serde::Deserialize)]
struct DiffRequest {
    reference: String,
    comparison: String,
    output: Option<String>,
}

async fn pixel_diff(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DiffRequest>,
) -> Json<serde_json::Value> {
    let ref_path = PathBuf::from(&req.reference);
    let comp_path = PathBuf::from(&req.comparison);
    let out_path = req
        .output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("diff.png"));

    match diff::pixel_diff(&ref_path, &comp_path, &out_path) {
        Ok(_) => {
            let _ = state
                .spine
                .publish(
                    "eyes.diffed",
                    &serde_json::json!({
                        "reference": req.reference,
                        "comparison": req.comparison,
                        "output": out_path.display().to_string(),
                    }),
                )
                .await;
            Json(serde_json::json!({"success": true}))
        }
        Err(e) => Json(serde_json::json!({"success": false, "error": e.to_string()})),
    }
}

#[derive(serde::Deserialize)]
struct DomIndexRequest {
    url: String,
    #[serde(default = "default_max_elements")]
    max_elements: usize,
}

fn default_max_elements() -> usize {
    5000
}

async fn dom_index_url(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DomIndexRequest>,
) -> Json<serde_json::Value> {
    match dom_index::index_url(&req.url, req.max_elements).await {
        Ok(report) => {
            let _ = state
                .spine
                .publish(
                    "eyes.dom.indexed",
                    &serde_json::json!({
                        "url": report.url,
                        "elements_indexed": report.elements_indexed,
                    }),
                )
                .await;
            Json(serde_json::json!({ "ok": true, "report": report }))
        }
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}

async fn dom_stats(State(_): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match dom_index::load_stats() {
        Ok(stats) => Json(serde_json::json!({ "ok": true, "stats": stats })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(serde::Deserialize)]
struct DomSearchQuery {
    q: String,
    limit: Option<u32>,
}

async fn dom_search(
    State(_): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<DomSearchQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(20);
    match dom_index::search(&params.q, limit) {
        Ok(hits) => Json(serde_json::json!({ "ok": true, "hits": hits })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}
