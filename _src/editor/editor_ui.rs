use crate::runtime::function::ui::window_ui::{WindowUI, WindowUIInitInfo};

#[derive(Default)]
pub struct EditorUI {

}

impl WindowUI for EditorUI  {
    fn initialize(&self, _info: WindowUIInitInfo) {

    }

    fn pre_render(&self) {
        
    }
}