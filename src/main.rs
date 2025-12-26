use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

mod backend;
mod config;

#[derive(Clone)]
struct AppState {
    config: Arc<config::Config>,
    client: Client,
    ready: Arc<RwLock<bool>>,
}

#[derive(Deserialize)]
struct GenerateRequest {
    prompt: String,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Serialize)]
struct ReadyStatus {
    ready: bool,
}

async fn health() -> &'static str {
    "ok"
}

async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ready = *state.ready.read().await;
    (StatusCode::OK, Json(ReadyStatus { ready }))
}

async fn pull(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match ollama_pull(&state.client, &state.config.ollama_url, &state.config.model_name).await {
        Ok(_) => {
            *state.ready.write().await = true;
            (StatusCode::OK, "pulled")
        }
        Err(err) => {
            error!("pull failed: {}", err);
            (StatusCode::BAD_GATEWAY, "pull failed")
        }
    }
}

async fn generate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> impl IntoResponse {
    let model = req.model.unwrap_or_else(|| state.config.model_name.clone());
    let body = serde_json::json!({
        "model": model,
        "prompt": req.prompt,
        "stream": false
    });
    let url = format!("{}/api/generate", state.config.ollama_url);
    match state.client.post(&url).json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(text) => (status, text),
                Err(err) => {
                    error!("generate read error: {}", err);
                    (StatusCode::BAD_GATEWAY, "upstream read error".to_string())
                }
            }
        }
        Err(err) => {
            error!("generate upstream error: {}", err);
            (StatusCode::BAD_GATEWAY, "upstream error".to_string())
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cfg = Arc::new(config::Config::load());
    let client = Client::new();

    // Ensure backend subprocess is running
    if let Err(e) = backend::ensure_backend_running(&cfg.backend, &client).await {
        error!("failed to start backend: {}", e);
    }

    let state = Arc::new(AppState {
        config: cfg.clone(),
        client: client.clone(),
        ready: Arc::new(RwLock::new(false)),
    });

    // Start health-check loop for backend subprocess
    let health_cfg = cfg.backend.clone();
    let health_client = client.clone();
    tokio::spawn(async move {
        backend::health_check_loop(health_cfg, health_client).await;
    });

    // Preload (pull) target model
    let preload_state = state.clone();
    tokio::spawn(async move {
        if preload_state.config.preload {
            // Wait a bit for container to be ready
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            match ollama_pull(
                &preload_state.client,
                &preload_state.config.ollama_url,
                &preload_state.config.model_name,
            )
            .await
            {
                Ok(_) => {
                    *preload_state.ready.write().await = true;
                    info!("model pulled and ready");
                }
                Err(err) => error!("preload pull failed: {}", err),
            }
        } else {
            *preload_state.ready.write().await = true;
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/pull", post(pull))
        .route("/generate", post(generate))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    let addr: SocketAddr = format!("{}:{}", state.config.api_host, state.config.api_port)
        .parse()
        .unwrap_or_else(|e| {
            error!("Invalid bind address: {}", e);
            SocketAddr::from(([0, 0, 0, 0], 7103))
        });
    info!(
        "listening on {} and proxying to {} for model {}",
        addr, state.config.ollama_url, state.config.model_name
    );
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ollama_pull(client: &Client, base_url: &str, model: &str) -> Result<(), String> {
    let url = format!("{}/api/pull", base_url);
    let body = serde_json::json!({
        "model": model,
        "stream": false
    });
    match client.post(url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(_) => Err("pull failed".to_string()),
        Err(e) => Err(e.to_string()),
    }
}
