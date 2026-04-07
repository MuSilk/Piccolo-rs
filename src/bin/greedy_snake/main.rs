use std::cell::RefCell;

use rand::Rng;
use runtime::engine::Engine;
use runtime::function::framework::object::object_id_allocator::GObjectID;
use runtime::function::framework::scene::scene::Scene as EngineScene;
use runtime::{
    app::App,
    core::math::{quaternion::Quaternion, transform::Transform, vector3::Vector3},
    function::{
        framework::{
            component::{
                camera_component::CameraComponent,
                component::{Component, ComponentTrait},
                mesh::mesh_component::MeshComponent,
                transform_component::TransformComponent,
            },
            scene::scene::SceneTrait,
        },
        input::input_system::GameCommand,
    },
};

const GRID_SIZE: i32 = 21;
const CELL_SIZE: f32 = 1.0;
const SNAKE_Z: f32 = 0.0;
const MOVE_SPEED: f32 = 6.0;
const MAX_LOGIC_STEPS_PER_TICK: usize = 8;
const HIDDEN_Z: f32 = -1000.0;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash)]
struct GridPos {
    x: i32,
    y: i32,
}

impl GridPos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn to_world(self) -> Vector3 {
        Vector3::new(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE, SNAKE_Z)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
enum Direction {
    Forward,
    Backward,
    Left,
    #[default]
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    fn to_offset(self) -> GridPos {
        match self {
            Direction::Forward => GridPos::new(0, 1),
            Direction::Backward => GridPos::new(0, -1),
            Direction::Left => GridPos::new(-1, 0),
            Direction::Right => GridPos::new(1, 0),
        }
    }
}

struct SnakeState {
    accumulator: f32,
    direction: Direction,
    pending_direction: Direction,
    cells: Vec<GridPos>,
    head_id: GObjectID,
    body_ids: Vec<GObjectID>,
    food_id: GObjectID,
    food_cell: GridPos,
}

impl SnakeState {
    fn new(head_id: GObjectID, body_ids: Vec<GObjectID>, food_id: GObjectID) -> Self {
        let mut state = Self {
            accumulator: 0.0,
            direction: Direction::Right,
            pending_direction: Direction::Right,
            cells: vec![],
            head_id,
            body_ids,
            food_id,
            food_cell: GridPos::new(0, 0),
        };
        state.reset_snake();
        state
    }

    fn reset_snake(&mut self) {
        let center = GridPos::new(GRID_SIZE / 2, GRID_SIZE / 2);
        self.direction = Direction::Right;
        self.pending_direction = Direction::Right;
        self.accumulator = 0.0;
        self.cells = vec![
            center,
            GridPos::new(center.x - 1, center.y),
            GridPos::new(center.x - 2, center.y),
        ];
    }
}

struct Scene {
    scene: EngineScene,
}

impl Scene {
    fn new() -> Self {
        let mut scene = EngineScene::new();
        scene.set_url("greedy_snake");
        Self { scene }
    }
}

impl SceneTrait for Scene {
    fn load(&mut self, engine: &Engine) {
        setup(self, engine);
        self.scene.set_loaded(true);
    }

    fn save(&self) {}

    fn tick(&mut self, engine: &Engine, delta_time: f32) {
        if !self.is_loaded() {
            return;
        }

        process_input(self, engine);
        update(self, engine, delta_time);

        let input_system = engine.m_runtime_context.input_system();
        let input_system = input_system.borrow();
        let render_system = engine.m_runtime_context.render_system();
        let render_system = render_system.borrow();

        self.scene.query_mut::<CameraComponent>().for_each(|mut camera| {
            camera.tick_free_camera(&input_system, &render_system, delta_time);
        });
        self.scene.tick_transform_components();
        self.scene.tick_mesh_components(&render_system);
    }

    fn get_url(&self) -> String {
        self.scene.get_url().clone()
    }

    fn is_loaded(&self) -> bool {
        self.scene.is_loaded()
    }
}

fn setup(scene: &mut Scene, engine: &Engine) {
    spawn_camera(&mut scene.scene);
    spawn_ground(&mut scene.scene, engine);

    let head_id = spawn_head_entity(&mut scene.scene, engine);
    let body_ids = vec![
        spawn_segment_entity(&mut scene.scene, engine, 0),
        spawn_segment_entity(&mut scene.scene, engine, 1),
    ];
    let food_id = spawn_food_entity(&mut scene.scene, engine);
    let mut state = SnakeState::new(head_id, body_ids, food_id);
    state.food_cell = random_free_cell(&state.cells);

    scene.scene.add_resource(state);
    sync_entity_transforms(scene);
}

fn process_input(scene: &mut Scene, engine: &Engine) {
    let input_system = 
        engine.m_runtime_context.input_system().borrow();
    let command = input_system.get_game_command();

    let wanted = if command.contains(GameCommand::forward) {
        Some(Direction::Forward)
    } else if command.contains(GameCommand::backward) {
        Some(Direction::Backward)
    } else if command.contains(GameCommand::left) {
        Some(Direction::Left)
    } else if command.contains(GameCommand::right) {
        Some(Direction::Right)
    } else {
        None
    };

    if let Some(wanted_dir) = wanted {
        let state = scene.scene.get_mut_resource::<SnakeState>().unwrap();
        if wanted_dir != state.direction.opposite() {
            state.pending_direction = wanted_dir;
        }
    }
}

fn update(scene: &mut Scene, engine: &Engine, delta_time: f32) {
    let step = 1.0 / MOVE_SPEED;
    let mut steps = 0usize;

    {
        let state = scene.scene.get_mut_resource::<SnakeState>().unwrap();
        state.accumulator += delta_time;
    }

    loop {
        let should_step = {
            let state = scene.scene.get_resource::<SnakeState>().unwrap();
            state.accumulator >= step && steps < MAX_LOGIC_STEPS_PER_TICK
        };
        if !should_step {
            break;
        }

        snake_step(scene, engine);
        sync_entity_transforms(scene);
        let state = scene.scene.get_mut_resource::<SnakeState>().unwrap();
        state.accumulator -= step;
        steps += 1;
    }
}

fn snake_step(scene: &mut Scene, engine: &Engine) {
    let mut ate_food = false;
    let mut need_reset = false;

    {
        let state = scene.scene.get_mut_resource::<SnakeState>().unwrap();
        state.direction = state.pending_direction;
        let offset = state.direction.to_offset();
        let head = state.cells[0];
        let next = GridPos::new(head.x + offset.x, head.y + offset.y);

        let out_of_bounds = next.x < 0 || next.x >= GRID_SIZE || next.y < 0 || next.y >= GRID_SIZE;
        let self_hit = state.cells.iter().skip(1).any(|&cell| cell == next);
        if out_of_bounds || self_hit {
            need_reset = true;
        } else {
            state.cells.insert(0, next);
            if next == state.food_cell {
                ate_food = true;
                state.food_cell = random_free_cell(&state.cells);
            } else {
                state.cells.pop();
            }
        }
    }

    if need_reset {
        let state = scene.scene.get_mut_resource::<SnakeState>().unwrap();
        state.reset_snake();
        state.food_cell = random_free_cell(&state.cells);
        return;
    }

    if ate_food {
        loop {
            let need_more = {
                let state = scene.scene.get_resource::<SnakeState>().unwrap();
                state.cells.len().saturating_sub(1) > state.body_ids.len()
            };
            if !need_more {
                break;
            }
            let next_pool_index = {
                let state = scene.scene.get_resource::<SnakeState>().unwrap();
                state.body_ids.len()
            };
            let new_segment_id = spawn_segment_entity(&mut scene.scene, engine, next_pool_index);
            let state = scene.scene.get_mut_resource::<SnakeState>().unwrap();
            state.body_ids.push(new_segment_id);
        }
    }
}

fn sync_entity_transforms(scene: &mut Scene) {
    let (cells, food_cell) = {
        let state = scene.scene.get_resource::<SnakeState>().unwrap();
        (state.cells.clone(), state.food_cell)
    };

    if let Some((_, mut head_transform)) = scene
        .scene
        .query_pair_mut::<SnakeHead, TransformComponent>()
        .next()
    {
        head_transform.set_position(cells[0].to_world());
    }

    if let Some((_, mut food_transform)) = scene.scene.query_pair_mut::<Food, TransformComponent>().next() {
        food_transform.set_position(food_cell.to_world());
    }

    scene
        .scene
        .query_pair_mut::<SnakeSegment, TransformComponent>()
        .for_each(|(segment, mut transform)| {
            if let Some(cell) = cells.get(segment.pool_index + 1) {
                transform.set_position(cell.to_world());
            } else {
                transform.set_position(Vector3::new(0.0, 0.0, HIDDEN_Z));
            }
        });
}

fn random_free_cell(occupied: &[GridPos]) -> GridPos {
    let mut rng = rand::rng();
    for _ in 0..128 {
        let x = rng.random_range(0..GRID_SIZE);
        let y = rng.random_range(0..GRID_SIZE);
        let candidate = GridPos::new(x, y);
        if !occupied.contains(&candidate) {
            return candidate;
        }
    }

    // 极端情况下线性扫描，确保一定返回可用格子
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let candidate = GridPos::new(x, y);
            if !occupied.contains(&candidate) {
                return candidate;
            }
        }
    }

    GridPos::new(0, 0)
}

fn spawn_camera(scene: &mut EngineScene) {
    let object = scene.spawn();
    let mut camera = Box::new(CameraComponent::new_free_camera());
    camera.look_at(
        Vector3::new((GRID_SIZE as f32 * CELL_SIZE) / 2.0, -GRID_SIZE as f32 * CELL_SIZE * 0.5, 10.0),
        &Vector3::new((GRID_SIZE as f32 * CELL_SIZE) / 2.0, GRID_SIZE as f32 * CELL_SIZE * 0.5, 0.0),
        &Vector3::UNIT_Z,
    );
    scene.create_object(object, vec![RefCell::new(camera as Box<dyn ComponentTrait>)]);
}

fn spawn_ground(scene: &mut EngineScene, engine: &Engine) {
    let object = scene.spawn();
    let mut ground = Box::new(MeshComponent::default());
    let asset_manager = engine.m_runtime_context.asset_manager().borrow();
    let config_manager = engine.m_runtime_context.config_manager().borrow();
    let mesh_res = asset_manager
        .load_asset(
            &config_manager,
            "asset/greedy_snake/ground.json"
        )
        .unwrap();
    ground.post_load_resource(&asset_manager, &config_manager, &mesh_res);

    let mut transform = Box::new(TransformComponent::default());
    transform.post_load_resource(Transform::new(
        Vector3::new(0.0, 0.0, 0.0),
        Quaternion::identity(),
        Vector3::ONES * (GRID_SIZE as f32 * CELL_SIZE),
    ));

    scene.create_object(
        object,
        vec![
            RefCell::new(ground as Box<dyn ComponentTrait>),
            RefCell::new(transform as Box<dyn ComponentTrait>),
        ],
    );
}

fn spawn_head_entity(scene: &mut EngineScene, engine: &Engine) -> GObjectID {
    let object = scene.spawn();
    let head = Box::new(SnakeHead::default());
    let mut mesh = Box::new(MeshComponent::default());
    let asset_manager = engine.m_runtime_context.asset_manager().borrow();
    let config_manager = engine.m_runtime_context.config_manager().borrow();
    let mesh_res = asset_manager
        .load_asset(
            &config_manager,
            "asset/greedy_snake/head.json"
        )
        .unwrap();
    mesh.post_load_resource(&asset_manager, &config_manager, &mesh_res);

    let mut transform = Box::new(TransformComponent::default());
    transform.post_load_resource(Transform::new(
        Vector3::new(0.0, 0.0, 0.0),
        Quaternion::identity(),
        Vector3::ONES * CELL_SIZE,
    ));

    scene.create_object(
        object,
        vec![
            RefCell::new(head as Box<dyn ComponentTrait>),
            RefCell::new(mesh as Box<dyn ComponentTrait>),
            RefCell::new(transform as Box<dyn ComponentTrait>),
        ],
    );
    object
}

fn spawn_segment_entity(scene: &mut EngineScene, engine: &Engine, pool_index: usize) -> GObjectID {
    let object = scene.spawn();
    let segment = Box::new(SnakeSegment {
        component: Component::default(),
        pool_index,
    });
    let mut mesh = Box::new(MeshComponent::default());
    let asset_manager = engine.m_runtime_context.asset_manager().borrow();
    let config_manager = engine.m_runtime_context.config_manager().borrow();
    let mesh_res = asset_manager
        .load_asset(
            &config_manager,
            "asset/greedy_snake/head.json"
        )
        .unwrap();
    mesh.post_load_resource(&asset_manager, &config_manager, &mesh_res);

    let mut transform = Box::new(TransformComponent::default());
    transform.post_load_resource(Transform::new(
        Vector3::new(0.0, 0.0, 0.0),
        Quaternion::identity(),
        Vector3::ONES * CELL_SIZE,
    ));

    scene.create_object(
        object,
        vec![
            RefCell::new(segment as Box<dyn ComponentTrait>),
            RefCell::new(mesh as Box<dyn ComponentTrait>),
            RefCell::new(transform as Box<dyn ComponentTrait>),
        ],
    );
    object
}

fn spawn_food_entity(scene: &mut EngineScene, engine: &Engine) -> GObjectID {
    let object = scene.spawn();
    let food = Box::new(Food::default());
    let mut mesh = Box::new(MeshComponent::default());
    let asset_manager = engine.m_runtime_context.asset_manager().borrow();
    let config_manager = engine.m_runtime_context.config_manager().borrow();
    let mesh_res = asset_manager
        .load_asset(
            &config_manager,
            "asset/greedy_snake/head.json"
        )
        .unwrap();
    mesh.post_load_resource(&asset_manager, &config_manager, &mesh_res);

    let mut transform = Box::new(TransformComponent::default());
    transform.post_load_resource(Transform::new(
        Vector3::new(0.0, 0.0, 0.0),
        Quaternion::identity(),
        Vector3::ONES * CELL_SIZE,
    ));

    scene.create_object(
        object,
        vec![
            RefCell::new(food as Box<dyn ComponentTrait>),
            RefCell::new(mesh as Box<dyn ComponentTrait>),
            RefCell::new(transform as Box<dyn ComponentTrait>),
        ],
    );
    object
}

#[derive(Default)]
struct SnakeSegment {
    component: Component,
    pool_index: usize,
}

impl ComponentTrait for SnakeSegment {
    fn get_component(&self) -> &Component {
        &self.component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default)]
struct Food {
    component: Component,
}

impl ComponentTrait for Food {
    fn get_component(&self) -> &Component {
        &self.component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default)]
struct SnakeHead {
    component: Component,
}

impl ComponentTrait for SnakeHead {
    fn get_component(&self) -> &Component {
        &self.component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() {
    let mut app = App::new();
    app.add_scene(Scene::new());
    app.set_default_scene("greedy_snake");
    app.run();
}