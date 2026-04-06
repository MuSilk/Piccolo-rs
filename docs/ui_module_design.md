# 自研 UI 模块设计方案（替代 imgui）

## 1. 目标与约束

### 目标
- 将业务 UI 从 `imgui` 解耦，构建可控、可扩展、可主题化的自研 UI 模块。
- 与当前渲染管线兼容：继续走 `UIPass` 和 `CombineUIPass` 合成流程。
- 支持游戏常见能力：HUD、菜单、弹窗、输入框、焦点导航、动画、分辨率自适应。

### 约束
- 当前工程已有 Vulkan 渲染基础与 UI 子通道（见 `runtime/src/function/render/passes/ui_pass.rs`）。
- 需要渐进迁移，不能一次性替换全部 UI。
- 运行时性能优先，减少 CPU 分配与状态切换。

## 2. 总体方案

采用 **Retained + Diff + Batched Draw** 的混合架构：
- 上层使用保留式组件树（更适合游戏 UI 状态管理）。
- 每帧计算变化（diff），只更新脏区域和脏节点。
- 底层输出统一 `UiDrawList`，进入现有 `UIPass` 绘制与 `CombineUIPass` 合成。

> 这样可以保留 imgui 的“每帧出绘制命令”的稳定性，同时获得更清晰的 UI 状态模型。

## 3. 模块分层设计

建议新增 `runtime/src/function/ui2/`（可按实际命名调整）：

- `core/`
  - `ui_runtime.rs`：UI 生命周期与帧入口。
  - `widget_tree.rs`：节点树、父子关系、脏标记。
  - `state_store.rs`：状态存储（按 widget id）。
  - `event.rs`：输入事件与冒泡/捕获。
  - `focus.rs`：键盘/手柄焦点系统。
- `layout/`
  - `measure.rs`：测量阶段（min/max/content）。
  - `flex.rs`：基础布局（行列、对齐、padding）。
  - `anchor.rs`：游戏 HUD 常用锚点布局。
- `style/`
  - `theme.rs`：主题、颜色、字号、圆角、阴影。
  - `atlas.rs`：字体/图标图集索引。
- `render/`
  - `draw_cmd.rs`：`UiDrawCmd`、`UiVertex`、`UiDrawList`。
  - `tessellator.rs`：矩形/圆角/文本网格化。
  - `ui_renderer_backend.rs`：把 `UiDrawList` 提交给 Vulkan。
- `widgets/`
  - `panel.rs`、`text.rs`、`button.rs`、`slider.rs`、`image.rs`、`input.rs`。

## 4. 关键数据结构

```rust
pub struct UiFrame {
    pub frame_id: u64,
    pub dt: f32,
    pub viewport: [f32; 2],
    pub input: UiInputSnapshot,
}

pub struct UiDrawList {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
    pub commands: Vec<UiDrawCmd>, // 含 scissor / texture / blend key
}

pub enum UiDrawCmd {
    DrawIndexed {
        first_index: u32,
        index_count: u32,
        vertex_offset: i32,
        clip_rect: [f32; 4],
        texture_id: u32,
    }
}
```

### 设计要点
- 所有 widget 必须有稳定 `WidgetId`（字符串 hash 或编译期 id）。
- 渲染侧按 `(texture_id, clip_rect, blend)` 聚合，减少 draw call。
- 文本渲染统一走字形缓存，避免每帧重复栅格化。

## 5. 与现有渲染管线的对接

当前已有流程：场景渲染 -> `UIPass` -> `CombineUIPass`。

迁移策略：
1. 保留 `UIPass` 的 Vulkan 管线创建逻辑（descriptor/pipeline/buffer）。
2. 将 `imgui_render(draw_data)` 替换为 `ui2_render(draw_list)`。
3. `UiRuntime::build_frame()` 每帧产出 `UiDrawList`。
4. `UIPass` 仅负责 GPU 资源上传、scissor、draw indexed。

即：`imgui::DrawData` 只是中间格式，你的目标是替换成项目内定义的 `UiDrawList`。

## 6. 输入系统设计

统一输入快照：
- 鼠标：位置、按键、滚轮。
- 键盘：按下、字符输入、修饰键。
- 手柄（可选）：方向、确认、取消。

流程：
1. `WindowSystem` 收集原始事件。
2. 转换为 `UiInputSnapshot`。
3. `UiRuntime` 执行命中测试（hit test）并派发事件。
4. 先捕获再冒泡；焦点组件优先处理键盘事件。

## 7. 布局与坐标体系

- 逻辑坐标使用 dp（与分辨率解耦）。
- 渲染前统一转换为像素坐标（乘 DPI scale）。
- 支持安全区（safe area）与锚点（左上/右上/底部居中等）。

推荐最小布局能力：
- `Row`, `Column`, `Stack`, `Anchor`, `Padding`, `SizedBox`。
- 文本自动测量，图片支持保持纵横比。

## 8. 资源管理

- 字体：预构建 SDF/MSDF 图集，运行时按语言包加载。
- 图标：合并到 icon atlas，减少 texture 切换。
- 皮肤：JSON/TOML 主题配置，可热更新。

可沿用你当前 `asset` 目录风格，新增 `asset/ui/`：
- `asset/ui/theme/default.toml`
- `asset/ui/font/*.ttf`
- `asset/ui/atlas/*.png`

## 9. 迁移路线（建议 4 个里程碑）

### M1：打通最小闭环（1-2 周）
- 新建 `UiDrawList` 和 `UiRuntime` 骨架。
- `UIPass` 支持消费 `UiDrawList`（矩形 + 文字）。
- 用一个简单 HUD（例如 FPS 文本）验证链路。

### M2：核心控件与输入（1-2 周）
- `Text`、`Panel`、`Button`、`Image`。
- 鼠标命中、点击、悬停、焦点切换。
- 替换一个现有 imgui 面板。

### M3：布局与主题（2 周）
- Row/Column/Anchor 布局。
- 主题系统（颜色、字号、间距）。
- 引入基础动画（alpha/position tween）。

### M4：完全替换与清理（1 周）
- 迁移剩余窗口逻辑。
- 清理 `imgui` 依赖与桥接代码。
- 建立回归测试与性能基线。

## 10. 性能与稳定性建议

- 每帧复用 `Vec` 容量（`clear` 不 `shrink`）。
- 使用帧环形缓冲，避免频繁创建/销毁 Vulkan buffer。
- 对文本和静态控件缓存网格化结果（仅在脏时重建）。
- 打点统计：`ui_build_ms`、`ui_upload_ms`、`ui_draw_calls`、`ui_vertices`。

## 11. 测试策略

- 单元测试：布局计算、事件冒泡顺序、焦点切换。
- 集成测试：窗口缩放、DPI 变化、输入法文本。
- 视觉回归：关键 UI 场景截图对比（菜单/HUD/设置页）。
- 性能回归：固定场景记录 draw call 和帧时间。

## 12. 风险与规避

- 风险：文本渲染复杂度高  
  规避：先做 ASCII/英文路径，后续补全多语言 shaping（如 rustybuzz）。

- 风险：一次性替换成本高  
  规避：保留 `imgui` 并行期，按窗口逐步迁移。

- 风险：布局系统设计过重  
  规避：先实现游戏必需布局，再扩展到通用容器。

## 13. 你项目里的落地建议（最小改动版）

第一阶段可只做三件事：
- 在 `runtime` 内新增 `ui2` 模块，定义 `UiRuntime` 和 `UiDrawList`。
- 在 `UIPass` 里新增 `render_ui_draw_list(&UiDrawList)`，暂不删除原 `imgui_render`。
- 在游戏入口（`src/bin/greedy_snake/main.rs`）增加开关：`use_ui2 = true/false`，用于 A/B 对比。

这样可以快速验证自研 UI 的工程可行性，同时保留回退路径。
