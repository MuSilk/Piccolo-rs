use std::{collections::HashMap, fs::File, io::{BufReader, Read}, path::PathBuf};

use image::EncodableLayout;
use log::error;
use itertools::Itertools;
use vulkanalia::prelude::v1_0::*;

use crate::{core::math::{axis_aligned::AxisAlignedBox, vector2::Vector2, vector3::Vector3}, function::{global::global_context::RuntimeGlobalContext, render::{render_object::GameObjectDynamicMeshDesc, render_type::{ImageType, MaterialSourceDesc, MeshSourceDesc, MeshVertexDataDefinition, RenderMaterialData, RenderMeshData, StaticMeshData, TextureData}}}, resource::res_type::data::mesh_data::MeshData};


#[derive(Clone, Default)]
pub struct RenderResourceBase{
    m_bounding_box_cache_map: HashMap<MeshSourceDesc, AxisAlignedBox>,
}

impl RenderResourceBase {

    pub fn load_texture_hdr(file: &str, desired_channels: u32) -> Option<TextureData> {
        let asset_manager = RuntimeGlobalContext::get_asset_manager().borrow();

        let image = image::open(asset_manager.get_full_path(file)).ok()?;
        let mut texture = TextureData::default();
        match desired_channels {
            4 => {
                let image = image.to_rgba32f();
                texture.m_pixels = image.as_bytes().to_vec();
                texture.m_width = image.width();
                texture.m_height = image.height();
                texture.m_format = vk::Format::R32G32B32A32_SFLOAT;
            },
            _ => {
                error!("Unsupported number of channels: {}", desired_channels);
                return None;
            }
        }
        texture.m_depth = 1;
        texture.m_array_layers = 1;
        texture.m_mip_levels = 1;
        texture.m_type = ImageType::_2D;

        Some(texture)
    }

    pub fn load_texture(file: &str, is_srgb: bool) -> Option<TextureData> {
        let asset_manager = RuntimeGlobalContext::get_asset_manager().borrow();

        let image = image::open(asset_manager.get_full_path(file)).ok()?;
        let image = image.to_rgba8();

        let mut texture = TextureData::default();
        texture.m_pixels = image.bytes().map(|byte| byte.unwrap()).collect::<Vec<_>>();
        texture.m_width = image.width();
        texture.m_height = image.height();
        texture.m_format = if is_srgb {
            vk::Format::R8G8B8A8_SRGB
        } else {
            vk::Format::R8G8B8A8_UNORM
        };
        texture.m_depth = 1;
        texture.m_array_layers = 1;
        texture.m_mip_levels = 1;
        texture.m_type = ImageType::_2D;

        Some(texture)
    }

    pub fn load_mesh_data(&mut self, source: &MeshSourceDesc) -> (RenderMeshData, AxisAlignedBox) {
        let mut ret: RenderMeshData = RenderMeshData::default();
        let mut bounding_box = AxisAlignedBox::default();
        if PathBuf::from(&source.m_mesh_file).extension().unwrap() == "obj" {
            (ret.m_static_mesh_data, bounding_box) = Self::load_static_mesh(&source.m_mesh_file);
        }
        else if PathBuf::from(&source.m_mesh_file).extension().unwrap() == "json" {
            let mesh_data: MeshData = {
                let asset_manager = RuntimeGlobalContext::get_asset_manager().borrow();
                asset_manager.load_asset(&source.m_mesh_file).unwrap()
            };

            let vertices = mesh_data.vertices
                .iter()
                .map(|vertex| MeshVertexDataDefinition {
                    x: vertex.px, y: vertex.py, z: vertex.pz,
                    nx: vertex.nx, ny: vertex.ny, nz: vertex.nz,
                    tx: vertex.tx, ty: vertex.ty, tz: vertex.tz,
                    u: vertex.u, v: vertex.v,
                })
                .collect_vec();
            vertices.iter().for_each(|vertice|bounding_box.merge(&Vector3::new(vertice.x, vertice.y, vertice.z)));

            ret.m_static_mesh_data.m_vertex_buffer.m_data = bytemuck::pod_collect_to_vec(&vertices);

            let indices = mesh_data.indices
                .iter()
                .map(|indice| *indice as u16)
                .collect_vec();

            ret.m_static_mesh_data.m_index_buffer.m_data = bytemuck::pod_collect_to_vec(&indices);
            ret.m_static_mesh_data.m_index_type = vk::IndexType::UINT16;
    

            //todo: skeleton bindings
        }
        else {
            panic!("Unsupported mesh format: {}", source.m_mesh_file);
        }

        self.m_bounding_box_cache_map.insert(source.clone(), bounding_box.clone());

        (ret, bounding_box)
    }

    pub fn load_mesh_data_from_raw(&mut self, source: &MeshSourceDesc, data: &GameObjectDynamicMeshDesc) -> (RenderMeshData, AxisAlignedBox) {
        let mut ret: RenderMeshData = RenderMeshData::default();
        let mut bounding_box = AxisAlignedBox::default();

        data.m_vertices.iter().for_each(|vertice| bounding_box.merge(&Vector3::new(vertice.x, vertice.y, vertice.z)));
        ret.m_static_mesh_data.m_vertex_buffer.m_data = bytemuck::pod_collect_to_vec(&data.m_vertices);
        ret.m_static_mesh_data.m_index_buffer.m_data = bytemuck::pod_collect_to_vec(&data.m_indices);
        ret.m_static_mesh_data.m_index_type = vk::IndexType::UINT32;

        //todo: skeleton bindings
        self.m_bounding_box_cache_map.insert( source.clone(), bounding_box.clone());

        (ret, bounding_box)
    }

    pub fn load_material_data(source: &MaterialSourceDesc) -> RenderMaterialData {
        let mut ret = RenderMaterialData::default();
        ret.m_base_color_texture = Self::load_texture(&source.m_base_color_file, true);
        ret.m_metallic_roughness_texture = Self::load_texture(&source.m_metallic_roughness_file, false);
        ret.m_normal_texture = Self::load_texture(&source.m_normal_file, false);
        ret.m_occlusion_texture = Self::load_texture(&source.m_occlusion_file, false);
        ret.m_emissive_texture = Self::load_texture(&source.m_emissive_file, false);
        ret
    }
    
    pub fn get_cached_bounding_box(&self, mesh_source: &MeshSourceDesc) -> Option<&AxisAlignedBox> {
        self.m_bounding_box_cache_map.get(mesh_source)
    }

    fn load_static_mesh(filename: &str) -> (StaticMeshData, AxisAlignedBox) {
        let mut bounding_box = AxisAlignedBox::default();
        let mut reader = BufReader::new(File::open(filename).unwrap_or_else(|_|{
            panic!("Failed to open mesh file: {}", filename);
        }));
        let (models, _) = tobj::load_obj_buf(&mut reader, &tobj::LoadOptions{
            triangulate: true,
            ..Default::default()
        }, |_| Ok(Default::default())).unwrap();

        let mut mesh_vertices = Vec::new();

        for model in models {
            for index in 0..model.mesh.indices.len()/3 {
                let mut with_normal = true;
                let mut with_texcoord = true;
                let mut vertex = [Vector3::default(); 3];
                let mut normal = [Vector3::default(); 3];
                let mut uv = [Vector2::default(); 3];
                for i in 0..3 {
                    vertex[i] = Vector3::new(
                        model.mesh.positions[model.mesh.indices[index * 3 + i] as usize * 3 + 0],
                        model.mesh.positions[model.mesh.indices[index * 3 + i] as usize * 3 + 1],
                        model.mesh.positions[model.mesh.indices[index * 3 + i] as usize * 3 + 2],
                    );

                    bounding_box.merge(&vertex[i]);

                    if !model.mesh.normals.is_empty() {
                        normal[i] = Vector3::new(
                            model.mesh.normals[model.mesh.normal_indices[index * 3 + i] as usize * 3 + 0],
                            model.mesh.normals[model.mesh.normal_indices[index * 3 + i] as usize * 3 + 1],
                            model.mesh.normals[model.mesh.normal_indices[index * 3 + i] as usize * 3 + 2],
                        );
                    } else {
                        with_normal = false;
                    } 

                    if !model.mesh.texcoords.is_empty() {
                        uv[i] = Vector2::new(
                            model.mesh.texcoords[model.mesh.texcoord_indices[(index + i) * 2 + 0] as usize],
                            model.mesh.texcoords[model.mesh.texcoord_indices[(index + i) * 2 + 1] as usize],
                        );
                    } else {
                        with_texcoord = false;
                    }                    
                }

                if !with_normal {
                    let v0 = vertex[1] - vertex[0];
                    let v1 = vertex[2] - vertex[1];
                    normal[0] = v0.cross(&v1).normalize();
                    normal[1] = normal[0];
                    normal[2] = normal[0];
                }

                if !with_texcoord {
                    uv[0] = Vector2::new(0.5, 0.5);
                    uv[1] = Vector2::new(0.5, 0.5);
                    uv[2] = Vector2::new(0.5, 0.5);
                }

                let mut tangent = Vector3::new(1.0, 0.0, 0.0);
                {
                    let edge1 = vertex[1] - vertex[0];
                    let edge2 = vertex[2] - vertex[0];
                    let delta_uv1 = uv[1] - uv[0];
                    let delta_uv2 = uv[2] - uv[0];

                    let mut devide = delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y;
                    if devide >= 0.0 && devide < 0.000001 {
                        devide = 0.000001;
                    }
                    else if devide < 0.0 && devide > -0.000001 {
                        devide = -0.000001;
                    }
                    let f = 1.0 / devide;
                    tangent.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
                    tangent.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
                    tangent.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);
                    tangent = tangent.normalize();
                }

                for i in 0..3  {
                    let mesh_vert = MeshVertexDataDefinition {
                        x: vertex[i].x, y: vertex[i].y, z: vertex[i].z,
                        nx: normal[i].x, ny: normal[i].y, nz: normal[i].z,
                        tx: tangent.x, ty: tangent.y, tz: tangent.z,
                        u: uv[i].x, v: uv[i].y,
                    };
                    mesh_vertices.push(mesh_vert);
                }
            }
        }
        let mut mesh_data = StaticMeshData::default();

        let mesh_indices = (0..mesh_vertices.len()).map(|i| i as u16).collect::<Vec<_>>();
        
        mesh_data.m_vertex_buffer.m_data = mesh_vertices.iter().flat_map(|v|{
            let ptr = v as *const MeshVertexDataDefinition as *const u8;
            unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<MeshVertexDataDefinition>()) }
        })
        .copied()
        .collect();
        
        mesh_data.m_index_type = vk::IndexType::UINT16;
        mesh_data.m_index_buffer.m_data = mesh_indices.iter().flat_map(|i|{
            let ptr = i as *const u16 as *const u8;
            unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<u16>()) }
        })
        .copied()
        .collect();

        (mesh_data, bounding_box)
    }
}