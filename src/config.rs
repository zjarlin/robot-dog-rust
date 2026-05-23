use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mode: RunMode,
    pub api_base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub robot_name: String,
    pub max_linear_speed: f32,
    pub max_angular_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Auto,
    Online,
    Offline,
}

impl AppConfig {
    pub fn from_env(mode_override: Option<RunMode>) -> Result<Self> {
        let env_mode = match std::env::var("ROBOT_DOG_MODE")
            .unwrap_or_else(|_| "auto".to_string())
            .to_lowercase()
            .as_str()
        {
            "online" => RunMode::Online,
            "offline" => RunMode::Offline,
            _ => RunMode::Auto,
        };

        let mode = mode_override.unwrap_or(env_mode);
        let api_base_url = std::env::var("ROBOT_DOG_OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string());
        let api_key = std::env::var("ROBOT_DOG_OPENAI_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let model =
            std::env::var("ROBOT_DOG_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        let robot_name =
            std::env::var("ROBOT_DOG_ROBOT_NAME").unwrap_or_else(|_| "mini_pupper_2".to_string());
        let max_linear_speed = parse_env_f32("ROBOT_DOG_MAX_LINEAR_SPEED", 0.25)?;
        let max_angular_speed = parse_env_f32("ROBOT_DOG_MAX_ANGULAR_SPEED", 1.2)?;

        Ok(Self {
            mode,
            api_base_url,
            api_key,
            model,
            robot_name,
            max_linear_speed,
            max_angular_speed,
        })
    }

    pub fn can_use_online_model(&self) -> bool {
        self.api_key.is_some()
    }
}

fn parse_env_f32(name: &str, default: f32) -> Result<f32> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<f32>()
            .with_context(|| format!("环境变量 {name} 不是合法数字: {value}")),
        Err(_) => Ok(default),
    }
}
