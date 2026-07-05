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
use crate::vlm;

pub struct AppState {
    pub config: Config,
    pub spine: SpineClient,
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    tracing::info!("Starting agent-eyes daemon...");

    let spine = SpineClient::new(&config.spine.url, "agent-eyes", env!("CARGO_PKG_VERSION"));
    if let Err(e) = spine.register().await {
        tracing::warn!(error = %e, "Failed to register with agent-spine, continuing without registration");
    }

    let spine_clone = spine.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let _ = spine_clone.heartbeat().await;
        }
    });

    let port = config.server.port;

    let mcp_config = config.clone();
    tokio::spawn(async move {
        crate::mcp_server::EyesMcp::run(mcp_config).await.ok();
    });

    let state = Arc::new(AppState { config, spine });

    let app = Router::new()
        .route("/health", get(health))
        .route("/capture", post(capture_url))
        .route("/diff", post(pixel_diff))
        .route("/dom/index", post(dom_index_url))
        .route("/dom/stats", get(dom_stats))
        .route("/dom/search", get(dom_search))
        .route("/vlm/status", get(vlm_status_handler))
        .route("/vlm/describe", post(vlm_describe))
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
    let url = req.url.clone();
    let max_elements = req.max_elements;
    let payload = crate::dom_diff_coalesce::coalesce_dom_index(&url, || async {
        match dom_index::index_url(&url, max_elements).await {
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
                serde_json::json!({ "ok": true, "report": report })
            }
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }),
        }
    })
    .await;
    Json(payload)
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

async fn vlm_status_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let status = vlm::vlm_status(&state.config.vlm);
    Json(serde_json::json!({ "ok": true, "vlm": status }))
}

#[derive(serde::Deserialize)]
struct VlmDescribeRequest {
    image: String,
    prompt: Option<String>,
}

async fn vlm_describe(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VlmDescribeRequest>,
) -> Json<serde_json::Value> {
    let image_path = PathBuf::from(&req.image);
    match vlm::describe_image(&image_path, req.prompt.as_deref(), &state.config.vlm).await {
        Ok(result) => {
            let _ = state
                .spine
                .publish(
                    "eyes.vlm.described",
                    &serde_json::json!({
                        "image": result.image,
                        "model": result.model,
                        "caption_len": result.caption.len(),
                    }),
                )
                .await;
            Json(serde_json::json!({ "ok": true, "result": result }))
        }
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}
