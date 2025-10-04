use std::{cell::{RefCell}, env, rc::Rc};

use anyhow::{anyhow,Result};

use crate::{editor::editor::Editor, runtime::engine::{Engine}};

pub mod runtime;
pub mod editor;
pub mod shader;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let executable_path = env::current_exe()?;
    let config_file_path = executable_path.parent().ok_or_else(||
        anyhow!("Failed to get parent directory")
    )?;

    let engine = Engine::default();
    engine.initialize(config_file_path);
    let engine = Rc::new(RefCell::new(engine));

    let mut editor = Editor::default();
    editor.initialize(&engine)?;

    editor.run()?;

    editor.shutdown();
    engine.borrow().shutdown();

    Ok(())
}