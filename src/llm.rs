use crate::config::AppConfig;
use crate::planner::{normalize_plan, RobotPlan};
use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    temperature: f32,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

pub async fn request_plan(config: &AppConfig, prompt: &str) -> Result<RobotPlan> {
    let client = reqwest::Client::new();
    let system_prompt = build_system_prompt(config);
    let request = ChatCompletionRequest {
        model: &config.model,
        temperature: 0.2,
        messages: vec![
            ChatMessage {
                role: "system",
                content: &system_prompt,
            },
            ChatMessage {
                role: "user",
                content: prompt,
            },
        ],
    };

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let api_key = config
        .api_key
        .as_ref()
        .context("在线模式需要配置 ROBOT_DOG_OPENAI_API_KEY")?;
    let bearer = format!("Bearer {api_key}");
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&bearer).context("API key 含有非法字符")?,
    );

    let url = format!(
        "{}/v1/chat/completions",
        config.api_base_url.trim_end_matches('/')
    );
    let response = client
        .post(url)
        .headers(headers)
        .json(&request)
        .send()
        .await
        .context("调用大模型接口失败")?
        .error_for_status()
        .context("大模型接口返回非成功状态码")?;

    let payload: ChatCompletionResponse = response.json().await.context("解析大模型响应失败")?;
    let content = payload
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .cloned()
        .ok_or_else(|| anyhow!("大模型没有返回内容"))?;

    let json_text = extract_json_payload(&content)?;
    let plan: RobotPlan = serde_json::from_str(json_text)
        .with_context(|| format!("无法解析大模型返回的 JSON: {json_text}"))?;
    normalize_plan(plan, config)
}

fn build_system_prompt(config: &AppConfig) -> String {
    format!(
        r#"
你是一个四足机器狗控制计划生成器。目标是把用户自然语言转换成严格 JSON。

约束：
- 只能返回 JSON，不能返回 Markdown、注释、解释文字。
- 不要输出空对象。
- 动作必须来自允许集合。
- 速度必须保守。
- 机器人名称是 `{robot_name}`。

允许的 JSON 结构：
{{
  "summary": "一句中文摘要",
  "actions": [
    {{"type":"stand"}},
    {{"type":"sit"}},
    {{"type":"lie_down"}},
    {{"type":"stop"}},
    {{"type":"wave"}},
    {{"type":"dance"}},
    {{"type":"camera_scan"}},
    {{"type":"speak","text":"要播报的话"}},
    {{"type":"move","linear_x":0.1,"linear_y":0.0,"angular_z":0.0,"duration_ms":1500}},
    {{"type":"turn","degrees":30,"duration_ms":1200}}
  ],
  "safety_notes": ["需要人工确认的注意事项"]
}}

安全规则：
- 线速度绝对值不要超过 {max_linear:.2}。
- 角速度绝对值不要超过 {max_angular:.2}。
- 不确定时优先输出 `stop` 或 `speak`。
- 如果用户请求危险动作，转换成安全近似动作。
"#,
        robot_name = config.robot_name,
        max_linear = config.max_linear_speed,
        max_angular = config.max_angular_speed
    )
}

pub fn extract_json_payload(content: &str) -> Result<&str> {
    let trimmed = content.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Ok(trimmed);
    }

    let start = trimmed
        .find('{')
        .ok_or_else(|| anyhow!("响应里找不到 JSON 起始符号"))?;
    let end = trimmed
        .rfind('}')
        .ok_or_else(|| anyhow!("响应里找不到 JSON 结束符号"))?;

    if end <= start {
        return Err(anyhow!("响应中的 JSON 区域不合法"));
    }

    Ok(&trimmed[start..=end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_json_from_code_fence() {
        let raw = "```json\n{\"summary\":\"ok\",\"actions\":[],\"safety_notes\":[]}\n```";
        let json = extract_json_payload(raw).unwrap();
        assert_eq!(
            json,
            "{\"summary\":\"ok\",\"actions\":[],\"safety_notes\":[]}"
        );
    }
}
