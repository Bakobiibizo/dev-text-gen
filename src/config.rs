use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,
    pub ollama_url: String,
    pub model_name: String,
    pub preload: bool,
    pub api_token: Option<String>,
    pub max_body_bytes: usize,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let api_host = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let api_port = parse_env("API_PORT", 7103)?;
        let ollama_url = env::var("OLLAMA_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string())
            .trim_end_matches('/')
            .to_string();
        let model_name = env::var("MODEL_NAME").unwrap_or_else(|_| "gemma3:1b".to_string());
        let preload = parse_bool("PRELOAD", false)?;
        let api_token = env::var("API_TOKEN").ok().filter(|value| !value.is_empty());
        let max_body_bytes = parse_env("MAX_BODY_BYTES", 10 * 1024 * 1024)?;
        Ok(Self {
            api_host,
            api_port,
            ollama_url,
            model_name,
            preload,
            api_token,
            max_body_bytes,
        })
    }
}

fn parse_env<T>(name: &str, default: T) -> Result<T, String>
where
    T: std::str::FromStr,
{
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| format!("invalid {name}: {value}")),
        Err(_) => Ok(default),
    }
}

fn parse_bool(name: &str, default: bool) -> Result<bool, String> {
    match env::var(name) {
        Ok(value) if value == "1" || value.eq_ignore_ascii_case("true") => Ok(true),
        Ok(value) if value == "0" || value.eq_ignore_ascii_case("false") => Ok(false),
        Ok(value) => Err(format!("invalid {name}: {value}")),
        Err(_) => Ok(default),
    }
}
