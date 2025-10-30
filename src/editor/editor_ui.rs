use std::{cell::RefCell};

use imgui::{Condition, WindowFlags};
use runtime::{core::math::vector2::Vector2, function::{ui::window_ui::{WindowUI, WindowUIInitInfo}}};

use crate::editor::editor_global_context::EditorGlobalContext;


#[derive(Default)]
pub struct EditorUI {
    m_state: State,
}

#[derive(Default)]
pub struct State {
    m_editor_menu_window_open: RefCell<bool>,
    m_asset_window_open: RefCell<bool>,
    m_game_engine_window_open: RefCell<bool>,
    m_file_content_window_open:RefCell<bool>,
    m_detail_window_open:RefCell<bool>,
    m_scene_lights_window_open:RefCell<bool>,
    m_scene_lights_data_window_open:RefCell<bool>,
}

impl WindowUI for EditorUI  {
    fn initialize(&mut self, _init_info: WindowUIInitInfo) {
    
    }

    fn pre_render(&mut self, ui: &mut imgui::Ui) {
        self.show_editor_ui(ui);
        let global = EditorGlobalContext::global().borrow();
        let window_system = global.m_window_system.upgrade().unwrap();
        let window_size = window_system.borrow().get_window_size();
        let mut input_manager = global.m_input_manager.borrow_mut();
        input_manager.set_engine_window_size(Vector2::new(window_size.0 as f32, window_size.1 as f32));
    }
}

impl EditorUI {
    fn show_editor_ui(&mut self, ui: &mut imgui::Ui) {

        let size = ui.io().display_size;
        let main_window = ui
            .window("Main Docking")
            .position([0.0, 0.0], Condition::Always)
            .size([size[0], size[1]], Condition::Always)
            .flags( WindowFlags::MENU_BAR | WindowFlags::NO_TITLE_BAR |
                    WindowFlags::NO_COLLAPSE| WindowFlags::NO_RESIZE | 
                    WindowFlags::NO_MOVE | WindowFlags::NO_BACKGROUND |
                    WindowFlags::NO_MOUSE_INPUTS | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
            );
        let mut context_offset = [0.0, 18.0];
        let mut context_size = [size[0], size[1]-18.0];

        let components_details_window = ui
            .window("Components Details")
            .position([context_offset[0] + 0.75 * context_size[0] , context_offset[1]], Condition::Always)
            .size([0.25 * context_size[0], context_size[1]], Condition::Always)
            .flags(WindowFlags::NO_COLLAPSE);
        context_size[0] -= 0.25 * context_size[0];

        let file_content_window = ui
            .window("File Content")
            .position([context_offset[0]  , context_offset[1] + 0.7 * context_size[1]], Condition::Always)
            .size([context_size[0], 0.3 * context_size[1]], Condition::Always)
            .flags(WindowFlags::NO_COLLAPSE);
        context_size[1] -= 0.3 * context_size[1];

        let world_object_window: imgui::Window<'_, '_, &'static str> = ui
            .window("World Object")
            .position([context_offset[0]  , context_offset[1]], Condition::Always)
            .size([0.3 * context_size[0],  context_size[1]], Condition::Always)
            .flags(WindowFlags::NO_COLLAPSE);
        context_offset[0] += 0.3 * context_size[0];
        context_size[0] -= 0.3 * context_size[0];

        main_window.build(||{});
        components_details_window.build(||{
            self.show_editor_detail_window(ui);
        });
        file_content_window.build(||{
            self.show_editor_file_context_window(ui);
        });
        world_object_window.build(||{
            self.show_editor_world_objects_window(ui);
        });
        self.show_editor_menu(ui, &mut self.m_state.m_editor_menu_window_open.borrow_mut());
    }

    fn show_editor_menu(&self, ui: &mut imgui::Ui, p_open: &mut bool) {

        if let Some(_menu_bar) = ui.begin_main_menu_bar() {
            if let Some(_) = ui.begin_menu("Menu") {
                if ui.menu_item("Open") {
                    println!("Open");
                }
                if ui.menu_item("Save") {
                    println!("Save");
                }
                if ui.menu_item("Save As") {
                    println!("Save As");
                }
                if ui.menu_item("Close") {
                    
                }
            }
            if let Some(_) = ui.begin_menu("Window") {
                ui.menu_item_config("World Objects")
                    .build_with_ref(&mut self.m_state.m_asset_window_open.borrow_mut());
                ui.menu_item_config("Game")
                    .build_with_ref(&mut self.m_state.m_game_engine_window_open.borrow_mut());
                ui.menu_item_config("File Content")
                    .build_with_ref(&mut self.m_state.m_file_content_window_open.borrow_mut());
                ui.menu_item_config("Detail")
                    .build_with_ref(&mut self.m_state.m_detail_window_open.borrow_mut());
            }
        }
    }

    fn show_editor_world_objects_window(&mut self, ui: &imgui::Ui) {
        // if CollapsingHeader::new("World Objects").build(ui) {

        // }
    }

    fn show_editor_game_window(&mut self, ui: &imgui::Ui) {

    }

    fn show_editor_file_context_window(&mut self, ui: &imgui::Ui) {

    }

    fn show_editor_detail_window(&mut self, ui: &imgui::Ui) { 

    }
}