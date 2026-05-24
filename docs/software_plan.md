# 软件方案

## 总体思路

这个项目不从“重写机器狗控制器”开始，而是从“Rust 大模型控制层”开始。

原因很简单：

1. 底层步态和硬件驱动已经有成熟开源栈。
2. Rust 更适合做安全的编排层、状态机、协议层和大模型网关。
3. 直接用现成 ROS2 栈，能更快把机器动起来。

## 分层

### 1. 采购和硬件层

- `Mini Pupper 2`
- `Jetson Orin Nano` 边缘网关
- `Raspberry Pi CM5 + IO Board` 核心控制板
- `RK3588 + 底板` 成套开发板备选
- 摄像头
- 激光雷达
- 控制电脑

### 2. 机器人驱动层

- 继续使用官方或成熟社区的 ROS2 栈
- 这层不要用 Rust 重写

### 3. Rust 控制层

本仓库的 Rust 程序负责：

- 接收自然语言
- 调用 OpenAI 兼容接口
- 生成结构化动作计划
- 做安全检查和速度限制
- 输出 JSON 或 ROS2 可读计划
- 给边缘网关提供统一 HTTP 接口
- 给本体控制板提供上层任务入口

### 4. 执行层

后续可以把动作计划接到：

- 边缘算力网关上的视觉与语音服务
- 核心控制板上的本体 IO 与状态机
- `/cmd_vel`
- 姿态控制话题
- 语音播报
- 视觉巡检流程

## 边缘计算网关方案

边缘网关是这个方案里新增的一层，不替代机器狗本体控制板。它的职责是把大模型、视觉、语音、ROS2 上层编排和具体舵机/步态控制隔离开。

推荐部署拓扑：

```text
手机/电脑/语音入口
  -> robot-dog-rust 边缘网关 HTTP API
  -> RobotPlan JSON
  -> ROS2 桥接节点
  -> Mini Pupper 2 官方 ROS2/底层控制
```

网关运行位置：

- 首选：`Jetson Orin Nano Super Developer Kit`，负责视觉、语音、本地小模型和 `/v1/plan`。
- 低功耗控制：`Raspberry Pi CM5 + CM5 IO Board`，负责本体侧 ROS2、中继和 IO。
- 国产算力备选：`Firefly Core-3588J + ITX-3588J`，负责视觉、视频输入输出和国产供应链验证。

当前仓库已经提供最小可运行网关代码：

- `src/gateway.rs`：Axum HTTP 服务。
- `src/service.rs`：在线大模型和离线规则模式的统一计划生成入口。
- `src/main.rs`：`--serve` 和 `--bind` 命令行入口。

启动方式：

```bash
cargo run -- --serve --bind 0.0.0.0:8080
```

请求示例：

```bash
curl http://127.0.0.1:8080/v1/plan \
  -H 'content-type: application/json' \
  -d '{"prompt":"站起来，然后向前走一秒，再停下","mode":"auto","format":"json"}'
```

配置大模型：

```bash
export ROBOT_DOG_OPENAI_BASE_URL="https://api.openai.com"
export ROBOT_DOG_OPENAI_API_KEY="你的key"
export ROBOT_DOG_OPENAI_MODEL="gpt-4o-mini"
```

如果没有 API key，`auto` 模式会自动走离线规则解析，这样硬件和 ROS2 链路可以先调通。

## 为什么这样做

- 先可用，再可控。
- 先让人能填 API key 跑起来，再考虑复杂自治。
- 先把动作计划做成结构化数据，后面才能稳地接不同硬件。

## 第一版能力

- `站起来`
- `坐下`
- `趴下`
- `前进/后退/左转/右转`
- `停下`
- `说一句话`

## 第二版能力

- 巡逻
- 摄像头巡检
- 基础视觉问答
- 语音交互

## 第三版能力

- ROS2 事件驱动
- 导航
- 多模态感知
- 多设备联动
- 边缘网关 + 本体控制板分层部署
