# CuEngine

基于 Rust + vulkano 的 2D 游戏引擎，3D功能不做。

层间依赖声明与依赖选型：**写到该步骤时再现场定，不提前规划**。

游戏引擎五层架构参考 Piccolo：
- 平台层 platform 用以支持各个硬件平台，如ps switch epic steam。
- 核心层 core 用以系统调用，如垃圾管理 内存分配 线程管理 以及数学模块等底层工具
- 资源层 resorce 用以管理各类数据，如声音图像视频等
- 功能层 function 用以实现操作系统具体功能，如渲染 动画 物理等
- 工具层 tool 用以给使用者提供创作工具
  
上述四层在piccolo的具体实现在D:\games\Piccolo\engine\source\runtime,此外因piccolo是3D引擎，实现时亦可参考godot

另外source\editor是编辑器

## 构建与运行

```powershell
cargo run
```

环境：Rust stable (MSVC)、`Visual Studio\BuildTools`、`VulkanSDK\1.4.350.0`、RTX 4060 (Vulkan 1.4.312)。

---
## 以下是用来给AI看的，给人看的完整readme回头再写
### 各层在 Piccolo 的落点（照抄参考）

## 优先级

| 级别 | 内容 |
| --- | --- |
| **P0** | 骨架与核心：窗口/Vulkan 初始化、2D 渲染、场景树、资源加载 |
| **P1** | 完整度：文本渲染、帧动画、输入、配置、调试 overlay |
| **P2** | 物理、脚本、音频、粒子、TileMap —— 时间不足先砍这里 |

| 层 | Piccolo 位置 | 内容 |
| --- | --- | --- |
| 平台层 | `runtime/platform` | 只有 `file_service` + `path`，共 93 行；纯文件/路径，与窗口输入无关 |
| 核心层 | `runtime/core` | base、log、math、color、meta（反射） |
| 资源层 | `runtime/resource` | `asset_manager`、`config_manager`、`res_type` |
| 功能层 | `runtime/function` | render（含 `window_system`）、input、animation、physics、framework、ui、particle、character、controller、global |
| 工具层 | `source/editor` | 编辑器 |

> 引擎主循环 `runtime/engine.cpp` 在 runtime 顶层：持有各系统、`logicalTick → world_manager->tick`。**每帧自动 tick 对象树（`Level::tick → GObject::tick → Component::tick`），使用者只实现 `Component::tick`，不自己驱动循环。**
>
> CuEngine 对应 **`runtime/engine` crate**：位于 runtime 顶层、**独立于四层之外**，由 `app` 调用。

---

### 开发方式：自顶向下（纵向切片）

先打通"app 入口 → 屏幕可见"的最薄一条路径，再逐层加厚，底层用到再补。

- 分层与依赖方向不变（上层依赖下层）；只是**实现顺序**自顶向下。
- 下层先给"够用的最小实现"，接口留清楚，后续替换实现，别让临时硬编码渗进上层。
- 每轮迭代都要能跑出可见成果。

---

### 迭代计划

### 迭代 0：空窗口 + 帧循环

依赖方向：`app → engine → function → engine-core`（`engine` 直接用 `engine-core`）。

- `runtime/engine`（顶层）：`Engine` 持有各系统、实现 winit 事件循环与每帧 tick；帧计时 + FPS 计数
- 功能层 `function`：`WindowSystem`（建窗、事件、close/尺寸）
- 核心层 `engine-core`：log
- `app`：`main` 建 Engine、驱动循环
- **可见成果**：空窗口出现，标题显示 FPS，可关闭

### 迭代 1：Vulkan 清屏

- 功能层：Renderer（Instance + validation、设备/队列、Swapchain、2 frames-in-flight、RenderPass、清屏）
- **可见成果**：纯色窗口，resize/最小化不崩，validation layer 0 error

> 迭代 0+1 = W1 目标（纯色窗口）。

### 迭代 2：配置外置 + 文件读取

- 资源层：`config_manager`（窗口尺寸/vsync/log 级别）
- 平台层：文件系统 + 路径工具
- **可见成果**：改配置文件即可改窗口参数

### 迭代 3：画图元 + 相机

- 核心层：2D 数学、坐标系约定、正交投影
- 功能层：shader、顶点/索引缓冲、正交相机、精灵渲染
- **可见成果**：屏幕上出现精灵

### 迭代 4：游戏框架

- 功能层：`GObject` / `Component` / `Level` / `World`，引擎每帧自动 tick
- 资源层：资源管理（纹理等）、场景文件读写
- **可见成果**：一个会动的对象

### 工具层（后续）

- 编辑器 / 创作工具

### W1 总验收

五层目录就位；空窗口跑起来、纯色清屏、resize 与最小化不崩、validation clean。

---

## 后续里程碑

| 阶段 | 目标 |
| --- | --- |
| **W2** | 迭代 2 + 迭代 3：配置/文件、精灵渲染、正交相机 |
| **W3** | 迭代 4：游戏框架、资源管理、场景文件 |
| **W4** | 整合 demo + 录屏 + 五层架构图，开题报告与 PPT（**开题 ≈ 一个月后**） |
| **M2** | 文本渲染、帧动画、输入完善 |
| **M3** | 物理、脚本、热重载（P2，可砍） |
| **M4** | 编辑器、示例小游戏、性能分析、论文撰写 |

## 风险与砍单预案

- Vulkan 必须在 Windows 侧运行，WSL 只负责编辑。
- 砍单顺序：**P2 全砍 → P1 砍部分**，P0 骨架保证完整。
- vulkano 各版本 API 变动大，实现只认所装版本的官方 examples，不照抄网络教程。
