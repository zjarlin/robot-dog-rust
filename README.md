# robot-dog-rust

这是一个面向 `Mini Pupper 2` 这类四足机器狗的 Rust 控制仓库，目标很直接：

1. 先把采购清单定清楚，尽量选中国淘宝可以买到、中文资料多、上手快的方案。
2. 再把大模型控制层做成一个可直接配置 API key 的 Rust 工具。
3. 最后再把它接到 ROS2 或具体硬件驱动上，而不是一开始就重写底层步态。

## 这个仓库现在有什么

- `docs/procurement.md`：采购清单，按“能买到、能上手、能扩展”来排。
- `docs/software_plan.md`：软件方案，说明为什么先做 Rust 控制层，再接 ROS2。
- `docs/ros2_bridge.md`：后续如何把动作计划接到 ROS2。
- `src/`：Rust 控制程序，支持在线大模型和离线规则模式。
- `src/gateway.rs`：边缘计算网关 HTTP 服务，直接对外提供计划生成接口。

## 快速开始

```bash
cp .env.example .env
cargo run -- --prompt "让机器人站起来，然后向前走两秒，再停下"
```

如果你已经配置了 API key，就走在线模式：

```bash
export ROBOT_DOG_OPENAI_API_KEY="你的key"
cargo run -- --prompt "向左转 30 度，然后坐下"
```

如果暂时没有 API key，也能先跑离线模式：

```bash
cargo run -- --mode offline --prompt "站起来并挥手"
```

## 推荐硬件方向

首选 `Mini Pupper 2`。它的官方资料、ROS2 指南、快速上手和 AI 入口都比较完整，而且官方明确给了中国淘宝购买入口。

如果你想把系统拆成“边缘网关 + 本体控制板”两层，优先顺序是：

1. `Jetson Orin Nano Super Developer Kit` 做边缘计算网关。
2. `Raspberry Pi Compute Module 5 + Compute Module 5 IO Board` 做核心板与底板开发套件。
3. `Firefly Core-3588J + ITX-3588J 底板` 做国内供应链更友好的 RK3588 备选。

具体采购清单、淘宝搜索词和资料入口在 `docs/procurement.md`。

## 边缘网关模式

边缘网关适合跑在 `Jetson Orin Nano`、`RK3588`、`CM5` 或普通 Ubuntu 迷你主机上。本体控制板只负责 ROS2 和硬件动作，网关负责把自然语言、大模型、视觉/语音服务和 ROS2 动作计划隔离开。

启动 HTTP 服务：

```bash
export ROBOT_DOG_OPENAI_API_KEY="你的key"
cargo run -- --serve --bind 0.0.0.0:8080
```

调用计划生成接口：

```bash
curl http://127.0.0.1:8080/v1/plan \
  -H 'content-type: application/json' \
  -d '{"prompt":"站起来，然后向前走一秒，再停下","mode":"auto","format":"json"}'
```

不填 API key 时会走离线规则模式，适合先验证链路；填入 `ROBOT_DOG_OPENAI_API_KEY` 后走 OpenAI 兼容接口。

## 代码定位

这个仓库先把“大模型把自然语言变成安全动作计划”这件事做稳。后面接真机器时，把 `RobotPlan` 映射到：

- ROS2 的 `/cmd_vel`
- 机器人姿态话题
- 语音播报
- 摄像头巡检

现在已经额外提供了一个边缘网关模式，可以直接用 `--serve` 起 HTTP 服务，对外暴露 `/healthz` 和 `/v1/plan`。

这样不会把业务逻辑绑死在某个硬件 SDK 上。
