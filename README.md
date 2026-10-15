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

**迭代 1 待补（先记着，以后再说）**：

1. 真正的 2 frames-in-flight：目前每帧 `future.wait(None)` 等 GPU 完成（正确但吞吐低），要补 per-frame semaphore + fence。
2. 错误处理统一成错误类型 + 日志：目前初始化用 `expect`、acquire 失败用 `eprintln`。
3. `render/vulkan.rs` 拆成 `context / swapchain / frame`。
4. 接 `DebugUtilsMessenger` 显式收集校验信息（现在只开了校验层，靠默认输出）。
5. 透明窗口：`CLEAR_COLOR` 第 4 位确实是 alpha，但当前 swapchain 用的是 surface 首个格式（不带 alpha / 合成器当不透明处理），改 alpha 无效果。要做透明需：swapchain 换带 alpha 的格式（如 `R8G8B8A8_UNORM`）、窗口支持 layered、并在合成时混合。暂不做。
6. 测试分离：把各 crate 里的 `#[cfg(test)] mod tests` 拆到独立文件（如 `runtime/engine/src/engine/tests.rs`，用 `#[cfg(test)] mod tests;` 引入）。

### 迭代 2：配置外置 + 文件读取

目标：引擎启动时从配置文件读取窗口标题/尺寸、日志级别、vsync，改文件即生效。

依赖方向（本迭代）：`resource → platform`，`engine → resource`（resource 用 platform 读文件；engine 用 resource 读配置）。

- **平台层 `platform`**（对齐 Piccolo，纯 `std::fs` / `std::path`，无第三方库）
  - `path`：exe 所在目录、路径拼接、扩展名/文件名/纯名（对齐 Piccolo `Path`）
  - `file_service`：读文件为字符串、存在性判断、列目录（对齐 Piccolo `FileSystem`）
- **资源层 `resource`**
  - `config`：`ConfigManager`，加载配置 → `EngineConfig { window{title,width,height}, log_level, vsync }`
  - 配置缺失 → 用默认值 + warn
  - 配置位置：exe 同级目录（用 platform 的 path 定位）
- **功能层 `function`**
  - `render`：vsync → swapchain present mode（`true` = `Fifo`；`false` = 尝试 `Mailbox`/`Immediate`）
- **`engine`**：`run()` 开头加载配置 → 设日志级别 → 用 window 配置建窗 → 把 vsync 传给 `RenderSystem`

**待拍板——配置格式**：
- A) `toml` + `serde`（外部 crate，你装；直观，将来场景序列化也复用）
- B) 手写极简 `key = value` 解析（零依赖，多写 ~30 行）

**任务清单**：

1. `platform`：`path` + `file_service`（纯 std）
2. `resource`：`config` 模块（加载 + 默认值）
3. 加内部 path 依赖：`resource → platform`、`engine → resource`
4. 配置格式定下后加对应依赖（你）
5. `function/render`：vsync → present mode
6. `engine`：`run()` 读配置、设日志级别、传 window/vsync 配置
7. 放一份示例配置文件
8. **验收**：改标题/尺寸/日志级别/vsync 后重启生效；配置文件缺失走默认 + warn

**验收方法：怎么测 vsync 生效**

- `vsync = true`：标题 FPS 锁在刷新率（60Hz≈60，144Hz≈144）。
- `vsync = false`：FPS 解除上限、明显更高。
- FPS 是平滑过的（α=1/100），启动约 1 秒后才稳定，别看第一帧。
- 坑：
  - Windows 窗口化 + DWM：桌面合成器本身按刷新率呈现，MAILBOX 窗口化也可能被压到 ~刷新率；真正不锁帧通常要**独占全屏**。
  - 驱动不支持 MAILBOX 时会退回 FIFO，此时 `vsync = false` 仍表现为锁帧。
- 结论：以标题 FPS 判断；窗口化下若差异不明显，属 DWM/驱动所致，不是实现错。

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
