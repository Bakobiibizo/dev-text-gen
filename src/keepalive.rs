use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tracing::{error, info};

use crate::config::Config;

/// Periodic ping to keep Ollama backend/model warm.
pub async fn ping_loop(cfg: Arc<Config>, client: Client) {
    let url = format!("{}/api/tags", cfg.ollama_url);
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        match client.head(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                info!(target = "keepalive", "ping ok: {}", resp.status());
            }
            Ok(resp) => {
                error!(target = "keepalive", status = %resp.status(), "keepalive ping failed");
            }
            Err(err) => {
                error!(target = "keepalive", error = %err, "keepalive ping error");
            }
        }
    }
}
