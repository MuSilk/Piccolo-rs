use std::sync::Mutex;

use nalgebra_glm::{quat_to_mat4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::runtime::function::{render::debugdraw::{debug_draw_font::DebugDrawFont, debug_draw_primitive::{DebugDrawBox, DebugDrawCapsule, DebugDrawCylinder, DebugDrawLine, DebugDrawPoint, DebugDrawQuad, DebugDrawSphere, DebugDrawText, DebugDrawTriangle, DebugDrawVertex, FillMode}}};

#[derive(Default)]
pub struct DebugDrawGroup {
    m_mutex: Mutex<()>,
    pub m_name: String,

    m_points: Vec<DebugDrawPoint>,
    m_lines: Vec<DebugDrawLine>,
    pub m_triangles: Vec<DebugDrawTriangle>,
    m_quads: Vec<DebugDrawQuad>,
    m_boxes: Vec<DebugDrawBox>,
    m_cylinders: Vec<DebugDrawCylinder>,
    m_spheres: Vec<DebugDrawSphere>,
    m_capsules: Vec<DebugDrawCapsule>,
    m_texts: Vec<DebugDrawText>,
}

impl DebugDrawGroup {
    pub fn initialize(&self) { }

    pub fn clear(&mut self) {
        let _guard = self.m_mutex.lock();
        self.m_points.clear();
        self.m_lines.clear();
        self.m_triangles.clear();
        self.m_quads.clear();
        self.m_boxes.clear();
        self.m_cylinders.clear();
        self.m_spheres.clear();
        self.m_capsules.clear();
        self.m_texts.clear();
    }

    pub fn set_name(&mut self, name: &str) {
        let _guard = self.m_mutex.lock();
        self.m_name = name.to_string();
    }

    pub fn get_name(&self) -> &String {
        let _guard = self.m_mutex.lock();
        &self.m_name
    }

    pub fn add_point(&mut self, position: &Vec3, color: &Vec4, life_time: f32, no_depth_test: bool) {
        let _guard = self.m_mutex.lock();
        let mut point = DebugDrawPoint::default();
        point.m_base.set_time(life_time);
        point.m_base.m_fill_mode = FillMode::WireFrame;
        point.m_base.m_no_depth_test = no_depth_test;

        point.m_vertex.pos = *position;
        point.m_vertex.color = *color;
        self.m_points.push(point);
    }

    pub fn add_line(&mut self, point0: &Vec3, point1: &Vec3, color0: &Vec4, color1: &Vec4, life_time: f32, no_depth_test: bool) {
        let _guard = self.m_mutex.lock();
        let mut line = DebugDrawLine::default();
        line.m_base.set_time(life_time);
        line.m_base.m_fill_mode = FillMode::WireFrame;
        line.m_base.m_no_depth_test = no_depth_test;

        line.m_vertex[0].pos = *point0;
        line.m_vertex[0].color = *color0;
        line.m_vertex[1].pos = *point1;
        line.m_vertex[1].color = *color1;
        self.m_lines.push(line);
    }

    pub fn add_triangle(&mut self, 
        point0: &Vec3, point1: &Vec3, point2: &Vec3,
        color0: &Vec4, color1: &Vec4, color2: &Vec4, 
        life_time: f32, no_depth_test: bool, fill_mode: FillMode,
    ) {
        let _guard = self.m_mutex.lock();
        let mut triangle = DebugDrawTriangle::default();
        triangle.m_base.set_time(life_time);
        triangle.m_base.m_fill_mode = fill_mode;
        triangle.m_base.m_no_depth_test = no_depth_test;

        triangle.m_vertex[0].pos = *point0;
        triangle.m_vertex[0].color = *color0;
        triangle.m_vertex[1].pos = *point1;
        triangle.m_vertex[1].color = *color1;
        triangle.m_vertex[2].pos = *point2;
        triangle.m_vertex[2].color = *color2;
        self.m_triangles.push(triangle);
    }

    pub fn add_quad(&mut self, 
        point0: &Vec3, point1: &Vec3, point2: &Vec3, point3: &Vec3,
        color0: &Vec4, color1: &Vec4, color2: &Vec4, color3: &Vec4,
        life_time: f32, no_depth_test: bool, fill_mode: FillMode,
    ) {
        let _guard = self.m_mutex.lock();
        match fill_mode {
            FillMode::WireFrame => {
                let mut quad = DebugDrawQuad::default();
                quad.m_base.set_time(life_time);
                quad.m_base.m_fill_mode = FillMode::WireFrame;
                quad.m_base.m_no_depth_test = no_depth_test;

                quad.m_vertex[0].pos = *point0;
                quad.m_vertex[0].color = *color0;
                quad.m_vertex[1].pos = *point1;
                quad.m_vertex[1].color = *color1;
                quad.m_vertex[2].pos = *point2;
                quad.m_vertex[2].color = *color2;
                quad.m_vertex[3].pos = *point3;
                quad.m_vertex[3].color = *color3;

                self.m_quads.push(quad);
            }
            _ => {
                let mut triangle = DebugDrawTriangle::default();
                triangle.m_base.set_time(life_time);
                triangle.m_base.m_fill_mode = FillMode::Solid;
                triangle.m_base.m_no_depth_test = no_depth_test;

                triangle.m_vertex[0].pos = *point0;
                triangle.m_vertex[0].color = *color0;
                triangle.m_vertex[1].pos = *point1;
                triangle.m_vertex[1].color = *color1;
                triangle.m_vertex[2].pos = *point2;
                triangle.m_vertex[2].color = *color2;
                self.m_triangles.push(triangle);

                let mut triangle = DebugDrawTriangle::default();
                triangle.m_base.set_time(life_time);
                triangle.m_base.m_fill_mode = FillMode::Solid;
                triangle.m_base.m_no_depth_test = no_depth_test;

                triangle.m_vertex[0].pos = *point0;
                triangle.m_vertex[0].color = *color0;
                triangle.m_vertex[1].pos = *point2;
                triangle.m_vertex[1].color = *color2;
                triangle.m_vertex[2].pos = *point3;
                triangle.m_vertex[2].color = *color3;

                self.m_triangles.push(triangle);
            }
        }
    }

    pub fn add_box(
        &mut self, 
        center_point: &Vec3, 
        half_extends: &Vec3, 
        rotate: &Quat, 
        color: &Vec4, 
        life_time: f32, 
        no_depth_test: bool, 
    ) {
        let _guard = self.m_mutex.lock();
        let mut debug_box = DebugDrawBox::default();
        debug_box.m_base.set_time(life_time);
        debug_box.m_base.m_no_depth_test = no_depth_test;

        debug_box.m_center_point = *center_point;
        debug_box.m_half_extent = *half_extends;
        debug_box.m_rotate = *rotate;
        debug_box.m_color = *color;

        self.m_boxes.push(debug_box);

    }

    pub fn add_sphere(&mut self, center_point: &Vec3, radius: f32, color: &Vec4, life_time: f32, no_depth_test: bool) {
        let _guard = self.m_mutex.lock();
        let mut debug_sphere = DebugDrawSphere::default();
        debug_sphere.m_base.set_time(life_time);
        debug_sphere.m_base.m_no_depth_test = no_depth_test;

        debug_sphere.m_center = *center_point;
        debug_sphere.m_radius = radius;
        debug_sphere.m_color = *color;

        self.m_spheres.push(debug_sphere);
    }

    pub fn add_cylinder(&mut self,center: &Vec3, radius: f32, height: f32, rotate: &Quat, color: &Vec4, life_time: f32, no_depth_test: bool) {
        let _guard = self.m_mutex.lock();
        let mut debug_cylinder = DebugDrawCylinder::default();
        debug_cylinder.m_base.set_time(life_time);
        debug_cylinder.m_base.m_no_depth_test = no_depth_test;

        debug_cylinder.m_center = *center;
        debug_cylinder.m_rotate = *rotate;
        debug_cylinder.m_radius = radius;
        debug_cylinder.m_height = height;
        debug_cylinder.m_color = *color;

        self.m_cylinders.push(debug_cylinder);
    }

    pub fn add_capsule(&mut self, center: &Vec3, rotate: &Quat, scale: &Vec3, radius: f32, height: f32, color: &Vec4, life_time: f32, no_depth_test: bool) {
        let _guard = self.m_mutex.lock();
        let mut debug_capsule = DebugDrawCapsule::default();
        debug_capsule.m_base.set_time(life_time);
        debug_capsule.m_base.m_no_depth_test = no_depth_test;

        debug_capsule.m_center = *center;
        debug_capsule.m_rotate = *rotate;
        debug_capsule.m_scale = *scale;
        debug_capsule.m_radius = radius;
        debug_capsule.m_height = height;
        debug_capsule.m_color = *color;

        self.m_capsules.push(debug_capsule);
    }

    pub fn add_text(&mut self, content: &str, color: &Vec4, coordinate: &Vec3, size: i32, is_screen_text: bool, life_time: f32) {
        let _guard = self.m_mutex.lock();
        let mut text = DebugDrawText::default();
        text.m_base.set_time(life_time);

        text.m_content = content.to_string();
        text.m_color = *color;
        text.m_coordinate = *coordinate;
        text.m_size = size;
        text.m_is_screen_text = is_screen_text;

        self.m_texts.push(text);
    }

    pub fn remove_dead_primitives(&mut self, delta_time: f32){
        self.m_points.retain_mut(|p: &mut DebugDrawPoint| {
            !p.m_base.is_time_out(delta_time)
        });
        self.m_lines.retain_mut(|l: &mut DebugDrawLine| {
            !l.m_base.is_time_out(delta_time)
        });
        self.m_triangles.retain_mut(|t: &mut DebugDrawTriangle| {
            !t.m_base.is_time_out(delta_time)
        });
        self.m_quads.retain_mut(|q: &mut DebugDrawQuad| {
            !q.m_base.is_time_out(delta_time)
        });
        self.m_boxes.retain_mut(|b: &mut DebugDrawBox| {
            !b.m_base.is_time_out(delta_time)
        });
        self.m_cylinders.retain_mut(|c: &mut DebugDrawCylinder| {
            !c.m_base.is_time_out(delta_time)
        });
        self.m_spheres.retain_mut(|s: &mut DebugDrawSphere| {
            !s.m_base.is_time_out(delta_time)
        });
        self.m_capsules.retain_mut(|c: &mut DebugDrawCapsule| {
            !c.m_base.is_time_out(delta_time)
        });
        self.m_texts.retain_mut(|t: &mut DebugDrawText| {
            !t.m_base.is_time_out(delta_time)
        });
    }

    pub fn merge_from(&mut self, other: &DebugDrawGroup) {
        let _guard = self.m_mutex.lock();
        let _guard2 = other.m_mutex.lock();
        self.m_points.extend_from_slice(&other.m_points);
        self.m_lines.extend_from_slice(&other.m_lines);
        self.m_triangles.extend_from_slice(&other.m_triangles);
        self.m_quads.extend_from_slice(&other.m_quads);
        self.m_boxes.extend_from_slice(&other.m_boxes);
        self.m_cylinders.extend_from_slice(&other.m_cylinders);
        self.m_spheres.extend_from_slice(&other.m_spheres);
        self.m_capsules.extend_from_slice(&other.m_capsules);
        self.m_texts.extend_from_slice(&other.m_texts);
    }

    pub fn get_point_count(&self, no_depth_test: bool) -> usize {
        self.m_points.iter().filter(|p| p.m_base.m_no_depth_test == no_depth_test).count()
    }

    pub fn get_line_count(&self, no_depth_test: bool) -> usize {
        let mut res = 0;
        res += self.m_lines.iter().filter(|l| l.m_base.m_no_depth_test == no_depth_test).count();
        res += self.m_triangles.iter().filter(|t| {
            t.m_base.m_fill_mode == FillMode::WireFrame && t.m_base.m_no_depth_test == no_depth_test
        }).count() * 3;
        res += self.m_quads.iter().filter(|q| {
            q.m_base.m_fill_mode == FillMode::WireFrame && q.m_base.m_no_depth_test == no_depth_test
        }).count() * 4;
        res += self.m_boxes.iter().filter(|b| {
            b.m_base.m_fill_mode == FillMode::WireFrame && b.m_base.m_no_depth_test == no_depth_test
        }).count() * 12;
        res
    }

    pub fn get_triangle_count(&self, no_depth_test: bool) -> usize {
        self.m_triangles.iter().filter(|t| {
            t.m_base.m_fill_mode == FillMode::Solid && t.m_base.m_no_depth_test == no_depth_test
        }).count()
    }

    pub fn get_uniform_dynamic_data_count(&self) -> usize {
        self.m_cylinders.len() + self.m_spheres.len() + self.m_capsules.len()
    }

    pub fn write_point_data(&self, no_depth_test: bool) -> Vec<DebugDrawVertex> {
        self.m_points.iter()
            .filter(|p| p.m_base.m_no_depth_test == no_depth_test)
            .map(|p| p.m_vertex.clone())
            .collect()
    }

    pub fn write_line_data(&self, no_depth_test: bool) -> Vec<DebugDrawVertex> {
        let mut res = Vec::new();
        res.extend(self.m_lines.iter()
            .filter(|l| {
                l.m_base.m_fill_mode == FillMode::WireFrame && l.m_base.m_no_depth_test == no_depth_test
            })
            .flat_map(|p| p.m_vertex.clone()));
        res.extend(self.m_triangles.iter()
            .filter(|t|{
                t.m_base.m_fill_mode == FillMode::WireFrame && t.m_base.m_no_depth_test == no_depth_test
            })
            .flat_map(|t|{
                [0,1,1,2,2,0].iter().map(|i|{t.m_vertex[*i]}).collect::<Vec<_>>()
            }));
        res.extend(self.m_quads.iter()
            .filter(|q|{
                q.m_base.m_fill_mode == FillMode::WireFrame && q.m_base.m_no_depth_test == no_depth_test
            })
            .flat_map(|q|{
                [0,1,1,2,2,3,3,0].iter().map(|i|{q.m_vertex[*i]}).collect::<Vec<_>>()
            }));
        res.extend(self.m_boxes.iter()
            .filter(|b|{
                b.m_base.m_fill_mode == FillMode::WireFrame && b.m_base.m_no_depth_test == no_depth_test
            })
            .flat_map(|b|{
                let mut verts_4d = [DebugDrawVertex::default();8];
                let f = [-1.0,1.0];
                for i in 0..8 {
                    let v = Vec3::new(
                        f[i&1] * b.m_half_extent.x,
                        f[(i>>1)&1] * b.m_half_extent.y,
                        f[(i>>2)&1] * b.m_half_extent.z,
                    );
                    let qvec = Vec3::new(b.m_rotate[0],b.m_rotate[1],b.m_rotate[2]);
                    let uv = qvec.cross(&v);
                    let uuv = qvec.cross(&uv);
                    verts_4d[i].pos = v + uv + uuv + b.m_center_point;
                    verts_4d[i].color = b.m_color;

                }
                [0,1, 1,3, 3,2, 2,0,
                 4,5, 5,7, 7,6, 6,4,
                 0,4, 1,5, 3,7, 2,6].iter().map(|i|{verts_4d[*i]}).collect::<Vec<_>>()
            }));
        res
    }

    pub fn write_triangle_data(&self,no_depth_test: bool) -> Vec<DebugDrawVertex> {
        self.m_triangles.iter().filter(|t|{
            t.m_base.m_fill_mode == FillMode::Solid && t.m_base.m_no_depth_test == no_depth_test
        }).flat_map(|t|{
            t.m_vertex
        }).collect()
    }

    pub fn write_text_data(&self, _font: &DebugDrawFont, m_proj_view_matrix: &Mat4, screen_width: f32, screen_height: f32) -> Vec<DebugDrawVertex>{

        let mut vertices = Vec::with_capacity(self.get_text_character_count() * 6);
        for text in &self.m_texts{
            let absolute_w = text.m_size as f32;
            let absolute_h = (text.m_size * 2) as f32;
            let w = absolute_w  / (screen_width / 2.0);
            let h = absolute_h / (screen_height / 2.0);
            let mut coordinate = text.m_coordinate;
            if ! text.m_is_screen_text {
                let temp_coord = Vec4::new(coordinate.x,coordinate.y,coordinate.z,1.0);
                let temp_coord = m_proj_view_matrix * temp_coord;
                coordinate = Vec3::new(temp_coord.x / temp_coord.w, temp_coord.y / temp_coord.w,0.0);
            }
            let mut x = coordinate.x;
            let mut y = coordinate.y;
            for character in text.m_content.chars() {
                if character == '\n' {
                    y += h;
                    x = coordinate.x;
                }
                else{
                    let (x1,x2,y1,y2) = DebugDrawFont::get_character_texture_rect(character as u8);
                    let (cx1, cx2,cy1,cy2) = (x, w+x, y, h+y);

                    let mut vertex = DebugDrawVertex::default();
                    vertex.pos = Vec3::new(cx1, cy1, 0.0);
                    vertex.color = text.m_color;
                    vertex.texcoord = Vec2::new(x1,y1);
                    vertices.push(vertex);
                    let mut vertex = DebugDrawVertex::default();
                    vertex.pos = Vec3::new(cx1, cy2, 0.0);
                    vertex.color = text.m_color;
                    vertex.texcoord = Vec2::new(x1,y2);
                    vertices.push(vertex);
                    let mut vertex = DebugDrawVertex::default();
                    vertex.pos = Vec3::new(cx2, cy2, 0.0);
                    vertex.color = text.m_color;
                    vertex.texcoord = Vec2::new(x2,y2);
                    vertices.push(vertex);
                    let mut vertex = DebugDrawVertex::default();
                    vertex.pos = Vec3::new(cx1, cx1, 0.0);
                    vertex.color = text.m_color;
                    vertex.texcoord = Vec2::new(x1,y1);
                    vertices.push(vertex);

                    x += w;
                }
            }
        }
        vertices
    }

    pub fn write_unform_dynamic_data_to_cache(&self) -> Vec<(Mat4,Vec4)>{
        let mut res = Vec::with_capacity(self.get_uniform_dynamic_data_count()*3);
        let no_depth_tests = [false,true];
        for i in 0..2 {
            let no_depth_test = no_depth_tests[i];
            
            res.extend(self.m_spheres.iter().filter(|obj|{
                obj.m_base.m_no_depth_test == no_depth_test 
            }).map(|obj|{
                let trans = Mat4::new_translation(&obj.m_center);
                let scale = Mat4::new_scaling(obj.m_radius);
                (trans * scale, obj.m_color)
            }));
            res.extend(self.m_cylinders.iter().filter(|obj|{
                obj.m_base.m_no_depth_test == no_depth_test 
            }).map(|obj|{
                let trans = Mat4::new_translation(&obj.m_center);
                let scale = Mat4::new_nonuniform_scaling(
                    &Vec3::new(obj.m_radius,obj.m_radius, obj.m_height * 0.5)
                );
                let rotate = quat_to_mat4(&obj.m_rotate);
                (trans * rotate * scale, obj.m_color)
            }));
            res.extend(self.m_capsules.iter().filter(|obj|{
                obj.m_base.m_no_depth_test == no_depth_test 
            }).flat_map(|obj|{
                let trans = Mat4::new_translation(&obj.m_center);
                let scale = Mat4::new_nonuniform_scaling(&obj.m_scale);
                let rotate = quat_to_mat4(&obj.m_rotate);
                let trans1 = Mat4::new_translation(
                    &Vec3::new(0.0,0.0,obj.m_height*0.5 - obj.m_radius)
                );
                let scale2 = Mat4::new_nonuniform_scaling(
                    &Vec3::new(1.0,1.0,obj.m_height / (obj.m_radius * 2.0))
                );
                let trans3 = Mat4::new_translation(
                    &Vec3::new(0.0,0.0,-(obj.m_height*0.5 - obj.m_radius))
                );
                [(trans1 * trans  * scale * rotate, obj.m_color),
                 (trans  * scale2 * scale * rotate, obj.m_color),
                 (trans3 * trans  * scale * rotate, obj.m_color)]
            }));
        }

        res
    }

    pub fn get_sphere_count(&self, no_depth_test: bool) -> usize {
        self.m_spheres.iter().filter(|s| s.m_base.m_no_depth_test == no_depth_test).count()
    }
    
    pub fn get_cylinder_count(&self, no_depth_test: bool) -> usize {
        self.m_cylinders.iter().filter(|c| c.m_base.m_no_depth_test == no_depth_test).count()
    }

    pub fn get_capsule_count(&self, no_depth_test: bool) -> usize {
        self.m_capsules.iter().filter(|c| c.m_base.m_no_depth_test == no_depth_test).count()
    }

    pub fn get_text_character_count(&self) -> usize {
        self.m_texts.iter().map(|t| {
            t.m_content.chars().filter(|c| *c != '\n').count()
        }).sum()
    }

}