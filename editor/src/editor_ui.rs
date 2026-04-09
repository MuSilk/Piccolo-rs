use std::{cell::RefCell, rc::{Rc, Weak}};

use runtime::{core::math::vector2::Vector2, engine::Engine, function::{input::input_system::InputSystem, render::{render_camera::RenderCameraType, render_swap_context::CameraSwapData, render_system::RenderSystem, window_system::WindowSystem}, ui::{ui2::{UiPanel, UiPanelFlags, UiRuntime}, window_ui::{WindowUI, WindowUIInitInfo}}}};

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
    fn initialize(&mut self, init_info: WindowUIInitInfo) {
        self.m_engine = Rc::downgrade(init_info.engine);
        self.m_state.m_editor_menu_window_open.replace(true);
    }

    fn pre_render(
        &mut self, 
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem, 
        input_system: &RefCell<InputSystem>,
    ) {
        self.show_editor_ui(render_system, window_system, input_system);
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
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem,
        input_system: &RefCell<InputSystem>,
    ) {
        let engine = self.m_engine.upgrade().unwrap();
        let is_editor_mode = engine.borrow().is_editor_mode();
        let mut menu_open = *self.m_state.m_editor_menu_window_open.borrow();

        let mut switch_to_game = false;
        let mut switch_to_editor = false;

        {
            let t_engine = engine.borrow();
            let mut ui_runtime = t_engine.m_runtime_context.ui_runtime().borrow_mut();
            let viewport = ui_runtime.get_viewport();
            let mut context_offset = [0.0, 30.0];
            let mut context_size = [viewport[0], (viewport[1] - 30.0).max(0.0)];

            let detail_open = *self.m_state.m_detail_window_open.borrow();
            if detail_open {
                let detail_w = context_size[0] * 0.25;
                let detail_panel = ui_runtime.panel(
                    "detail_panel",
                    "Components Details",
                    [context_offset[0] + context_size[0] - detail_w, context_offset[1]],
                    [detail_w, context_size[1]],
                    UiPanelFlags::default(),
                );
                self.show_editor_detail_window(&mut ui_runtime, &detail_panel);
                context_size[0] -= detail_w;
            }

            let file_open = *self.m_state.m_file_content_window_open.borrow();
            if file_open {
                let file_h = context_size[1] * 0.3;
                let file_panel = ui_runtime.panel(
                    "file_panel",
                    "File Content",
                    [context_offset[0], context_offset[1] + context_size[1] - file_h],
                    [context_size[0], file_h],
                    UiPanelFlags::default(),
                );
                self.show_editor_file_context_window(&mut ui_runtime, &file_panel);
                context_size[1] -= file_h;
            }

            let world_open = *self.m_state.m_asset_window_open.borrow();
            if world_open {
                let world_w = context_size[0] * 0.3;
                let world_panel = ui_runtime.panel(
                    "world_panel",
                    "World Object",
                    [context_offset[0], context_offset[1]],
                    [world_w, context_size[1]],
                    UiPanelFlags::default(),
                );
                self.show_editor_world_objects_window(&mut ui_runtime, &world_panel);
                context_offset[0] += world_w;
                context_size[0] -= world_w;
            }

            let game_open = *self.m_state.m_game_engine_window_open.borrow();
            if game_open {
                let game_panel = ui_runtime.panel(
                    "game_panel",
                    "Game Engine",
                    [context_offset[0], context_offset[1]],
                    [context_size[0], context_size[1]],
                    UiPanelFlags::HEADER_BG | UiPanelFlags::BORDER,
                );
                let (to_game, to_editor) = self.show_editor_game_window(
                    &mut ui_runtime,
                    is_editor_mode,
                    &game_panel,
                );
                switch_to_game = to_game;
                switch_to_editor = to_editor;
            }

            // Draw menu last so popup stays on top of dock windows.
            self.show_editor_menu(&mut ui_runtime, &mut menu_open);
        }

        *self.m_state.m_editor_menu_window_open.borrow_mut() = menu_open;

        if switch_to_game {
            engine.borrow().set_editor_mode(true);
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
            swap_context.get_logic_swap_data().borrow_mut().m_camera_swap_data = Some(CameraSwapData {
                m_fov_x: None,
                m_camera_type: Some(RenderCameraType::Editor),
                m_view_matrix: Some(view_matrix),
            });
            window_system.set_focus_mode(false);
        }

        if switch_to_editor {
            engine.borrow().set_editor_mode(false);
            if let Some(im) = self.m_input_manager.as_ref() {
                im.borrow_mut().reset_editor_command();
            }
            window_system.set_focus_mode(true);
        }
    }

    fn show_editor_menu(&mut self, ui_runtime: &mut UiRuntime, p_open: &mut bool) {
        if !*p_open {
            return;
        }
        if ui_runtime.begin_main_menu_bar() {
            if ui_runtime.begin_menu("Menu") {
                if ui_runtime.menu_item("Open") {
                    println!("Open");
                }
                if ui_runtime.menu_item("Save") {
                    println!("Save");
                }
                if ui_runtime.menu_item("Save As") {
                    println!("Save As");
                }
                if ui_runtime.menu_item("Close") {
                    *p_open = false;
                }
            }

            if ui_runtime.begin_menu("Window") {
                ui_runtime
                    .menu_item_config("World Objects")
                    .build_with_ref(&mut self.m_state.m_asset_window_open.borrow_mut());
                ui_runtime
                    .menu_item_config("Game")
                    .build_with_ref(&mut self.m_state.m_game_engine_window_open.borrow_mut());
                ui_runtime
                    .menu_item_config("File Content")
                    .build_with_ref(&mut self.m_state.m_file_content_window_open.borrow_mut());
                ui_runtime
                    .menu_item_config("Detail")
                    .build_with_ref(&mut self.m_state.m_detail_window_open.borrow_mut());
            }

            ui_runtime.end_main_menu_bar();
        }
    }

    fn show_editor_world_objects_window(&mut self, ui_runtime: &mut UiRuntime, panel: &UiPanel) {
        ui_runtime.push_text_ascii(
            "World objects list (todo)",
            [panel.body_pos[0] + 6.0, panel.body_pos[1] + 6.0],
            [8.0, 14.0],
            [220, 225, 235, 255],
            panel.clip_rect,
        );
    }

    fn show_editor_game_window(
        &mut self, 
        ui_runtime: &mut UiRuntime,
        is_editor_mode: bool,
        panel: &UiPanel,
    ) -> (bool, bool) {
        let btn_pos = [panel.body_pos[0] + 8.0, panel.body_pos[1] + 8.0];
        let btn_size = [180.0, 38.0];
        if is_editor_mode {
            let resp = ui_runtime.button_in_clip(
                "EditorModeBtn",
                "Editor Mode",
                btn_pos,
                btn_size,
                panel.clip_rect,
            );
            (false, resp.clicked)
        } else {
            let resp = ui_runtime.button_in_clip(
                "GameModeBtn",
                "Game Mode",
                btn_pos,
                btn_size,
                panel.clip_rect,
            );
            (resp.clicked, false)
        }
    }

    fn show_editor_file_context_window(&mut self, ui_runtime: &mut UiRuntime, panel: &UiPanel) {
        ui_runtime.push_text_ascii(
            "File content window (todo)",
            [panel.body_pos[0] + 6.0, panel.body_pos[1] + 6.0],
            [8.0, 14.0],
            [220, 225, 235, 255],
            panel.clip_rect,
        );
    }

    fn show_editor_detail_window(&mut self, ui_runtime: &mut UiRuntime, panel: &UiPanel) {
        ui_runtime.push_text_ascii(
            "Component details (todo)",
            [panel.body_pos[0] + 6.0, panel.body_pos[1] + 6.0],
            [8.0, 14.0],
            [220, 225, 235, 255],
            panel.clip_rect,
        );
    }
}