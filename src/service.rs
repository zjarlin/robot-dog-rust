use crate::config::{AppConfig, RunMode};
use crate::llm;
use crate::planner::{normalize_plan, offline_plan, RobotPlan};
use anyhow::Result;

pub fn should_use_online(config: &AppConfig) -> bool {
    match config.mode {
        RunMode::Offline => false,
        RunMode::Online => true,
        RunMode::Auto => config.can_use_online_model(),
    }
}

pub async fn generate_plan(config: &AppConfig, prompt: &str) -> Result<RobotPlan> {
    let mut plan = if should_use_online(config) {
        match llm::request_plan(config, prompt).await {
            Ok(plan) => plan,
            Err(error) => {
                eprintln!("在线模式失败，回退到离线模式：{error:#}");
                offline_plan(prompt, config)
            }
        }
    } else {
        offline_plan(prompt, config)
    };

    plan = normalize_plan(plan, config)?;
    Ok(plan)
}

pub async fn generate_plan_with_mode(
    config: &AppConfig,
    prompt: &str,
    mode: Option<RunMode>,
) -> Result<RobotPlan> {
    let mut config = config.clone();
    if let Some(mode) = mode {
        config.mode = mode;
    }
    generate_plan(&config, prompt).await
}
