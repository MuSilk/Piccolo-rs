use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    core::math::bounding_box::{BoundingBox, bounding_box_transform},
    function::{
        framework::object::object_id_allocator::GObjectID,
        render::{
            light::{AmbientLight, DirectionalLight, PointLightList},
            render_camera::RenderCamera,
            render_common::RenderMeshNode,
            render_entity::RenderEntity,
            render_guid_allocator::GuidAllocator,
            render_object::GameObjectPartId,
            render_pass::RenderPass,
            render_resource::RenderResource,
            render_type::{MaterialSourceDesc, MeshSourceDesc},
        },
    },
};

#[derive(Default)]
pub struct RenderScene {
    pub m_ambient_light: AmbientLight,
    pub m_directional_light: DirectionalLight,
    pub m_point_light_list: PointLightList,

    m_render_entities: RefCell<HashMap<u32, Box<RenderEntity>>>,

    m_main_camera_visible_mesh_nodes: Rc<RefCell<Vec<RenderMeshNode>>>,
    m_directional_light_visible_mesh_nodes: Rc<RefCell<Vec<RenderMeshNode>>>,

    m_instance_id_allocator: RefCell<GuidAllocator<GameObjectPartId>>,
    m_mesh_asset_id_allocator: RefCell<GuidAllocator<MeshSourceDesc>>,
    m_material_asset_id_allocator: RefCell<GuidAllocator<MaterialSourceDesc>>,

    m_mesh_object_id_map: RefCell<HashMap<u32, GObjectID>>,
}

impl RenderScene {
    pub fn update_visible_objects(&self, render_resource: &RenderResource, camera: &RenderCamera) {
        self.update_visible_objects_main_camera(render_resource, camera);
        self.update_visible_objects_directional_light(render_resource, camera);
    }

    pub fn set_visible_nodes_reference(&self) {
        RenderPass::m_visible_nodes()
            .borrow_mut()
            .p_directional_light_visible_mesh_nodes =
            Rc::downgrade(&self.m_directional_light_visible_mesh_nodes);
        RenderPass::m_visible_nodes()
            .borrow_mut()
            .p_main_camera_visible_mesh_nodes =
            Rc::downgrade(&self.m_main_camera_visible_mesh_nodes);
    }

    pub fn alloc_instance_id(&self, part_id: &GameObjectPartId) -> usize {
        self.m_instance_id_allocator
            .borrow_mut()
            .alloc_guid(part_id)
    }

    pub fn get_mesh_asset_id_allocator(&self) -> &RefCell<GuidAllocator<MeshSourceDesc>> {
        &self.m_mesh_asset_id_allocator
    }

    pub fn get_material_asset_id_allocator(&self) -> &RefCell<GuidAllocator<MaterialSourceDesc>> {
        &self.m_material_asset_id_allocator
    }

    pub fn add_instance_id_to_map(&self, instance_id: u32, go_id: GObjectID) {
        self.m_mesh_object_id_map
            .borrow_mut()
            .insert(instance_id, go_id);
    }

    pub fn delete_entity_by_gobject_id(&self, go_id: GObjectID) {
        self.m_mesh_object_id_map
            .borrow_mut()
            .remove(&(go_id as u32));

        let part_id = GameObjectPartId {
            m_go_id: go_id,
            m_part_id: 0,
        };
        if let Some(instance_id) = self
            .m_instance_id_allocator
            .borrow()
            .get_element_guid(&part_id)
        {
            self.m_render_entities
                .borrow_mut()
                .remove(&(instance_id as u32));
        }
    }

    pub fn calc_scene_bounding_box(&self) -> BoundingBox {
        let mut scene_bounding_box = BoundingBox::default();

        self.m_render_entities
            .borrow()
            .iter()
            .for_each(|(_id, entity)| {
                let mesh_asset_bounding_box = BoundingBox {
                    min_bound: *entity.m_bounding_box.get_min_corner(),
                    max_bound: *entity.m_bounding_box.get_max_corner(),
                };
                let mesh_bounding_box_world =
                    bounding_box_transform(&mesh_asset_bounding_box, &entity.m_model_matrix);
                scene_bounding_box.merge_box(&mesh_bounding_box_world);
            });
        scene_bounding_box
    }

    pub fn insert_or_update_render_entity(&self, render_entity: Box<RenderEntity>) {
        let instance_id = render_entity.m_instance_id;
        if self.m_render_entities.borrow().contains_key(&instance_id) {
            *self
                .m_render_entities
                .borrow_mut()
                .get_mut(&instance_id)
                .unwrap() = render_entity;
        } else {
            self.m_render_entities
                .borrow_mut()
                .insert(instance_id, render_entity);
        }
    }
}

impl RenderScene {
    fn update_visible_objects_main_camera(
        &self,
        render_resource: &RenderResource,
        _camera: &RenderCamera,
    ) {
        let mut main_camera_visible_mesh_nodes = self.m_main_camera_visible_mesh_nodes.borrow_mut();
        main_camera_visible_mesh_nodes.clear();

        for (_instance_id, entity) in self.m_render_entities.borrow().iter() {
            let mut temp_node = RenderMeshNode::default();
            temp_node.model_matrix = entity.m_model_matrix.clone();
            temp_node.node_id = entity.m_instance_id;

            let mesh_asset = render_resource.get_entity_mesh(entity);
            temp_node.ref_mesh = Rc::downgrade(mesh_asset);
            temp_node.enable_vertex_blending = entity.m_enable_vertex_blending;

            let material_asset = render_resource.get_entity_material(entity);
            temp_node.ref_material = Rc::downgrade(material_asset);

            main_camera_visible_mesh_nodes.push(temp_node);
        }
    }

    fn update_visible_objects_directional_light(
        &self,
        render_resource: &RenderResource,
        _camera: &RenderCamera,
    ) {
        let mut directional_light_visible_mesh_nodes =
            self.m_directional_light_visible_mesh_nodes.borrow_mut();
        directional_light_visible_mesh_nodes.clear();

        for (_instance_id, entity) in self.m_render_entities.borrow().iter() {
            let mut temp_node = RenderMeshNode::default();
            temp_node.model_matrix = entity.m_model_matrix.clone();
            temp_node.node_id = entity.m_instance_id;

            let mesh_asset = render_resource.get_entity_mesh(entity);
            temp_node.ref_mesh = Rc::downgrade(mesh_asset);
            temp_node.enable_vertex_blending = entity.m_enable_vertex_blending;

            let material_asset = render_resource.get_entity_material(entity);
            temp_node.ref_material = Rc::downgrade(material_asset);

            directional_light_visible_mesh_nodes.push(temp_node);
        }
    }
}
