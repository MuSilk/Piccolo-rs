# 渲染管线结构总结与自定义管线落地方案

## 1. 当前管线结构（基于现有代码）

### 1.1 顶层调用链

每帧渲染主流程位于 `RenderSystem::tick`：

1. `process_swap_data`：处理逻辑线程提交的资源增删、相机参数更新。
2. `VulkanRHI::prepare_context`：准备当帧上下文。
3. `RenderResource::update_per_frame_buffer`：更新 per-frame GPU 数据。
4. `RenderScene::update_visible_objects`：可见性更新，写入可见节点集合。
5. `RenderPipeline::prepare_pass_data`：为各 Pass 准备本帧数据。
6. `RenderSystem::render`：
   - 等待 fence、重置 command pool；
   - `prepare_before_pass`（含 swapchain 重建分支）；
   - `RenderPipeline::draw` + `DebugDrawManager::draw`；
   - `submit_rendering`。

### 1.2 RenderPipeline 组成

`RenderPipeline` 当前固定包含四个 Pass：

- `DirectionalLightShadowPass`
- `PointLightShadowPass`
- `MainCameraPass`
- `PickPass`

初始化顺序是：先两种阴影 Pass，再主相机 Pass，最后 PickPass。  
其中主相机 Pass 会读取阴影 Pass 生成的 image view（方向光/点光阴影贴图）。

### 1.3 MainCameraPass 的多子通道结构

`MainCameraPass` 在一个 Vulkan RenderPass 中定义了 8 个 subpass（`MainCameraSubPass`）：

1. `BasePass`
2. `DeferredLighting`
3. `ForwardLighting`
4. `ToneMapping`
5. `ColorGrading`
6. `FXAA`
7. `UI`
8. `CombineUI`

其附件体系（9 个 attachment）包括：

- GBuffer A/B/C
- backup odd/even（场景和后处理中间结果）
- post process odd/even
- depth
- swapchain image（最终输出）

### 1.4 实际绘制路径（Deferred 与 Forward）

- `forward_draw == false`（默认 Deferred）：
  - BasePass（写 GBuffer）
  - DeferredLighting（读取 GBuffer + depth，输出到 backup odd）
  - ForwardLighting（此路径下只做 subpass 前进，不绘制 mesh）
  - ToneMapping -> ColorGrading -> (可选) FXAA -> UI -> CombineUI

- `forward_draw == true`（Forward）：
  - 跳过 BasePass 和 DeferredLighting 的绘制逻辑
  - ForwardLighting 中执行 `draw_mesh_lighting + draw_skybox`
  - 后续同样走 ToneMapping/ColorGrading/FXAA/UI/CombineUI

### 1.5 后处理 Pass 的组织方式

`ToneMappingPass`、`ColorGradingPass`、`FXAAPass`、`UIPass`、`CombineUIPass` 都是独立模块，  
但绑定在 `MainCameraPass` 的同一个 RenderPass 的特定 subpass 上。

这意味着：

- 逻辑上是模块化的；
- 资源和 subpass 拓扑上仍是“单大 RenderPass”架构；
- 扩展成本主要在 `MainCameraPass::setup_render_pass`（attachment/subpass/dependency）与对应 descriptor 更新。

---

## 2. 目前可扩展性的优缺点

### 优点

- 子 Pass 都有单独文件，便于复制已有后处理模板。
- 资源上传与 descriptor 管理范式统一，复用成本低。
- 已有 `recreate_after_swapchain` 的更新路径，便于新附件接入。

### 限制

- `RenderPipeline` 是固定字段，不是动态图结构，插拔式扩展能力弱。
- `MainCameraPass` 维护大量 attachment/subpass 索引常量，新增节点时改动点多。
- RenderGraph/GraphCompiler 已存在但未成为主路径，当前仍是手工编排。

---

## 3. 自定义管线的可实践方案

下面给出两级方案：**短期可快速落地** 与 **中期可维护重构**。建议先做 A，再演进到 B。

## 3.1 方案 A（推荐先做）：在 MainCameraPass 内增量扩展

目标：最小改动、快速上线一个自定义后处理或自定义光照子阶段。

### A1. 适用场景

- 增加一个屏幕后处理（例如：边缘描边、锐化、自定义 LUT、景深简化版）。
- 在现有流程中插入一个 subpass，不引入全新的跨 RenderPass 依赖。

### A2. 实施步骤（按顺序）

1. 新建 Pass 模块（建议仿照 `tone_mapping_pass.rs` 或 `color_grading_pass.rs`）：
   - 定义 `InitInfo`、`initialize`、`draw`、`update_after_framebuffer_recreate`。
2. 在 `passes.rs` 导出新模块。
3. 在 `MainCameraSubPass` 增加新枚举位（例如 `CustomPostProcess`）。
4. 在 `MainCameraPass`：
   - 增加新 Pass 字段；
   - 在 `initialize` 中初始化；
   - 在 `draw` 中插入 `cmd_next_subpass` 与 `new_pass.draw`；
   - 在 `recreate_after_swapchain` 中调用 `new_pass.update_after_framebuffer_recreate`。
5. 在 `setup_render_pass` 中改动三部分：
   - attachment（若需要新的输入/输出中间缓冲）；
   - subpass 描述（input/color attachment）；
   - subpass dependency（前后阶段同步）。
6. 若用到新 shader：
   - 加入 shader 编译产物；
   - 在对应 Pass 的 pipeline 创建中接入。
7. 编译验证并检查 swapchain resize 路径是否完整更新 descriptor。

### A3. 关键注意事项

- attachment 索引常量必须全链路同步，避免读写错位。
- `INPUT_ATTACHMENT` 与 `COMBINED_IMAGE_SAMPLER` 的 descriptor 类型要与 shader 声明一致。
- 如果新增 subpass，记得补全 dependency，否则容易出现隐式同步错误。
- 在 `forward_draw` 与 `deferred_draw` 两条路径都要检查 subpass 推进数量一致。

### A4. 预计改动文件

- `runtime/src/function/render/passes.rs`
- `runtime/src/function/render/passes/main_camera_pass.rs`
- `runtime/src/function/render/passes/<your_custom_pass>.rs`
- `runtime/src/shader/...` 与 `runtime/src/shader/generated/...`（取决于 shader 接入方式）

---

## 3.2 方案 B（中期）：把 RenderPipeline 从“固定字段”改为“可配置阶段”

目标：让“自定义管线”成为配置/注册行为，而不是每次手工改大文件。

### B1. 核心思路

1. 定义统一 Pass trait（示例）：
   - `prepare(&mut self, ...)`
   - `draw(&self, ...)`
   - `on_swapchain_recreate(&mut self, ...)`
2. `RenderPipeline` 持有 `Vec<Box<dyn RenderPassNode>>`（或枚举驱动）。
3. 用配置（json/toml）或编译期注册控制 Pass 顺序与开关。
4. 将 MainCamera 内部后处理拆分为可注册的“post process chain”。

### B2. 迁移建议

- 第一步先只把“后处理链”抽出来动态化，阴影与主几何路径先保持静态。
- 第二步再考虑把阴影、拾取等也统一成节点。
- 全量改造前可先引入“配置驱动启停”能力，降低硬编码分支。

### B3. 风险与收益

- 风险：前期重构量较大，涉及生命周期和资源拥有关系。
- 收益：后续加新效果基本变成“新增模块 + 注册”，维护成本显著下降。

---

## 4. 建议的落地路线图

### Sprint 1（1~2 天）

- 采用方案 A，新增一个自定义后处理 pass（例如 `custom_post_pass`）。
- 打通初始化、绘制、resize 重建。
- 在 `forward/deferred` 两种路径都跑通。

### Sprint 2（2~4 天）

- 提炼 `MainCameraPass` 的后处理公共逻辑（descriptor 更新、全屏三角形绘制模板）。
- 规范 attachment 命名与索引管理（避免魔法数字扩散）。

### Sprint 3（按需求）

- 启动方案 B：先做“后处理链动态化”最小版本。
- 若稳定再扩展到完整 RenderPipeline 节点化。

---

## 5. 一句话结论

当前项目是“固定主干 + MainCamera 多 subpass”的实用型架构；  
要实现自定义管线，最可实践路径是先在 `MainCameraPass` 内按模板增量插入自定义 Pass，等功能稳定后再逐步演进到可配置的动态管线。
