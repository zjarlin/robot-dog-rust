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
cargo run -- --offline --prompt "站起来并挥手"
```

## 推荐硬件方向

首选 `Mini Pupper 2`。它的官方资料、ROS2 指南、快速上手和 AI 入口都比较完整，而且官方明确给了中国淘宝购买入口。

## 代码定位

这个仓库先把“大模型把自然语言变成安全动作计划”这件事做稳。后面接真机器时，把 `RobotPlan` 映射到：

- ROS2 的 `/cmd_vel`
- 机器人姿态话题
- 语音播报
- 摄像头巡检

这样不会把业务逻辑绑死在某个硬件 SDK 上。
