mod config;
mod driver;
mod llm;
mod planner;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use config::{AppConfig, RunMode};
use planner::offline_plan;
use std::io::{self, IsTerminal, Read};

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Ros2Script,
}

#[derive(Debug, Clone, ValueEnum)]
enum ModeArg {
    Auto,
    Online,
    Offline,
}

#[derive(Parser, Debug)]
#[command(
    name = "robot-dog-rust",
    version,
    about = "Rust 四足机器狗大模型控制器"
)]
struct Cli {
    #[arg(short, long)]
    prompt: Option<String>,
    #[arg(long, value_enum, default_value_t = ModeArg::Auto)]
    mode: ModeArg,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let mode_override = Some(match cli.mode {
        ModeArg::Auto => RunMode::Auto,
        ModeArg::Online => RunMode::Online,
        ModeArg::Offline => RunMode::Offline,
    });
    let config = AppConfig::from_env(mode_override)?;
    let prompt = read_prompt(cli.prompt)?;
    let use_online = should_use_online(&config);

    let mut plan = if use_online {
        match llm::request_plan(&config, &prompt).await {
            Ok(plan) => plan,
            Err(error) => {
                eprintln!("在线模式失败，回退到离线模式：{error:#}");
                offline_plan(&prompt, &config)
            }
        }
    } else {
        offline_plan(&prompt, &config)
    };

    plan = planner::normalize_plan(plan, &config)?;

    let output = match cli.format {
        OutputFormat::Text => driver::render_output(&plan, false),
        OutputFormat::Json => serde_json::to_string_pretty(&plan)?,
        OutputFormat::Ros2Script => driver::render_output(&plan, true),
    };
    println!("{output}");
    Ok(())
}

fn read_prompt(prompt: Option<String>) -> Result<String> {
    if let Some(prompt) = prompt {
        let text = prompt.trim().to_string();
        if text.is_empty() {
            return Err(anyhow!("--prompt 不能为空"));
        }
        return Ok(text);
    }

    if io::stdin().is_terminal() {
        return Err(anyhow!("请使用 --prompt 参数，或通过标准输入提供任务"));
    }

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("读取标准输入失败")?;
    let text = buffer.trim().to_string();
    if text.is_empty() {
        return Err(anyhow!("没有读取到有效指令"));
    }
    Ok(text)
}

fn should_use_online(config: &AppConfig) -> bool {
    match config.mode {
        RunMode::Offline => false,
        RunMode::Online => true,
        RunMode::Auto => config.can_use_online_model(),
    }
}
