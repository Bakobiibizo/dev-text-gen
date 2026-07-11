use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::TryStreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{error, info};

mod config;

#[derive(Clone)]
struct AppState {
    config: Arc<config::Config>,
    client: Client,
}

#[derive(Deserialize)]
struct GenerateRequest {
    prompt: String,
    model: Option<String>,
    stream: Option<bool>,
}

#[derive(Serialize)]
struct StatusResponse {
    ready: bool,
    backend: bool,
    model: String,
}

fn app(config: config::Config) -> Router {
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(600))
        .build()
        .expect("valid HTTP client configuration");
    let state = Arc::new(AppState {
        config: Arc::new(config),
        client,
    });
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/pull", post(pull))
        .route("/generate", post(generate))
        .route("/v1/models", get(v1_models))
        .route("/v1/chat/completions", post(v1_chat_completions))
        .route("/v1/completions", post(v1_completions))
        .route("/v1/embeddings", post(v1_embeddings))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let url = format!("{}/api/tags", state.config.ollama_url);
    let (backend, model_ready) = match state.client.get(url).send().await {
        Ok(response) if response.status().is_success() => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let found = body["models"].as_array().is_some_and(|models| {
                models.iter().any(|model| {
                    model["name"].as_str() == Some(&state.config.model_name)
                        || model["model"].as_str() == Some(&state.config.model_name)
                })
            });
            (true, found)
        }
        _ => (false, false),
    };
    let status = StatusResponse {
        ready: backend && model_ready,
        backend,
        model: state.config.model_name.clone(),
    };
    let code = if status.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(status))
}

async fn pull(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if !authorized(&state.config, &headers) {
        return unauthorized();
    }
    let url = format!("{}/api/pull", state.config.ollama_url);
    let body = serde_json::json!({"model": state.config.model_name, "stream": false});
    forward_response(state.client.post(url).json(&body).send().await).await
}

async fn generate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<GenerateRequest>,
) -> Response {
    if !authorized(&state.config, &headers) {
        return unauthorized();
    }
    if request.prompt.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "prompt must not be empty").into_response();
    }
    let url = format!("{}/api/generate", state.config.ollama_url);
    let body = serde_json::json!({
        "model": request.model.unwrap_or_else(|| state.config.model_name.clone()),
        "prompt": request.prompt,
        "stream": request.stream.unwrap_or(true),
    });
    forward_response(state.client.post(url).json(&body).send().await).await
}

async fn v1_models(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if !authorized(&state.config, &headers) {
        return unauthorized();
    }
    let url = format!("{}/v1/models", state.config.ollama_url);
    forward_response(state.client.get(url).send().await).await
}

async fn v1_chat_completions(state: State<Arc<AppState>>, request: Request<Body>) -> Response {
    forward_v1_request(state.0, "v1/chat/completions", request).await
}

async fn v1_completions(state: State<Arc<AppState>>, request: Request<Body>) -> Response {
    forward_v1_request(state.0, "v1/completions", request).await
}

async fn v1_embeddings(state: State<Arc<AppState>>, request: Request<Body>) -> Response {
    forward_v1_request(state.0, "v1/embeddings", request).await
}

async fn forward_v1_request(
    state: Arc<AppState>,
    request_path: &str,
    request: Request<Body>,
) -> Response {
    if !authorized(&state.config, request.headers()) {
        return unauthorized();
    }
    let bytes = match axum::body::to_bytes(request.into_body(), state.config.max_body_bytes).await {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::PAYLOAD_TOO_LARGE, "request body too large").into_response(),
    };
    let url = format!("{}/{}", state.config.ollama_url, request_path);
    forward_response(
        state
            .client
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(bytes)
            .send()
            .await,
    )
    .await
}

async fn forward_response(result: Result<reqwest::Response, reqwest::Error>) -> Response {
    match result {
        Ok(upstream) => {
            let status = upstream.status();
            let content_type = upstream.headers().get(header::CONTENT_TYPE).cloned();
            let stream = upstream
                .bytes_stream()
                .map_err(|error| -> axum::BoxError { Box::new(error) });
            let mut response = (status, Body::from_stream(stream)).into_response();
            if let Some(value) = content_type {
                response.headers_mut().insert(header::CONTENT_TYPE, value);
            }
            response
        }
        Err(error) => {
            error!(%error, "upstream request failed");
            (StatusCode::BAD_GATEWAY, "upstream unavailable").into_response()
        }
    }
}

fn authorized(config: &config::Config, headers: &HeaderMap) -> bool {
    let Some(expected) = &config.api_token else {
        return true;
    };
    let supplied = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    supplied == Some(expected)
}

fn unauthorized() -> Response {
    (StatusCode::UNAUTHORIZED, "missing or invalid bearer token").into_response()
}

async fn preload(config: &config::Config, client: &Client) {
    if !config.preload {
        return;
    }
    let url = format!("{}/api/pull", config.ollama_url);
    let body = serde_json::json!({"model": config.model_name, "stream": false});
    match client.post(url).json(&body).send().await {
        Ok(response) if response.status().is_success() => {
            info!(model = %config.model_name, "model ready")
        }
        Ok(response) => error!(status = %response.status(), "model preload failed"),
        Err(error) => error!(%error, "model preload failed"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    let config = config::Config::load().map_err(|error| format!("configuration error: {error}"))?;
    let address: SocketAddr = format!("{}:{}", config.api_host, config.api_port).parse()?;
    let preload_config = config.clone();
    let preload_client = Client::new();
    tokio::spawn(async move { preload(&preload_config, &preload_client).await });
    info!(%address, upstream = %config.ollama_url, model = %config.model_name, "starting text proxy");
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, app(config)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, routing::post, Router};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn test_app(token: Option<&str>) -> Router {
        let upstream = Router::new()
            .route(
                "/api/tags",
                get(|| async { Json(serde_json::json!({"models": [{"name": "test:latest"}]})) }),
            )
            .route(
                "/api/generate",
                post(|| async {
                    (
                        [(header::CONTENT_TYPE, "application/x-ndjson")],
                        "{\"response\":\"ok\"}\n",
                    )
                }),
            )
            .route(
                "/v1/chat/completions",
                post(|| async {
                    (
                        [(header::CONTENT_TYPE, "text/event-stream")],
                        "data: done\n\n",
                    )
                }),
            );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, upstream).await.unwrap() });
        app(config::Config {
            api_host: "127.0.0.1".to_string(),
            api_port: 0,
            ollama_url: format!("http://{address}"),
            model_name: "test:latest".to_string(),
            preload: false,
            api_token: token.map(str::to_string),
            max_body_bytes: 1024,
        })
    }

    async fn body(response: Response) -> String {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn health_and_readiness_reflect_backend() {
        let app = test_app(None).await;
        let health = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::OK);
        let ready = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ready.status(), StatusCode::OK);
        assert!(body(ready).await.contains("\"ready\":true"));
    }

    #[tokio::test]
    async fn protected_routes_require_bearer_token() {
        let app = test_app(Some("secret")).await;
        let unauthorized = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/generate")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"prompt":"hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
        let authorized = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/generate")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, "Bearer secret")
                    .body(Body::from(r#"{"prompt":"hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(authorized.status(), StatusCode::OK);
        assert_eq!(
            authorized.headers()[header::CONTENT_TYPE],
            "application/x-ndjson"
        );
    }

    #[tokio::test]
    async fn openai_stream_is_forwarded_without_buffering_contract_loss() {
        let app = test_app(None).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/event-stream"
        );
        assert_eq!(body(response).await, "data: done\n\n");
    }
}
