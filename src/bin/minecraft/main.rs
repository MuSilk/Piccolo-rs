use editor::editor::Editor;
use runtime::{app::App};

mod block_res;
mod block;
mod chunk;
mod world;
mod scene;

fn main() {
    let mut app = App::new();
    app.add_system(Editor::default());

    let scene = scene::Scene::new();
    app.add_scene(scene);
    app.set_default_scene("MineCraft");

    app.run();
}