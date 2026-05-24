# 采购清单

这份清单按“淘宝能买到、中文资料多、先跑通再升级”排序。价格和库存变化很快，实际下单以淘宝店铺页面为准；这里固定的是选型逻辑和搜索词。

## 首选方案

| 项目 | 建议 | 淘宝搜索词 | 作用 | 备注 |
| --- | --- | --- | --- | --- |
| 四足本体 | `Mini Pupper 2 预装版` | `Mini Pupper 2 预装版` | 直接拿到可玩的四足平台 | 官方文档给了中国购买入口，优先选预装版 |
| 边缘计算网关 | `Jetson Orin Nano Super Developer Kit` | `Jetson Orin Nano Super 开发套件` | 跑视觉、语音、局部推理和 ROS2 边缘服务 | NVIDIA 官方资料明确面向边缘生成式 AI；适合做机器人脑侧网关 |
| 核心板 + 底板首选 | `Raspberry Pi Compute Module 5 + Compute Module 5 IO Board` | `树莓派 CM5 IO Board 套件` | 低功耗控制、原型开发、IO 扩展 | 官方 IO Board 是 Compute Module 的配套载板，资料和社区最稳 |
| 国产 RK3588 成套备选 | `Firefly Core-3588J + ITX-3588J 底板` | `Firefly Core-3588J ITX-3588J RK3588` | 视频、模型推理、Linux 原型验证 | 中文 Wiki 明确是 Core-3588J + MB-JM3-RK3588ITX 的开发板组合 |
| 激光雷达 | `STL-06P` | `STL-06P 雷达` | 建图、导航、避障 | 官方文档明确提到这套雷达适配更稳 |
| 摄像头 | `Raspberry Pi Camera v2` | `树莓派 摄像头 v2` | 视觉巡检、识别、远程画面 | 先用成熟款，不要一开始就堆复杂视觉 |
| 控制计算机 | `Ubuntu 22.04` 笔记本或迷你主机 | `Ubuntu 22.04 迷你主机` | 跑 ROS2、调试、开发 | 也可以先用你现有电脑 |
| 存储卡 | `64GB U3 microSD` | `64GB U3 TF卡` | 系统盘、备份镜像 | 建议多备一张 |
| 备件 | 电池、充电器、螺丝、线材 | `Mini Pupper 电池` | 保障持续测试 | 直接按官方套件配件买 |

## 预算建议

- 入门验证：先按本体 + 控制电脑 + 基础配件走。
- 边缘算力升级：优先加 `Jetson Orin Nano` 或 `RK3588` 套件。
- 低功耗控制板：如果更偏本体控制和接口扩展，优先看 `CM5 + IO Board`。
- 进阶巡检：再加雷达和摄像头。
- 如果要做更完整的室内导航，再补一套更稳定的充电和备电方案。

## 核心板 + 底板成套开发板推荐

| 优先级 | 成套开发板 | 建议买法 | 适合角色 | 为什么推荐 | 注意点 |
| --- | --- | --- | --- | --- | --- |
| 1 | `Raspberry Pi CM5 + CM5 IO Board` | 买 `CM5 8GB/16GB + 官方 CM5IO + 5V5A 电源 + NVMe SSD` | 本体控制板、中继控制板、ROS2 原型机 | 官方文档、社区、系统镜像和 ROS2 资料最容易找；IO Board 提供电源、GPIO、双 CSI/DSI、HDMI、USB、千兆网、M.2 扩展 | 不适合本地跑大模型；大模型走云 API 或交给 Jetson/RK3588 |
| 2 | `Firefly Core-3588J + ITX-3588J 底板` | 买 `Core-3588J 8GB/16GB + ITX-3588J 开发板套件 + 电源 + 散热` | 国产边缘网关、视觉处理、视频输入输出 | RK3588 算力和接口更强，中文 Wiki 完整，淘宝供应相对直接 | 板卡分支多，下单前确认内存、存储、Wi-Fi、底板型号一致 |
| 3 | `Jetson Orin Nano Super Developer Kit` | 买官方开发套件，再补 NVMe、主动散热、电源 | 主边缘计算网关、视觉/语音/本地小模型 | NVIDIA 官方软件栈适合边缘 AI 和机器人原型，2026 年资料仍活跃 | 成本和功耗高于 CM5；底层电机 IO 不建议直接挂在 Jetson 上 |

实际采购时，不建议把三套都一次买齐。第一阶段买 `Mini Pupper 2 预装版 + 控制电脑`；第二阶段按目标补一套网关或核心板：

- 想最快做大模型 + 视觉：补 `Jetson Orin Nano Super Developer Kit`。
- 想做低功耗本体控制和 IO 扩展：补 `CM5 + CM5 IO Board`。
- 想走国内供应链并留足视频/推理接口：补 `Firefly Core-3588J + ITX-3588J`。

## 边缘计算网关采购包

| 物料 | 推荐 | 作用 |
| --- | --- | --- |
| 主机 | `Jetson Orin Nano Super Developer Kit` 或 `Firefly Core-3588J + ITX-3588J` | 运行 `robot-dog-rust --serve`、视觉/语音服务、ROS2 上层节点 |
| 存储 | `128GB/256GB NVMe SSD` | 保存系统镜像、日志、模型缓存、ROS bag |
| 网络 | 千兆网线或稳定 5GHz Wi-Fi | 保证控制电脑、网关、本体在同网段 |
| 供电 | 原厂或商家配套电源 | 减少边缘推理时掉电和重启 |
| 散热 | 主动散热模块 | Jetson/RK3588 持续推理需要稳定散热 |

网关不直接接管舵机和步态。正确边界是：网关接收自然语言和视觉/语音输入，生成 `RobotPlan`；本体控制板或 ROS2 节点把 `RobotPlan` 映射到具体话题和动作。

## 选型原则

1. 先买官方生态里能直接跑的东西。
2. 先把 ROS2 和动作控制跑通，再谈自己改底层。
3. 不要一开始就买太散的零件，四足平台的时间成本比硬件差价更贵。

## 备选方案

| 方案 | 优点 | 缺点 |
| --- | --- | --- |
| `Jetson Orin Nano` | 边缘推理能力强，生态成熟 | 价格和功耗都更高 |
| `Raspberry Pi CM5 + IO Board` | 开发资料多，结构清晰 | 算力不如 Jetson，重模型不合适 |
| `RK3588 开发板 + 底板` | 国内资料和供应链更友好 | 板卡分裂严重，要挑成熟底板 |
| `Petoi Bittle X` | 中文资料也不少，开源友好 | 更偏轻量娱乐，不是最强 ROS2 起点 |
| `Unitree Go2` | 硬件成熟度高 | 成本高，生态更偏官方闭环 |

## 实操建议

- 第一单只买本体和基础供电，不要一次买满。
- 先验证软件链路，再决定要不要补雷达和视觉。
- 如果目标是 Rust + 大模型，底层先别改，先把控制面做好。

## 已核验资料入口

- Mini Pupper 官方订购页列出中国 `Taobao` 渠道。
- Mini Pupper 快速上手文档说明预装版已有软件镜像，并包含 AI 功能入口。
- Mini Pupper ROS2 文档说明其 ROS2 版本基于开源项目并覆盖 SLAM、Navigation、Simulation。
- NVIDIA 官方 Jetson Orin Nano Super 资料说明开发套件面向边缘生成式 AI，性能从 40 TOPS 升到 67 TOPS。
- Raspberry Pi 官方文档说明 CM5IO 是 Compute Module 5 的配套 IO Board，提供电源、USB、网口、M.2、相机/显示等接口。
- Firefly 中文 Wiki 说明 Core-3588J 使用 RK3588，ITX-3588J 由 Core-3588J + MB-JM3-RK3588ITX 组成。

具体链接集中放在 `docs/source_index.md`，避免采购表变得太长。
