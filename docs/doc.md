# CuEngine 开发文档

## 项目定位

**最终目标：完成一个吃豆人（Pac-Man）游戏。**

- **开题目标：按住空格播放帧动画。**
- **物理系统：必做（不再放进可砍的 P2）。**

游戏引擎五层架构：

- 平台层 platform 用以支持各个硬件平台，如 ps switch epic steam。
- 核心层 core 用以系统调用，如垃圾管理 内存分配 线程管理 以及数学模块等底层工具
- 资源层 resource 用以管理各类数据，如声音图像视频等
- 功能层 function 用以实现操作系统具体功能，如渲染 动画 物理等
- 工具层 tool 用以给使用者提供创作工具

上述四层在 Piccolo 的具体实现在 `D:\games\Piccolo\engine\source\runtime`，此外因 Piccolo 是 3D 引擎，实现时亦可参考 Godot。另外 `source\editor` 是编辑器。

## 构建与运行

```powershell
cargo run
```

环境：Rust stable (MSVC)、`Visual Studio\BuildTools`、`VulkanSDK\1.4.350.0`、RTX 4060 (Vulkan 1.4.312)。

## 优先级


| 级别     | 内容                                 |
| ------ | ---------------------------------- |
| **P0** | 骨架与核心：窗口/Vulkan 初始化、2D 渲染、场景树、资源加载、**物理系统（必做）**、**TileMap** |
| **P1** | 完整度：文本渲染、帧动画、输入、配置、调试 overlay      |
| **P2** | 脚本、音频、粒子 —— 时间不足先砍这里    |


## 各层在 Piccolo 的落点（照抄参考）


| 层   | Piccolo 位置         | 内容                                                                                                  |
| --- | ------------------ | --------------------------------------------------------------------------------------------------- |
| 平台层 | `runtime/platform` | 只有 `file_service` + `path`，共 93 行；纯文件/路径，与窗口输入无关                                                    |
| 核心层 | `runtime/core`     | base、log、math、color、meta（反射）                                                                        |
| 资源层 | `runtime/resource` | `asset_manager`、`config_manager`、`res_type`                                                         |
| 功能层 | `runtime/function` | render（含 `window_system`）、input、animation、physics、framework、ui、particle、character、controller、global |
| 工具层 | `source/editor`    | 编辑器                                                                                                 |


> 引擎主循环 `runtime/engine.cpp` 在 runtime 顶层：持有各系统、`logicalTick → world_manager->tick`。**每帧自动 tick 对象树（`Level::tick → GObject::tick → Component::tick`），使用者只实现 `Component::tick`，不自己驱动循环。**
>
> CuEngine 对应 `**runtime/engine` crate**：位于 runtime 顶层、**独立于四层之外**，由 `app` 调用。

## 开发方式：自顶向下（纵向切片）

先打通"app 入口 → 屏幕可见"的最薄一条路径，再逐层加厚，底层用到再补。

- 分层与依赖方向不变（上层依赖下层）；只是**实现顺序**自顶向下。
- 下层先给"够用的最小实现"，接口留清楚，后续替换实现，别让临时硬编码渗进上层。
- 每轮迭代都要能跑出可见成果。

## 迭代计划

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

### 迭代 2：配置外置 + 文件读取

目标：引擎启动时从配置文件读取窗口标题/尺寸、日志级别、vsync，改文件即生效。

依赖方向（本迭代）：`resource → platform`，`engine → resource`（resource 用 platform 读文件；engine 用 resource 读配置）。

- **平台层 `platform**`（对齐 Piccolo，纯 `std::fs` / `std::path`，无第三方库）
  - `path`：exe 所在目录、路径拼接、扩展名/文件名/纯名（对齐 Piccolo `Path`）
  - `file_service`：读文件为字符串、存在性判断、列目录（对齐 Piccolo `FileSystem`）
- **资源层 `resource**`
  - `config`：`ConfigManager`，加载配置 → `EngineConfig { window{title,width,height}, log_level, vsync }`
  - 配置缺失 → 用默认值 + warn
  - 配置位置：exe 同级目录（用 platform 的 path 定位）
- **功能层 `function**`
  - `render`：vsync → swapchain present mode（`true` = `Fifo`；`false` = 尝试 `Mailbox`/`Immediate`）
- `**engine**`：`run()` 开头加载配置 → 设日志级别 → 用 window 配置建窗 → 把 vsync 传给 `RenderSystem`

**配置格式**：`toml` + `serde`（外部 crate；直观，将来场景序列化也复用）。

**验收方法：怎么测 vsync 生效**

- `vsync = true`：标题 FPS 锁在刷新率（60Hz≈60，144Hz≈144）。
- `vsync = false`：FPS 解除上限、明显更高。
- FPS 是平滑过的（α=1/100），启动约 1 秒后才稳定，别看第一帧。
- 坑：
  - Windows 窗口化 + DWM：桌面合成器本身按刷新率呈现，MAILBOX 窗口化也可能被压到 ~刷新率；真正不锁帧通常要**独占全屏**。
  - 驱动不支持 MAILBOX 时会退回 FIFO，此时 `vsync = false` 仍表现为锁帧。
- 结论：以标题 FPS 判断；窗口化下若差异不明显，属 DWM/驱动所致，不是实现错。

### 迭代 3：画图元 + 相机（已完成）

- 核心层：`engine-core` 重导出 `glam`；`Camera2D`（`function/render/camera.rs`）用 `glam::camera::rh::proj::vulkan::orthographic`，**自动处理 Vulkan 的 Y 下 / Z[0,1]**
- 功能层：`sprite.vert`/`sprite.frag`（`build.rs` 用 `glslc` 编译 → `$OUT_DIR/*.spv` → `render::shader` 常量）；顶点/索引缓冲、uniform 缓冲（相机矩阵）、描述符集、`GraphicsPipeline`、每帧 `draw_indexed`
- **可见成果**：屏幕中央一个 100×100 的四色渐变 quad（`sprite.frag` 暂只输出顶点色，纹理见待补）

**渲染模块分工（`function/render`）**：

```
function/src/render/
├── window.rs          窗口 + winit 事件循环（WindowHandler 回调）
├── shader.rs          shader 字节常量（build.rs 用 glslc 编译）
├── camera.rs          Camera2D（正交投影）
├── render_system.rs   RenderSystem（engine 的门面）
└── vulkan/
    ├── context.rs     VulkanContext：实例/设备/队列/分配器
    ├── swapchain.rs   SwapchainContext：交换链/图像/帧缓冲/render pass + resize 重建
    ├── sprite.rs      SpriteRenderer：管线/布局/描述符/缓冲/相机 + 录制
    └── renderer.rs    VulkanRenderer：组合三者，render_frame 编排
```

**一次性初始化 vs 每帧**：

- 一次性（各 `new`）：
  - `context`：Instance / Surface / 物理设备 / Device+Queue / 三个分配器（command buffer、memory、descriptor set）
  - `swapchain`：Swapchain / images / image views / framebuffers / render pass
  - `sprite`：顶点·索引·uniform 缓冲、shader module、PipelineLayout、DescriptorSet、GraphicsPipeline、Camera2D、Viewport
- 每帧（`renderer.render_frame`）：
  - 尺寸变化 → 重建 swapchain / framebuffer
  - 更新相机 viewport + 写 uniform 缓冲内容
  - `acquire_next_image`
  - 录制命令缓冲：`begin_render_pass → set_viewport → bind pipeline → bind vertex/index → bind descriptor → draw_indexed → end_render_pass`
  - `submit → present → 等 fence`

对应 Piccolo：`context`≈RHI 初始化、`swapchain`≈RHI 的 `createSwapchain/recreate`、`sprite`≈`render_pipeline`/`passes`、`renderer`≈`render_system` + `prepareBeforePass`/`submitRendering`。

**如何把"画方块"从初始化里封装出去（下一步方向）**：

现状：`sprite.rs::new` 把 quad 的顶点/颜色**写死**，`record` 固定画一个 quad。

方向：把 **"画什么"(数据)** 与 **"怎么画"(机制)** 分开（对齐 Piccolo 的 `RenderScene` / `render_resource` / `render_pipeline`）：

- **渲染数据**（新）：一个 `Sprite { position, size, texture, uv, color }` 列表，由逻辑层（将来 framework 的组件）产出。
- **渲染器**（`sprite.rs`）：只管机制——管线/布局/采样器建一次；持**动态顶点缓冲**，每帧从 sprite 列表**重建顶点数据**（每个 sprite → 4 顶点 / 6 索引）并上传，然后一次 `draw_indexed`。
- 即 `SpriteRenderer::draw(&mut self, sprites: &[Sprite])` 取代写死的 quad；进阶做**批渲染**（动态缓冲 + 图集 → 一次 draw call）。
- 将来：framework 把 `GameObject` / `SpriteComponent` → 渲染数据 → 渲染器绘制。

### 迭代 4：游戏框架

- 功能层：`GObject` / `Component` / `Level` / `World`，引擎每帧自动 tick
- 资源层：资源管理（纹理等）、场景文件读写
- **可见成果**：一个会动的对象

### 迭代 5：纹理 + 精灵渲染

- `Image` + staging 上传 + `Sampler`（**Nearest** 过滤，关 mipmap）+ 第二个描述符绑定
- 引入渲染数据 `Sprite { position, size, texture, uv, color }` 列表 + 动态顶点缓冲 + 批渲染
- **可见成果**：把一张静态像素图显示到屏幕

### 迭代 6：输入系统

- `InputState`（`pressed` / `just_pressed` / `released`），键盘 A/D
- 放进功能层 `function/input`，接进 `WindowHandler`
- **可见成果**：能读到 A/D 按键状态

### 迭代 7：按住空格播放帧动画（开题交付物）

- 精灵表切帧 + 帧动画组件（按时间切换 UV/帧）
- 输入：按住空格 → 播放动画，松开 → 停/回到首帧
- **可见成果：按住空格播放帧动画 —— 开题 demo 就绪**

### 迭代 8（可选）：像素完美渲染

- 低分辨率 render target（如 320×180）+ 整数倍最近邻放大
- 相机整数缩放 + 像素吸附
- 图集 + 批渲染 / 实例化、视口裁剪

**优先级**：开题前把 **5 → 6 → 7** 打通（8 可后置）。需要一张**像素小人贴图（PNG）**；没有的话先用程序化占位图。

### 工具层（后续）

- 编辑器 / 创作工具

### W1 总验收

五层目录就位；空窗口跑起来、纯色清屏、resize 与最小化不崩、validation clean。

## 像素游戏优化清单

### 渲染侧（像素画质 + 性能）

1. **最近邻采样**：`Sampler` 用 `Nearest`、关 mipmap / anisotropy——否则像素会被插值糊掉。**必做**。
2. **低分辨率渲染 + 整数倍放大**：内部渲染到 320×180 之类的小 RT，再整数倍最近邻放大到窗口——经典"像素完美"，避免非整数缩放导致的抖动/糊边。
3. **相机像素吸附**：相机位置/缩放到整像素（整数 pixels-per-unit），否则会亚像素抖动（shimmer）。
4. **精灵批渲染 + 图集**：所有 sprite 打进一张图集 → 顶点合成一个动态缓冲 → **一次 draw call**；大量精灵时收益巨大。
5. **绘制顺序**：2D 要按 z/顺序做画家算法（批渲染时按顺序/纹理分桶）。
6. **视口裁剪**：屏幕外的 sprite 不进顶点缓冲。
7. （可选）**实例化**渲染重复精灵。

### 工具层（针对像素工作流）

1. **贴图导入设置**：每张纹理可配 filter（nearest）、是否 premultiply alpha 等（像 Godot 的 import preset）。
2. **精灵表切片 / 动画帧**：从一张 sprite sheet 切帧、定义动画。
3. **网格 + 像素吸附**：编辑器里按像素网格对齐（整数坐标）。
4. **图集生成**：从素材目录自动打包 atlas + 元数据。
5. **整数倍预览**：编辑器视口按整数倍缩放预览，避免误判。
6. **热重载**：改场景/贴图即时生效（我们已有文本场景的基础）。
7. （可选）**Tilemap 编辑器**——若小人是平台跳跃类才需要。

建议真做（有余力且好写进论文）：**像素完美渲染**（低分辨率 RT + 最近邻 + 整数缩放 + 相机像素吸附）与**精灵批渲染 + 图集**（可测 draw call 数 / 帧率）。

## 待补（先记着，以后再说）

1. 真正的 2 frames-in-flight：目前每帧 `future.wait(None)` 等 GPU 完成（正确但吞吐低），要补 per-frame semaphore + fence。
2. 错误处理统一成错误类型 + 日志：目前初始化用 `expect`、acquire 失败用 `eprintln`。
3. 接 `DebugUtilsMessenger` 显式收集校验信息（现在只开了校验层，靠默认输出）。
4. 透明窗口：`CLEAR_COLOR` 第 4 位确实是 alpha，但当前 swapchain 用的是 surface 首个格式（不带 alpha / 合成器当不透明处理），改 alpha 无效果。要做透明需：swapchain 换带 alpha 的格式（如 `R8G8B8A8_UNORM`）、窗口支持 layered、并在合成时混合。暂不做。
5. 测试分离：把各 crate 里的 `#[cfg(test)] mod tests` 拆到独立文件（如 `runtime/engine/src/engine/tests.rs`，用 `#[cfg(test)] mod tests;` 引入）。
6. 纹理/精灵：`sprite.frag` 目前只输出顶点色（不采样）；要做纹理需 `Image` + 上传（staging buffer + copy）+ `Sampler` + 第二个描述符绑定。

## 后续里程碑


| 阶段     | 目标                                            |
| ------ | --------------------------------------------- |
| **W2** | 迭代 2 + 迭代 3：配置/文件、精灵渲染、正交相机                   |
| **W3** | 迭代 4：游戏框架、资源管理、场景文件                           |
| **W4** | 整合 demo + 录屏 + 五层架构图，开题报告与 PPT（**开题 ≈ 一个月后**） |
| **M2** | TileMap、物理系统、AI、游戏逻辑、文本渲染                      |
| **M3** | 脚本、音效、像素优化、吃豆人整合（脚本/音效可砍）                  |
| **M4** | 编辑器、示例小游戏、性能分析、论文撰写                           |


## 风险与砍单预案

- Vulkan 必须在 Windows 侧运行，WSL 只负责编辑。
- 砍单顺序：**P2 全砍 → P1 砍部分**，P0 骨架保证完整。
- vulkano 各版本 API 变动大，实现只认所装版本的官方 examples，不照抄网络教程。

 