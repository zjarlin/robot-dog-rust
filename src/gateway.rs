use crate::config::{AppConfig, RunMode};
use crate::planner::{render_ros2_script, render_text, RobotPlan};
use crate::service::generate_plan_with_mode;
use anyhow::{Context, Result};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct GatewayState {
    pub config: AppConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RequestMode {
    Auto,
    Online,
    Offline,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum ResponseFormat {
    Text,
    Json,
    Ros2Script,
}

#[derive(Debug, Deserialize)]
struct PlanRequest {
    prompt: String,
    mode: Option<RequestMode>,
    format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    gateway: &'static str,
    robot_name: String,
    mode: &'static str,
}

#[derive(Debug, Serialize)]
struct PlanResponse {
    mode: String,
    format: String,
    plan: RobotPlan,
    rendered: String,
    ros2_script: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn serve(config: AppConfig, bind: SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(bind)
        .await
        .with_context(|| format!("绑定网关地址失败: {bind}"))?;
    let app = router(config);
    println!("网关已启动: http://{bind}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("网关运行失败")?;
    Ok(())
}

fn router(config: AppConfig) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/plan", post(plan))
        .with_state(GatewayState { config })
}

async fn healthz(State(state): State<GatewayState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        gateway: "robot-dog-edge-gateway",
        robot_name: state.config.robot_name,
        mode: match state.config.mode {
            RunMode::Auto => "auto",
            RunMode::Online => "online",
            RunMode::Offline => "offline",
        },
    })
}

async fn plan(
    State(state): State<GatewayState>,
    Json(request): Json<PlanRequest>,
) -> Result<Json<PlanResponse>, (StatusCode, Json<ErrorResponse>)> {
    let prompt = request.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(bad_request("prompt 不能为空"));
    }

    let mode = request.mode.map(map_request_mode_to_run_mode);
    let plan = generate_plan_with_mode(&state.config, &prompt, mode)
        .await
        .map_err(internal_error)?;

    let request_format = request.format.unwrap_or(ResponseFormat::Json);
    let ros2_script = render_ros2_script(&plan);
    let rendered = match request_format {
        ResponseFormat::Text => render_text(&plan),
        ResponseFormat::Json => {
            serde_json::to_string_pretty(&plan).map_err(|error| internal_error(error.into()))?
        }
        ResponseFormat::Ros2Script => ros2_script.clone(),
    };

    Ok(Json(PlanResponse {
        mode: mode_to_string(mode.unwrap_or(state.config.mode)),
        format: format_to_string(request_format),
        plan,
        rendered,
        ros2_script,
    }))
}

fn map_request_mode_to_run_mode(mode: RequestMode) -> RunMode {
    match mode {
        RequestMode::Auto => RunMode::Auto,
        RequestMode::Online => RunMode::Online,
        RequestMode::Offline => RunMode::Offline,
    }
}

fn mode_to_string(mode: RunMode) -> String {
    match mode {
        RunMode::Auto => "auto",
        RunMode::Online => "online",
        RunMode::Offline => "offline",
    }
    .to_string()
}

fn format_to_string(format: ResponseFormat) -> String {
    match format {
        ResponseFormat::Text => "text",
        ResponseFormat::Json => "json",
        ResponseFormat::Ros2Script => "ros2_script",
    }
    .to_string()
}

fn bad_request(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

fn internal_error(error: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_config() -> AppConfig {
        AppConfig {
            mode: RunMode::Offline,
            api_base_url: "https://api.openai.com".to_string(),
            api_key: None,
            model: "test".to_string(),
            robot_name: "mini_pupper_2".to_string(),
            max_linear_speed: 0.25,
            max_angular_speed: 1.2,
        }
    }

    #[test]
    fn request_mode_to_run_mode_maps_correctly() {
        assert!(matches!(
            map_request_mode_to_run_mode(RequestMode::Online),
            RunMode::Online
        ));
        assert!(matches!(
            map_request_mode_to_run_mode(RequestMode::Offline),
            RunMode::Offline
        ));
    }

    #[test]
    fn format_name_is_stable() {
        assert_eq!(format_to_string(ResponseFormat::Ros2Script), "ros2_script");
    }

    #[tokio::test]
    async fn health_endpoint_returns_gateway_identity() {
        let app = router(test_config());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("robot-dog-edge-gateway"));
    }

    #[tokio::test]
    async fn plan_endpoint_returns_ros2_script_in_offline_mode() {
        let app = router(test_config());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/plan")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"prompt":"站起来然后向前走1秒","mode":"offline","format":"json"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"ros2_script\""));
        assert!(text.contains("/cmd_vel"));
    }

    #[tokio::test]
    async fn plan_endpoint_rejects_empty_prompt() {
        let app = router(test_config());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/plan")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"prompt":"   "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
