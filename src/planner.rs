use crate::config::AppConfig;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RobotPlan {
    pub summary: String,
    pub actions: Vec<RobotAction>,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RobotAction {
    Stand,
    Sit,
    LieDown,
    Stop,
    Wave,
    Dance,
    CameraScan,
    Speak {
        text: String,
    },
    Move {
        linear_x: f32,
        linear_y: f32,
        angular_z: f32,
        duration_ms: u32,
    },
    Turn {
        degrees: f32,
        duration_ms: u32,
    },
}

pub fn offline_plan(prompt: &str, config: &AppConfig) -> RobotPlan {
    let mut actions = Vec::new();
    let text = prompt.trim();
    let lower = text.to_lowercase();

    if contains_any(&lower, &["站", "stand", "起立"]) {
        actions.push(RobotAction::Stand);
    }
    if contains_any(&lower, &["坐", "sit"]) {
        actions.push(RobotAction::Sit);
    }
    if contains_any(&lower, &["趴", "lie", "down"]) {
        actions.push(RobotAction::LieDown);
    }
    if contains_any(&lower, &["停", "stop"]) {
        actions.push(RobotAction::Stop);
    }
    if contains_any(&lower, &["挥手", "wave"]) {
        actions.push(RobotAction::Wave);
    }
    if contains_any(&lower, &["跳舞", "dance"]) {
        actions.push(RobotAction::Dance);
    }
    if contains_any(&lower, &["看", "扫", "camera", "拍照", "巡检"]) {
        actions.push(RobotAction::CameraScan);
    }
    if contains_any(&lower, &["说", "讲", "speak", "voice"]) {
        actions.push(RobotAction::Speak {
            text: text.to_string(),
        });
    }

    if let Some((linear_x, linear_y, angular_z, duration_ms)) = extract_motion(&lower, config) {
        actions.push(RobotAction::Move {
            linear_x,
            linear_y,
            angular_z,
            duration_ms,
        });
    }

    if actions.is_empty() {
        actions.push(RobotAction::Speak {
            text: text.to_string(),
        });
    }

    let safety_notes = vec![
        format!("离线模式只做关键词解析，适合先验证流程，不等于真实运动控制。"),
        format!(
            "建议最大线速度 {:.2} m/s、最大角速度 {:.2} rad/s。",
            config.max_linear_speed, config.max_angular_speed
        ),
    ];

    RobotPlan {
        summary: format!("离线解析完成：{text}"),
        actions,
        safety_notes,
    }
}

pub fn normalize_plan(mut plan: RobotPlan, config: &AppConfig) -> Result<RobotPlan> {
    if plan.actions.is_empty() {
        return Err(anyhow!("动作计划为空"));
    }

    let mut normalized = Vec::with_capacity(plan.actions.len());
    for action in plan.actions {
        let action = match action {
            RobotAction::Move {
                linear_x,
                linear_y,
                angular_z,
                duration_ms,
            } => RobotAction::Move {
                linear_x: clamp(linear_x, -config.max_linear_speed, config.max_linear_speed),
                linear_y: clamp(linear_y, -config.max_linear_speed, config.max_linear_speed),
                angular_z: clamp(
                    angular_z,
                    -config.max_angular_speed,
                    config.max_angular_speed,
                ),
                duration_ms: duration_ms.max(100),
            },
            RobotAction::Turn {
                degrees,
                duration_ms,
            } => RobotAction::Turn {
                degrees: degrees.clamp(-180.0, 180.0),
                duration_ms: duration_ms.max(100),
            },
            other => other,
        };
        normalized.push(action);
    }

    plan.actions = normalized;
    if plan.summary.trim().is_empty() {
        plan.summary = "动作计划".to_string();
    }
    if plan.safety_notes.is_empty() {
        plan.safety_notes.push("无额外安全提示".to_string());
    }
    Ok(plan)
}

pub fn render_text(plan: &RobotPlan) -> String {
    let mut out = String::new();
    out.push_str(&format!("计划：{}\n", plan.summary));
    out.push_str("动作：\n");
    for (index, action) in plan.actions.iter().enumerate() {
        out.push_str(&format!("  {}. {}\n", index + 1, render_action(action)));
    }
    if !plan.safety_notes.is_empty() {
        out.push_str("安全提示：\n");
        for note in &plan.safety_notes {
            out.push_str(&format!("  - {}\n", note));
        }
    }
    out
}

pub fn render_ros2_script(plan: &RobotPlan) -> String {
    let mut out = String::new();
    out.push_str("#!/usr/bin/env bash\n");
    out.push_str("set -euo pipefail\n\n");
    out.push_str("# 这个脚本是计划草稿。具体姿态控制话题需要按你的机器狗型号再适配。\n");
    out.push_str("# 其中 Move 会映射到 /cmd_vel 示例。\n\n");
    for action in &plan.actions {
        match action {
            RobotAction::Move {
                linear_x,
                linear_y,
                angular_z,
                duration_ms,
            } => {
                out.push_str(&format!(
                    "ros2 topic pub --once /cmd_vel geometry_msgs/msg/Twist '{{linear: {{x: {:.3}, y: {:.3}, z: 0.0}}, angular: {{x: 0.0, y: 0.0, z: {:.3}}}}}'\n",
                    linear_x, linear_y, angular_z
                ));
                out.push_str(&format!("sleep {:.3}\n", *duration_ms as f32 / 1000.0));
            }
            RobotAction::Turn {
                degrees,
                duration_ms,
            } => {
                out.push_str(&format!(
                    "# 转向 {:.1} 度，持续 {} ms。这里需要接到具体姿态节点。\n",
                    degrees, duration_ms
                ));
            }
            other => {
                out.push_str(&format!("# {}\n", render_action(other)));
            }
        }
    }
    out
}

fn render_action(action: &RobotAction) -> String {
    match action {
        RobotAction::Stand => "站立".to_string(),
        RobotAction::Sit => "坐下".to_string(),
        RobotAction::LieDown => "趴下".to_string(),
        RobotAction::Stop => "停止".to_string(),
        RobotAction::Wave => "挥手".to_string(),
        RobotAction::Dance => "跳舞".to_string(),
        RobotAction::CameraScan => "摄像头巡检".to_string(),
        RobotAction::Speak { text } => format!("播报：{}", text),
        RobotAction::Move {
            linear_x,
            linear_y,
            angular_z,
            duration_ms,
        } => format!(
            "移动：linear_x={:.3}, linear_y={:.3}, angular_z={:.3}, duration_ms={}",
            linear_x, linear_y, angular_z, duration_ms
        ),
        RobotAction::Turn {
            degrees,
            duration_ms,
        } => format!("转向：degrees={:.1}, duration_ms={}", degrees, duration_ms),
    }
}

fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| text.contains(pattern))
}

fn extract_motion(lower_prompt: &str, config: &AppConfig) -> Option<(f32, f32, f32, u32)> {
    if !(lower_prompt.contains("前进")
        || lower_prompt.contains("后退")
        || lower_prompt.contains("左转")
        || lower_prompt.contains("右转")
        || lower_prompt.contains("move")
        || lower_prompt.contains("turn")
        || lower_prompt.contains("走"))
    {
        return None;
    }

    let mut linear_x = 0.0;
    let linear_y = 0.0;
    let mut angular_z = 0.0;
    let mut duration_ms = 1500;

    if lower_prompt.contains("前进") || lower_prompt.contains("forward") {
        linear_x = config.max_linear_speed.min(0.15);
    }
    if lower_prompt.contains("后退") || lower_prompt.contains("back") {
        linear_x = -config.max_linear_speed.min(0.15);
    }
    if lower_prompt.contains("左转") || lower_prompt.contains("left") {
        angular_z = config.max_angular_speed.min(0.6);
    }
    if lower_prompt.contains("右转") || lower_prompt.contains("right") {
        angular_z = -config.max_angular_speed.min(0.6);
    }

    if let Some(ms) = parse_duration_ms(lower_prompt) {
        duration_ms = ms;
    }

    Some((linear_x, linear_y, angular_z, duration_ms))
}

fn parse_duration_ms(text: &str) -> Option<u32> {
    let mut number = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            number.push(ch);
        } else if !number.is_empty() {
            break;
        }
    }

    number.parse::<u32>().ok().map(|value| {
        if text.contains("秒") || text.contains("sec") {
            value.saturating_mul(1000)
        } else {
            value
        }
    })
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, RunMode};

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
    fn offline_plan_detects_basic_actions() {
        let plan = offline_plan("让机器人站起来然后向前走2秒再停下", &test_config());
        assert!(plan.actions.iter().any(|a| matches!(a, RobotAction::Stand)));
        assert!(plan
            .actions
            .iter()
            .any(|a| matches!(a, RobotAction::Move { .. })));
        assert!(plan.actions.iter().any(|a| matches!(a, RobotAction::Stop)));
    }

    #[test]
    fn normalize_plan_clamps_speed() {
        let plan = RobotPlan {
            summary: "".to_string(),
            actions: vec![RobotAction::Move {
                linear_x: 9.0,
                linear_y: -9.0,
                angular_z: 9.0,
                duration_ms: 10,
            }],
            safety_notes: vec![],
        };
        let normalized = normalize_plan(plan, &test_config()).unwrap();
        match &normalized.actions[0] {
            RobotAction::Move {
                linear_x,
                linear_y,
                angular_z,
                duration_ms,
            } => {
                assert_eq!(*linear_x, 0.25);
                assert_eq!(*linear_y, -0.25);
                assert_eq!(*angular_z, 1.2);
                assert_eq!(*duration_ms, 100);
            }
            other => panic!("unexpected action: {other:?}"),
        }
        assert!(!normalized.summary.is_empty());
        assert!(!normalized.safety_notes.is_empty());
    }
}
