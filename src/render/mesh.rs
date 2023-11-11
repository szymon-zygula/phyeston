use super::{gl_drawable::GlDrawable, opengl};
use crate::utils;
use glow::HasContext;
use nalgebra as na;
use num_traits::ToPrimitive;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Triangle(pub [u32; 3]);

pub trait Vertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context);
}

impl Vertex for na::Point3<f32> {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<na::Point3<f32>>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ClassicVertex {
    pub position: na::Point3<f32>,
    pub normal: na::Vector3<f32>,
}

impl ClassicVertex {
    pub fn new(position: na::Point3<f32>, normal: na::Vector3<f32>) -> Self {
        Self { position, normal }
    }
}

impl Vertex for ClassicVertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<ClassicVertex>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<ClassicVertex>() as i32,
                std::mem::size_of::<na::Point3<f32>>() as i32,
            );
            gl.enable_vertex_attrib_array(1);
        }
    }
}

pub struct Mesh<V: Vertex> {
    pub vertices: Vec<V>,
    pub triangles: Vec<Triangle>,
}

impl<V: Vertex> Mesh<V> {
    pub fn new(vertices: Vec<V>, triangles: Vec<Triangle>) -> Self {
        Self {
            vertices,
            triangles,
        }
    }

    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

pub struct GlMesh {
    vertex_buffer: glow::Buffer,
    element_buffer: glow::Buffer,
    element_count: u32,
    vertex_array: glow::VertexArray,
    gl: Arc<glow::Context>,
}

pub struct GlTriangleMesh(GlMesh);

impl GlTriangleMesh {
    pub fn new<V: Vertex>(gl: Arc<glow::Context>, mesh: &Mesh<V>) -> Self {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let element_buffer = unsafe { gl.create_buffer() }.unwrap();

        let vertex_array = opengl::init_vao(&gl, || unsafe {
            let raw_points = utils::slice_as_raw(&mesh.vertices);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);

            let raw_elements = utils::slice_as_raw(&mesh.triangles);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, raw_elements, glow::STATIC_DRAW);

            V::set_vertex_attrib_pointers(&gl);
        });

        Self(GlMesh {
            vertex_buffer,
            element_buffer,
            element_count: 3 * mesh.triangles.len() as u32,
            vertex_array,
            gl,
        })
    }
}

impl GlDrawable for GlTriangleMesh {
    fn draw(&self) {
        opengl::with_vao(&self.0.gl, self.0.vertex_array, || unsafe {
            self.0.gl.draw_elements(
                glow::TRIANGLES,
                self.0.element_count as i32,
                glow::UNSIGNED_INT,
                0,
            );
        });
    }
}

impl Drop for GlTriangleMesh {
    fn drop(&mut self) {
        unsafe {
            self.0.gl.delete_vertex_array(self.0.vertex_array);
            self.0.gl.delete_buffer(self.0.vertex_buffer);
            self.0.gl.delete_buffer(self.0.element_buffer);
        }
    }
}

pub struct GlLineStrip {
    vertex_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
    vertex_count: i32,
    gl: Arc<glow::Context>,
}

impl GlLineStrip {
    pub fn new(gl: Arc<glow::Context>, strip: &Vec<na::Point3<f32>>) -> Self {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();

        let vertex_array = opengl::init_vao(&gl, || unsafe {
            let raw_points = utils::slice_as_raw(&strip);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);

            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<na::Point3<f32>>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);
        });

        Self {
            vertex_buffer,
            vertex_count: strip.len().to_i32().unwrap(),
            vertex_array,
            gl,
        }
    }
}

impl GlDrawable for GlLineStrip {
    fn draw(&self) {
        opengl::with_vao(&self.gl, self.vertex_array, || unsafe {
            self.gl.draw_arrays(glow::LINE_STRIP, 0, self.vertex_count);
        });
    }
}

impl Drop for GlLineStrip {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
        }
    }
}
