use anyhow::{Result};
use runtime::app::App;
pub mod editor;
use crate::editor::editor::Editor;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut app = App::default();
    app.add_system(Editor::default());
    app.run();

    Ok(())
}