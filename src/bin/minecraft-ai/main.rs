//! 独立示例 `minecraft-ai`：体素地形 + 第一人称移动，实现与 `src/bin/minecraft` 无代码复用关系。
//!
//! 运行：`cargo run --bin minecraft-ai`
//!
//! 不挂载 `Editor`，以纯游戏循环运行（无编辑器 UI/系统）。

use runtime::app::App;

mod game_scene;
mod player_controller;
mod voxel_world;

fn main() {
    let mut app = App::new();
    app.add_scene(game_scene::GameScene::new());
    app.set_default_scene("MinecraftAI");
    app.run();
}
