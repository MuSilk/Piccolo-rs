use std::{cell::RefCell, rc::{Rc}};

use runtime::{core::math::vector2::Vector2, engine::Engine, function::{render::{render_camera::RenderCameraType, render_swap_context::CameraSwapData}, ui::{ui2::{UiPanel, UiPanelFlags, UiRuntime}, window_ui::{WindowUI, WindowUIInitInfo}}}};

use crate::editor_input_manager::EditorInputManager;
use crate::editor_scene_manager::EditorSceneManager;
use crate::render_graph::graph_ui::RenderGraphUI;

pub struct EditorUI {
    m_state: RefCell<State>,
    m_render_graph_ui: RefCell<RenderGraphUI>,
    m_input_manager: Option<Rc<RefCell<EditorInputManager>>>,
    m_scene_manager: Option<Rc<RefCell<EditorSceneManager>>>,
}

impl Default for EditorUI {
    fn default() -> Self {
        Self {
            m_state: RefCell::new(State::default()),
            m_render_graph_ui: RefCell::new(RenderGraphUI::default()),
            m_input_manager: None,
            m_scene_manager: None,
        }
    }
}

#[derive(Default)]
pub struct State {
    m_editor_menu_window_open: bool,
    m_asset_window_open: bool,
    m_game_engine_window_open: bool,
    m_render_graph_window_open: bool,
    m_detail_window_open:bool,
    m_scene_lights_window_open:bool,
    m_scene_lights_data_window_open:bool,
}

impl WindowUI for EditorUI  {
    fn initialize(&mut self, _init_info: WindowUIInitInfo) {
        self.m_state.replace(State{
            m_editor_menu_window_open: true,
            m_asset_window_open: true,
            m_game_engine_window_open: true,
            m_render_graph_window_open: true,
            m_detail_window_open: true,
            m_scene_lights_window_open: true,
            m_scene_lights_data_window_open: true,
        });
    }

    fn pre_render(
        &self,
        engine: &Engine,
    ) {
        let window_system = engine.window_system().borrow();
        self.show_editor_ui(engine);
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
    ) {
        self.m_input_manager = Some(input_manager);
        self.m_scene_manager = Some(scene_manager);
    }

    fn show_editor_ui(
        &self, 
        engine: &Engine,
    ) {
        let is_editor_mode = engine.is_editor_mode();
        let mut menu_open = self.m_state.borrow().m_editor_menu_window_open;

        let mut switch_to_game = false;
        let mut switch_to_editor = false;

        {
            let mut ui_runtime = engine.ui_runtime().borrow_mut();
            let viewport = ui_runtime.get_viewport();
            let mut context_offset = [0.0, 30.0];
            let mut context_size = [viewport[0], (viewport[1] - 30.0).max(0.0)];

            let detail_open = self.m_state.borrow().m_detail_window_open;
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

            let render_graph_open = self.m_state.borrow().m_render_graph_window_open;
            if render_graph_open {
                let graph_h = context_size[1] * 0.38;
                let graph_panel = ui_runtime.panel(
                    "render_graph_panel",
                    "Render Graph",
                    [context_offset[0], context_offset[1] + context_size[1] - graph_h],
                    [context_size[0], graph_h],
                    UiPanelFlags::default(),
                );
                self.show_render_graph_window(&mut ui_runtime, &graph_panel);
                context_size[1] -= graph_h;
            }

            let world_open = self.m_state.borrow().m_asset_window_open;
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

            let game_open = self.m_state.borrow().m_game_engine_window_open;
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

        self.m_state.borrow_mut().m_editor_menu_window_open = menu_open;

        if switch_to_game {
            engine.set_editor_mode(true);
            engine.input_system().borrow_mut().reset_game_command();
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
            let render_system = engine.render_system().borrow();
            let swap_context = render_system.get_swap_context();
            swap_context.get_logic_swap_data().borrow_mut().m_camera_swap_data = Some(CameraSwapData {
                m_fov_x: None,
                m_camera_type: Some(RenderCameraType::Editor),
                m_view_matrix: Some(view_matrix),
            });
            engine.window_system().borrow().set_focus_mode(false);
        }

        if switch_to_editor {
            engine.set_editor_mode(false);
            if let Some(im) = self.m_input_manager.as_ref() {
                im.borrow_mut().reset_editor_command();
            }
            engine.window_system().borrow().set_focus_mode(true);
        }
    }

    fn show_editor_menu(&self, ui_runtime: &mut UiRuntime, p_open: &mut bool) {
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
                    .build_with_ref(&mut self.m_state.borrow_mut().m_asset_window_open);
                ui_runtime
                    .menu_item_config("Game")
                    .build_with_ref(&mut self.m_state.borrow_mut().m_game_engine_window_open);
                ui_runtime
                    .menu_item_config("Render Graph")
                    .build_with_ref(&mut self.m_state.borrow_mut().m_render_graph_window_open);
                ui_runtime
                    .menu_item_config("Detail")
                    .build_with_ref(&mut self.m_state.borrow_mut().m_detail_window_open);
            }

            ui_runtime.end_main_menu_bar();
        }
    }

    fn show_editor_world_objects_window(&self, ui_runtime: &mut UiRuntime, panel: &UiPanel) {
        ui_runtime.push_text_ascii(
            "World objects list (todo)",
            [panel.body_pos[0] + 6.0, panel.body_pos[1] + 6.0],
            [8.0, 14.0],
            [220, 225, 235, 255],
            panel.clip_rect,
        );
    }

    fn show_editor_game_window(
        &self, 
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

    fn show_render_graph_window(&self, ui_runtime: &mut UiRuntime, panel: &UiPanel) {
        self.m_render_graph_ui.borrow_mut().draw(ui_runtime, panel);
    }

    fn show_editor_detail_window(&self, ui_runtime: &mut UiRuntime, panel: &UiPanel) {
        ui_runtime.push_text_ascii(
            "Component details (todo)",
            [panel.body_pos[0] + 6.0, panel.body_pos[1] + 6.0],
            [8.0, 14.0],
            [220, 225, 235, 255],
            panel.clip_rect,
        );
    }
}