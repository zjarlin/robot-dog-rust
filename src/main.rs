mod config;
mod driver;
mod gateway;
mod llm;
mod planner;
mod service;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use config::{AppConfig, RunMode};
use service::generate_plan;
use std::io::{self, IsTerminal, Read};
use std::net::SocketAddr;

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
    #[arg(long)]
    serve: bool,
    #[arg(long)]
    bind: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    if cli.serve && cli.prompt.is_some() {
        return Err(anyhow!("--serve 运行时不要同时传 --prompt"));
    }
    let mode_override = Some(match cli.mode {
        ModeArg::Auto => RunMode::Auto,
        ModeArg::Online => RunMode::Online,
        ModeArg::Offline => RunMode::Offline,
    });
    let config = AppConfig::from_env(mode_override)?;
    if cli.serve {
        let bind = parse_bind(cli.bind)?;
        return gateway::serve(config, bind).await;
    }
    let prompt = read_prompt(cli.prompt)?;
    let plan = generate_plan(&config, &prompt).await?;

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

fn parse_bind(bind: Option<String>) -> Result<SocketAddr> {
    let bind = bind
        .or_else(|| std::env::var("ROBOT_DOG_GATEWAY_BIND").ok())
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());
    bind.parse::<SocketAddr>()
        .with_context(|| format!("无法解析网关地址: {bind}"))
}
