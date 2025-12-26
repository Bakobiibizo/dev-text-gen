use std::env;

use crate::backend::BackendConfig;

pub struct Config {
    pub api_host: String,
    pub api_port: u16,
    pub ollama_url: String,
    pub model_name: String,
    pub preload: bool,
    pub backend: BackendConfig,
}

impl Config {
    pub fn load() -> Self {
        let api_host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let api_port = env::var("API_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7103);
        let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model_name = env::var("MODEL_NAME").unwrap_or_else(|_| "gemma3:27b".to_string());
        let preload = env::var("PRELOAD")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        let backend = BackendConfig::from_env(11434);  // Ollama's default port
        Self {
            api_host,
            api_port,
            ollama_url,
            model_name,
            preload,
            backend,
        }
    }
}
