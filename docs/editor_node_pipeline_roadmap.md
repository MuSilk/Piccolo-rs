# 节点编辑器：从着色器节点到 RenderPass / Subpass / Pipeline（路线图）

本文档在原有「固定 Pass 编排」基础上，**重新规划**为：以**给定着色器节点**为输入，在编辑器中构图，由工具链与运行时**自动推导** `VkRenderPass`（含 subpass、依赖）与 `VkPipeline`（及 layout、descriptor），最终驱动 `RenderPipeline`。

---

## 1. 目标与边界

### 1.1 核心目标

| 层级 | 目标 |
|------|------|
| **编辑器** | 基于「着色器节点」构图：每个节点绑定 SPIR-V（或 GLSL 路径）、暴露输入/输出端口（纹理、buffer、attachment 语义等），连线表达数据流与执行顺序。 |
| **中间表示（IR）** | 图编译为与引擎无关的 **Render Graph IR**（节点、边、资源生命周期、subpass 候选划分）。 |
| **RenderPass** | 由 IR + 全局 **Framebuffer 契约**（附件槽位、格式）自动生成：`VkAttachmentDescription`、`VkSubpassDescription`、`VkSubpassDependency`、`preserve_attachments` 等。 |
| **Pipeline** | 每个可执行子图单元对应 **GraphicsPipelineCreateInfo**：shader 阶段、顶点输入（全屏三角等）、blend、dynamic state；与 **PipelineLayout**（descriptor set / push constant）一致。 |
| **调度** | `RenderPipeline` 按编译后的 **pass 序 + subpass 序** 调用 `cmd_begin_render_pass` / `cmd_next_subpass` / draw，资源绑定与 IR 一致。 |

### 1.2 边界（分阶段）

- **阶段 0～1**：仍以「主相机单 framebuffer、已知附件槽」为底盘；节点先覆盖 **全屏后处理链**（Tone / Grade / FXAA 等）或与现有 `SubpassLayout` 可映射的类型。
- **阶段 2**：引入「纯着色器节点」模板（单输入 RT → 单输出 RT），自动分配 ping-pong attachment，仍限制在**单 render pass、线性 subpass** 内。
- **阶段 3**（可选）：多 render pass、与 shadow / UI 等 **跨 pass** 资源同步；或迁移 **Vulkan Dynamic Rendering**，弱化传统 `VkRenderPass` 手工拼装。
- **不做（初期）**：任意拓扑的任意 GLSL（无类型约束）；跨进程资产市场；完整材质图与 Mesh 节点图（可与本路线图并行另立文档）。

---

## 2. 概念模型：三层图

```
┌─────────────────────────────────────────────────────────────┐
│  A. 编辑视图图（Editor Graph）                                │
│  - 节点：ShaderNode | RasterPassNode | CompositeNode …       │
│  - 边：端口级，带类型（SceneColor / Depth / …）               │
└──────────────────────────┬──────────────────────────────────┘
                           │ 序列化 JSON / 二进制
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  B. 资源与执行 IR（Runtime Graph IR）                         │
│  - 拓扑序、环检测、端口 → 物理 attachment / buffer 绑定        │
│  - 每个 ShaderNode：SPIR-V、入口、descriptor 绑定意图          │
│  - Subpass 划分：默认 1 节点 ≈ 1 subpass（可合并见 §6）        │
└──────────────────────────┬──────────────────────────────────┘
                           │ build_render_pass + build_pipelines
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  C. Vulkan 对象                                              │
│  - VkRenderPass + VkFramebuffer + VkPipeline[] + Layout      │
└─────────────────────────────────────────────────────────────┘
```

与现有代码对齐：

- **B 的子集** 已由 `RenderGraphAsset` / `CompiledRenderGraph` / `graph_compiler::compile_render_graph` 实现拓扑与节点种类。
- **C 的 RenderPass 部分** 已由 `graph_compiler::build_subpass` + 边 `framebuffer` 字段 + `SubpassLayout` 思路生成子通道与依赖；后续把 **Pipeline** 生成接到同一 IR。

---

## 3. 着色器节点：最小 schema（建议）

每个 **ShaderNode**（或后处理节点）在资产中至少包含：

```json
{
  "id": 10,
  "kind": "ShaderFullscreen",
  "name": "MyTone",
  "shader": {
    "spirv_path": "generated/spv/my_tone.frag.spv",
    "stage": "fragment",
    "entry": "main"
  },
  "ports": {
    "inputs": [
      { "name": "hdr", "kind": "SubpassInput", "binding": 0 }
    ],
    "outputs": [
      { "name": "ldr", "kind": "ColorAttachment", "format": "inherit" }
    ]
  },
  "params": []
}
```

设计要点：

- **ports** 与 Vulkan **descriptor / attachment** 一一可追溯；`binding` / `input_attachment_index` 与 SPIR-V 反射或手写表一致。
- **outputs** 的物理槽由边 `framebuffer` 字段或自动分配器写入（与当前 `main_camera_pass.json` 一致）。
- 编辑器侧可先用 **固定模板**（ToneMapping / ColorGrading / FXAA）填好 `ports`，再过渡到通用 `ShaderFullscreen`。

---

## 4. RenderPass / Subpass 自动生成策略

### 4.1 默认策略（易实现、易调试）

- **一个「可 raster 的 Shader 节点」→ 一个 subpass**（与当前 `MainCameraPass` 拆分一致）。
- **边** 声明 `from_port` / `to_port` / `framebuffer`（附件语义名），编译器映射到 `_MAIN_CAMERA_PASS_*` 常量（见 `graph_compiler::framebuffer_name_to_index`）。
- **依赖**：线性链 `EXTERNAL → 0` + `i-1 → i`；`preserve_attachments` 用「前序已写 ∩ 后续要读 \ 本 subpass 已用」的通用公式（已实现思路，见 `compute_preserve_attachments`）。

### 4.2 与「仅节点、无手工 framebuffer」结合

- 自动分配器：在 **DAG + 端口类型** 约束下，为每条「逻辑 RT」分配 `BACKUP_ODD/EVEN/POST_*` ping-pong，并**写回**边或 IR 缓存，再调 `build_subpass`。
- 校验：无环、无端口类型错连、swapchain 仅出现在最后一级合成等。

### 4.3 可选优化（后期）

- **同一 subpass 内合并**多个全屏 draw：仅当两个节点 **无中间 RT 需求** 且可合并为 **单 shader 多 pass 内** 或 **动态渲染单次** 时启用（见前文「单 shader 串行」讨论）；默认关闭。

---

## 5. Pipeline 自动生成策略

### 5.1 每个 subpass 对应 pipeline 条目

| 组件 | 来源 |
|------|------|
| **Shader stages** | 节点 `spirv_path` + `stage`；全屏可共用 `post_process.vert.spv`。 |
| **Vertex input** | 全屏三角：空或固定 `vec2` 位置。 |
| **Input attachments** | 端口 `SubpassInput` → `VkDescriptorSetLayout` + `VkPipeline` 与 render pass **subpass 索引** 一致。 |
| **Color blend** | 默认单 RT `ONE, ZERO`；需 alpha 合成时由节点 meta 或端口类型切换。 |
| **Dynamic state** | viewport/scissor 与现有一致。 |
| **Pipeline layout** | 反射 SPIR-V 或节点声明的 `set/binding` 生成 layout；与 **descriptor 写入** 同源。 |

### 5.2 实现顺序建议

1. **反射或静态表**：对现有 `tone_mapping` / `color_grading` / `fxaa` 的 SPIR-V 建立「binding → 端口名」表（可先手写，后 `shaderc`/`spirv-reflect`）。
2. **工厂函数**：`fn build_fullscreen_pipeline(rhi, render_pass, subpass_index, node: &ShaderNodeIr) -> VkPipeline`。
3. **缓存**：`(render_pass, subpass_index, shader_hash)` 为 key，图变更时 invalidation。

---

## 6. 编辑器功能规划（相对旧版路线图的调整）

### 6.1 已具备（保留描述）

- `editor/src/render_graph/*`：节点、端口语义边、校验、JSON IO、画布 UI。
- 资产：`asset/render_graph/*.json`、`asset/render_pipeline/main_camera_pass*.json`。

### 6.2 新增 / 强化

| 模块 | 内容 |
|------|------|
| **节点类型** | 区分 `BuiltinPassNode`（与现有一致）与 `ShaderNode`（带 `shader` 字段与端口 schema）。 |
| **Inspector** | 编辑 `spirv_path`、entry、宏/ specialization、端口 binding 覆盖。 |
| **预览 / 校验** | 调用 **headless 编译器**（可复用 `runtime` 的 `compile_render_graph` + 未来 `validate_shader_ports`）在保存前报错。 |
| **导出** | 一键导出「IR JSON + 可选生成的 pipeline 描述 JSON」，供 runtime 加载。 |

---

## 7. Runtime 集成规划

| 步骤 | 工作 |
|------|------|
| R1 | 扩展 `RenderGraphNodeKind` 或并行 **ShaderNodeId**，IR 中含 `spirv` 与端口绑定。 |
| R2 | `graph_compiler`：`compile_render_graph` 保持拓扑；新增 **`compile_pipeline_layout`** / **`build_pipelines_for_graph`**（或并入 `MainCameraPass::initialize`）。 |
| R3 | `build_subpass` 输入不变或仅增加「自动 framebuffer 分配」分支；与 **pipeline 数组下标** 与 `execution_order` 对齐。 |
| R4 | `MainCameraPass::draw`：按 `CompiledRenderGraph.execution_order` 绑定 pipeline + descriptor + `cmd_next_subpass`。 |
| R5 | 特性开关：`use_shader_graph_pipelines`，失败回退固定管线。 |

---

## 8. 里程碑（建议重排）

| 阶段 | 交付物 | 说明 |
|------|--------|------|
| **M1** | IR 文档 + `ShaderNode` JSON schema + 编辑器加载/展示 | 不接 Vulkan，仅数据层。 |
| **M2** | 1 个「自定义全屏 ShaderNode」走通：手填 SPIR-V + 单 subpass pipeline | 证明 pipeline 自动生成闭环。 |
| **M3** | 现有 Tone→Grade→FXAA 链改为 **数据驱动**（节点仍可用内置 kind，但绑定信息来自 IR） | 与 `build_subpass` 对齐。 |
| **M4** | 自动 attachment 分配 + `preserve` 全自动 | 减少 JSON 手工 `framebuffer`。 |
| **M5** | 多 shader 链 + 性能与缓存 | 可选动态渲染迁移评估。 |

---

## 9. 风险与规避

| 风险 | 规避 |
|------|------|
| SPIR-V 与端口 / attachment 不一致 | 保存时运行校验；CI 跑小型 `vkCreateGraphicsPipelines` 冒烟（可选）。 |
| RenderPass 与 Pipeline 不同步 | 单一入口 `rebuild_from_graph(asset)` 原子更新两者。 |
| 调试困难 | 保留「固定管线」开关；导出 IR 与 Vulkan 参数 JSON 对照。 |
| 合并 subpass 过度优化 | 默认一节点一 subpass；合并仅作实验 flag。 |

---

## 10. 与旧版章节的对应关系

- 原「阶段 A～D」中 **A（编辑+保存）**、编辑器 UI 与 JSON：**已完成**。
- 原「阶段 B（图驱动调度）」：**部分完成**（`compile_render_graph`、`build_subpass`）；**Pipeline 与 draw 循环仍待接**。
- 原「阶段 C（资源依赖）」：**演进为** §4 自动分配 + §5 descriptor 与 attachment 同源。
- 原「阶段 D（扩展节点）」：**聚焦为** ShaderNode + IR schema，而非泛泛自定义节点。

---

## 11. 附录：依赖层级（示意）

### 执行顺序依赖（DAG）

- 例：`DirectionalShadow` / `PointShadow` → `MainCamera` 子图入口 → … → `CombineUI`。

### Framebuffer / attachment 依赖

- 边 `framebuffer` 或 IR 分配的物理槽 → `build_subpass` / `VkFramebuffer` 附件列表顺序一致。

### Subpass 依赖

- 线性 `VkSubpassDependency` + 通用 `preserve_attachments`；与拓扑序一致即可保证可见性。

---

## 12. 近期可执行任务清单（按新路线图）

**短期（M1～M2）**

1. 在 `docs/` 或 `asset/schema/` 固化 **ShaderNode JSON schema**（可与编辑器共用）。
2. 编辑器：`ShaderNode` 类型 + Inspector 编辑 `shader` 元数据。
3. Runtime：实现 **单节点测试** 的 `build_fullscreen_pipeline` + 与单 subpass render pass 对接。

**中期（M3）**

4. 将 `ToneMapping` / `ColorGrading` / `FXAA` 的绑定信息抽成 **数据表**，由 IR 填充而非硬编码。
5. `MainCameraPass::draw` 按 `CompiledRenderGraph` 循环 subpass 与 pipeline。

**中长期（M4～M5）**

6. 自动 ping-pong 分配与 JSON 简化。
7. Pipeline 缓存与可选 Dynamic Rendering 评估。

---

*文档版本：以「着色器节点 → RenderPass + Pipeline」为主线的修订版；与仓库中 `graph_compiler.rs`、`editor/src/render_graph/` 当前实现相互参照更新。*
