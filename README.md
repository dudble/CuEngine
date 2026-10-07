# CuEngine

以下全是AI帮忙写的，回头再改

基于 Rust + vulkano 的 2D 游戏引擎（毕业设计）。

四层架构参考 Piccolo：**工具层 → 功能层 → 资源层 → 核心层**，上层依赖下层，禁止反向依赖。3D 功能不做。

- 层间依赖声明与依赖选型：**写到该步骤时再现场定，不提前规划**。
- 参考源码：Piccolo（重点 `engine/source/runtime`）、Godot
## 构建与运行

```powershell
cd D:\games\CuEngine
cargo run
```

环境：Rust stable (MSVC)、`Visual Studio\BuildTools`、`VulkanSDK\1.4.350.0`、RTX 4060 (Vulkan 1.4.312)。

## 优先级

| 级别 | 内容 |
| --- | --- |
| **P0** | 骨架与核心：窗口/Vulkan 初始化、2D 渲染、场景树、资源加载 |
| **P1** | 完整度：文本渲染、帧动画、输入、配置、调试 overlay |
| **P2** | 可砍：物理、脚本、音频、粒子、TileMap —— 时间不足先砍这里 |

---

## W1 任务清单（目标：分层骨架 + 纯色窗口）

### D1 工程骨架

- [x] workspace 成员：`app` + `runtime/{core,platform,resource,function}`
- [x] runtime 各层用 `src/lib.rs`（可被引用），`app` 用 `src/main.rs`
- [x] `cargo build` / `cargo run` 跑通
- [x] 包名 `core`：撞 Rust 标准库 `core`，实测会让 `core::time::Instant` 报 E0433，接依赖后必炸


验收：整区编译通过、格式化/lint 通过、依赖方向无环。

### D2 core 层

- [ ] 日志：级别过滤、带时间戳
- [ ] 时间：帧 delta、FPS 平滑（参考 Piccolo `engine.cpp` 的 `s_fps_alpha`）
- [ ] 2D 数学与坐标系约定：原点、Y 朝向、像素坐标 vs 世界坐标、正交投影
- [ ] 基础类型：`Handle { index, generation }`、错误类型、事件枚举
- [ ] 以上各自带单元测试

### D3 配置 + 窗口

- [ ] 配置文件：窗口标题/尺寸、vsync、日志级别（对应 Piccolo `config_manager`）
- [ ] 窗口事件循环：关闭、尺寸变化、键盘、鼠标
- [ ] 输入状态缓存：`pressed` / `just_pressed` / `released`，每帧刷新

### D4–D5 Vulkan 初始化（最重，照抄对应版本 vulkano 的官方 example）

- [ ] Instance + validation layer（debug 构建开启）
- [ ] 物理设备选择（优先独显 NVIDIA）+ 逻辑设备/队列
- [ ] Swapchain：格式与呈现模式从配置读取
- [ ] 双缓冲同步：2 frames-in-flight，每帧 semaphore + fence
- [ ] RenderPass + 清屏 + submit/present

验收：纯色窗口稳定 60fps，validation layer 0 error。

### D6 健壮性

- [ ] resize / 最小化（0 尺寸）/ `OUT_OF_DATE` → 重建 swapchain
- [ ] FPS 显示到窗口标题（对应 Piccolo `setTitle`）
- [ ] 错误统一走错误类型 + 日志，禁止裸 `unwrap` 崩溃

验收：反复拖窗口、切后台、最小化均不崩。

### D7 收尾

- [ ] README 补充构建/运行细节
- [ ] `git commit`
- [ ] 录 W1 demo、画四层架构图（开题素材）
- [ ] 预研 W2：2D 批渲染方案

---

## 后续里程碑

| 阶段 | 目标 |
| --- | --- |
| **W2** | 着色器 + 顶点/索引缓冲、正交相机、精灵渲染 |
| **W3** | 资源管理（纹理等）、场景树 + Transform、场景文件读写 |
| **W4** | 整合 demo + 录屏 + 架构图，完成开题报告与 PPT（**开题答辩 ≈ 一个月后**） |
| **M2** | 文本渲染、帧动画、输入完善、配置系统 |
| **M3** | 物理、脚本、热重载（P2，可砍） |
| **M4** | 示例小游戏、性能分析、论文撰写 |

## 风险与砍单预案

- 时间不足时的砍单顺序：**P2 全砍 → P1 砍部分**，P0 骨架保证完整。
- vulkano 各版本 API 变动大，实现只认所装版本的官方 examples
