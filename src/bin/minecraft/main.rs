use runtime::app::App;

mod game_scene;
mod minecraft_motor_component;
mod player_controller;
mod voxel_world;

fn main() {
    let mut app = App::new();
    app.add_scene(game_scene::GameScene::new());
    app.set_default_scene("MinecraftAI");
    app.run();
}
