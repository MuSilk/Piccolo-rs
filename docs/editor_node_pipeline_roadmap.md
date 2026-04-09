# Editor 节点编辑器实现路径（用于渲染管线构建）

## 1. 目标与边界

### 目标
- 在 `editor` 中提供可视化节点编辑器，用于构建和调整渲染管线流程。
- 输出稳定的“管线图描述（Graph）”，由运行时转换为可执行的 pass 调度顺序。
- 初期优先支持当前已有渲染链路：阴影 -> 主相机 -> 后处理 -> UI 合成。

### 边界（第一阶段）
- 不修改底层 Vulkan RHI 资源模型，只做编辑器层与渲染管线层的编排适配。
- 不做任意自定义 shader 节点（先支持固定类型节点 + 参数编辑）。
- 不做跨工程共享资产系统（先把图存为项目内配置文件）。

---

## 2. 与现有工程的对接点

当前代码中，渲染 pass 主要在 `runtime/src/function/render/render_pipeline.rs` 内按固定顺序调用：
- `m_directional_light_pass.draw()`
- `m_point_light_pass.draw()`
- `m_main_camera_pass.draw/draw_forward(...)`
- `m_ui_pass` + `m_combine_ui_pass`

节点编辑器的目标不是立刻替换全部逻辑，而是先把“顺序与开关”参数化：
1. 编辑器产出 `RenderGraphAsset`（节点+连线+参数）。
2. 运行时把 `RenderGraphAsset` 编译为 `CompiledRenderGraph`（可执行拓扑）。
3. `RenderPipeline` 增加“图驱动模式”，按编译结果调度现有 pass。

---

## 3. 建议目录结构

### Editor 侧（可视化与编辑）
- `editor/src/render_graph/`
  - `graph_types.rs`：节点/Pin/连线的数据结构（编辑态）。
  - `graph_state.rs`：当前画布状态、选择态、拖拽态、缩放平移。
  - `graph_commands.rs`：增删节点、连线、撤销重做命令。
  - `graph_ui.rs`：节点画布 UI 绘制与交互（基于现有 `UiRuntime`）。
  - `graph_inspector.rs`：节点参数面板（右侧 Detail）。
  - `graph_io.rs`：图的序列化/反序列化（TOML/JSON）。

### Runtime 侧（编译与执行）
- `runtime/src/function/render/render_graph/`
  - `graph_asset.rs`：运行时可加载的图资源结构。
  - `graph_compiler.rs`：拓扑排序、连线校验、资源依赖分析。
  - `graph_executor.rs`：将编译结果映射到 `RenderPipeline` 调用。
  - `node_registry.rs`：节点类型注册表（节点类型 -> pass 执行器）。

---

## 4. 核心数据模型（建议）

```rust
pub type NodeId = u64;
pub type PinId = u64;

pub enum NodeKind {
    DirectionalShadow,
    PointShadow,
    MainCamera,
    ToneMapping,
    ColorGrading,
    FXAA,
    UIPass,
    CombineUI,
}

pub struct GraphNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    pub position: [f32; 2],
    pub params: std::collections::HashMap<String, String>,
    pub input_pins: Vec<PinId>,
    pub output_pins: Vec<PinId>,
}

pub struct GraphEdge {
    pub from_pin: PinId,
    pub to_pin: PinId,
}

pub struct RenderGraphAsset {
    pub version: u32,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
```

设计要点：
- `NodeKind` 先与现有 pass 一一对应，降低接入复杂度。
- `params` 先采用字符串字典，后续可升级为强类型参数系统。
- 必须支持版本号，方便后续资产迁移。

---

## 5. 节点画布 UI 实现策略

结合当前 `editor/src/editor_ui.rs` 的多面板布局，新增一个“Render Graph”面板：

1. 在 `EditorUI::show_editor_game_window` 或新增专用窗口函数中接入画布渲染。
2. 画布能力最小集：
   - 网格背景（可选）
   - 节点框渲染（标题/输入输出 pin）
   - 鼠标拖拽移动节点
   - 连线创建（从输出 pin 拖到输入 pin）
3. 交互状态建议放在 `graph_state.rs`：
   - `selected_node`
   - `dragging_node`
   - `creating_edge_from_pin`
   - `canvas_offset`, `canvas_zoom`

备注：当前 `UiRuntime` 已能渲染文本、按钮、面板，可先用“矩形+文本+按钮点击区域”拼装节点画布，后续再抽象控件。

---

## 6. 图校验与编译规则（Runtime）

编译阶段（`graph_compiler.rs`）建议分三步：

1. **结构校验**
   - pin 连接方向是否合法（输出 -> 输入）
   - 单输入 pin 是否被重复连接
   - 必需节点是否存在（如 MainCamera / CombineUI）

2. **环检测与拓扑排序**
   - 检测有向环，报错并定位节点。
   - 输出稳定拓扑序，保证执行确定性。

3. **执行计划生成**
   - 生成 `CompiledPassCall` 数组（按拓扑顺序）。
   - 每个调用项带节点参数快照与输入资源引用。

---

## 7. 渲染管线接入方案（渐进）

### 阶段 A：仅做“可视化 + 保存”
- Editor 可编辑图并写入 `asset/render_graph/default_graph.toml`。
- Runtime 不消费该图（仅工具链验证）。

### 阶段 B：图驱动开关与顺序
- Runtime 加载图，映射到已有 pass 调用顺序。
- 先支持 pass 开启/关闭和固定链路顺序调整（受白名单约束）。

### 阶段 C：图驱动资源依赖
- 在不重写 RHI 的前提下，补充附件输入输出映射。
- 把 `tone_mapping/color_grading/fxaa/combine_ui` 的输入附件来源改为图配置。

### 阶段 D：扩展节点类型
- 引入自定义后处理节点与参数模板。
- 接入材质/shader 资源描述（可选）。

---

## 8. 文件格式建议（先 TOML）

建议新增：
- `asset/render_graph/default_graph.toml`
- `asset/render_graph/schema_version.toml`（可选）

示例（简化）：
```toml
version = 1

[[nodes]]
id = 1
kind = "DirectionalShadow"
name = "Directional Shadow"
position = [120.0, 180.0]

[[nodes]]
id = 2
kind = "MainCamera"
name = "Main Camera"
position = [420.0, 180.0]

[[edges]]
from_pin = 1001
to_pin = 2001
```

---

## 9. 开发里程碑（推荐 4 周）

### Week 1：编辑器基础闭环
- 完成 `graph_types/graph_state/graph_io`。
- 在 Editor 中显示节点卡片与拖拽。
- 支持保存/加载图文件。

### Week 2：连线与参数面板
- 完成 pin 连线交互与删除。
- 增加 Inspector 参数编辑。
- 增加基础校验提示（非法连线、缺失输入）。

### Week 3：Runtime 编译与只读执行
- 完成 `graph_compiler`（校验+拓扑排序）。
- `RenderPipeline` 接入图驱动模式（先只读，不改资源绑定）。
- 输出执行日志用于比对固定管线。

### Week 4：图驱动实际调度
- 支持 pass 开关、顺序控制。
- 接入最小附件依赖映射（后处理链）。
- 做性能与稳定性回归。

---

## 10. 风险与规避

- 风险：UI 交互复杂度高（拖拽、连线、命中）  
  规避：先做最小交互，不一次性上缩放/框选/多选。

- 风险：图配置与 runtime 实际能力不一致  
  规避：用 `NodeRegistry` 严格限制节点类型与连接规则。

- 风险：替换固定管线导致回归  
  规避：保留“固定管线模式”开关，支持 A/B 运行切换。

---

## 11. 近期可执行任务清单（直接开工）

1. 在 `editor/src` 新增 `render_graph` 模块骨架与 `mod.rs` 导出。
2. 在 `EditorUI` 增加 `Render Graph` 面板入口（先显示静态节点）。
3. 增加 `graph_io.rs`，先硬编码保存/加载到 `asset/render_graph/default_graph.toml`。
4. 在 `runtime` 新增 `render_graph/graph_asset.rs`，定义与 editor 对齐的数据结构。
5. 给 `RenderPipeline` 增加 `use_render_graph` 配置入口（先只打印编译顺序，不改变 draw 行为）。

以上 5 项完成后，你就有了“编辑器可编辑 + runtime 可读取 + 可验证执行计划”的最小闭环。
