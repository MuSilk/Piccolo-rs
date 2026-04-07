use std::{cell::RefCell, rc::{Rc, Weak}};

use imgui::{Condition, WindowFlags};
use runtime::{core::math::vector2::Vector2, engine::Engine, function::{input::input_system::InputSystem, render::{render_camera::RenderCameraType, render_swap_context::CameraSwapData, render_system::RenderSystem, window_system::WindowSystem}, ui::window_ui::{WindowUI, WindowUIInitInfo}}};

use crate::editor_input_manager::EditorInputManager;
use crate::editor_scene_manager::EditorSceneManager;

pub struct EditorUI {
    m_state: State,
    m_input_manager: Option<Rc<RefCell<EditorInputManager>>>,
    m_scene_manager: Option<Rc<RefCell<EditorSceneManager>>>,
    m_engine: Weak<RefCell<Engine>>,
}

impl Default for EditorUI {
    fn default() -> Self {
        Self {
            m_state: State::default(),
            m_input_manager: None,
            m_scene_manager: None,
            m_engine: Weak::new(),
        }
    }
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

    fn pre_render(
        &mut self, 
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem, 
        input_system: &RefCell<InputSystem>,
        ui: &mut imgui::Ui
    ) {
        self.show_editor_ui(ui, render_system, window_system, input_system);
        let window_size = window_system.get_window_size();
        if let Some(input_manager) = self.m_input_manager.as_ref() {
            input_manager.borrow_mut().set_engine_window_size(Vector2::new(
                window_size.0 as f32,
                window_size.1 as f32,
            ));
        }
    }
}

impl EditorUI {
    pub fn set_editor_handles(
        &mut self,
        input_manager: Rc<RefCell<EditorInputManager>>,
        scene_manager: Rc<RefCell<EditorSceneManager>>,
        engine: &Rc<RefCell<Engine>>,
    ) {
        self.m_input_manager = Some(input_manager);
        self.m_scene_manager = Some(scene_manager);
        self.m_engine = Rc::downgrade(engine);
    }

    fn show_editor_ui(
        &mut self, 
        ui: &mut imgui::Ui,
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem,
        input_system: &RefCell<InputSystem>,
    ) {

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

        let world_object_window = ui
            .window("World Object")
            .position([context_offset[0]  , context_offset[1]], Condition::Always)
            .size([0.3 * context_size[0],  context_size[1]], Condition::Always)
            .flags(WindowFlags::NO_COLLAPSE);
        context_offset[0] += 0.3 * context_size[0];
        context_size[0] -= 0.3 * context_size[0];

        let editor_game_window  = ui
            .window("Game Engine")
            .position([context_offset[0]  , context_offset[1]], Condition::Always)
            .size([context_size[0],  context_size[1]], Condition::Always)
            .flags(WindowFlags::NO_COLLAPSE | WindowFlags::NO_BACKGROUND | WindowFlags::MENU_BAR
        );

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
        editor_game_window.build(||{
            self.show_editor_game_window(render_system, window_system, input_system, ui);
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

    fn show_editor_game_window(
        &mut self, 
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem, 
        input_system: &RefCell<InputSystem>,
        ui: &imgui::Ui
    ) {
        if let Some(_) =  ui.begin_menu_bar() {
            let Some(engine_rc) = self.m_engine.upgrade() else {
                return;
            };
            if engine_rc.borrow().is_editor_mode() {
                if ui.button("Editor Mode") {
                    engine_rc.borrow_mut().set_editor_mode(false);
                    if let Some(im) = self.m_input_manager.as_ref() {
                        im.borrow_mut().reset_editor_command();
                    }
                    window_system.set_focus_mode(true);
                }
            } else{
                if ui.button("Game Mode") {
                    engine_rc.borrow_mut().set_editor_mode(true);
                    input_system.borrow_mut().reset_game_command();
                    let view_matrix = {
                        let sm = self
                            .m_scene_manager
                            .as_ref()
                            .expect("editor scene_manager not wired");
                        let editor_camera = sm
                            .borrow()
                            .get_editor_camera()
                            .upgrade()
                            .unwrap();
                        editor_camera.borrow().get_view_matrix()
                    };
                    let render_system = render_system.borrow_mut();
                    let swap_context = render_system.get_swap_context();
                    swap_context.get_logic_swap_data().borrow_mut().m_camera_swap_data = Some(CameraSwapData{
                        m_fov_x: None,
                        m_camera_type: Some(RenderCameraType::Editor),
                        m_view_matrix: Some(view_matrix)
                    });
                    window_system.set_focus_mode(false);
                }
            }
        } 
    }

    fn show_editor_file_context_window(&mut self, ui: &imgui::Ui) {

    }

    fn show_editor_detail_window(&mut self, ui: &imgui::Ui) {
        
    }
}