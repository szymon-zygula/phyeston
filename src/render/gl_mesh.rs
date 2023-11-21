use super::{
    gl_drawable::GlDrawable,
    mesh::{Mesh, Vertex},
    opengl,
};
use crate::utils;
use glow::HasContext;
use nalgebra as na;
use num_traits::ToPrimitive;
use std::sync::Arc;

const POINT_SIZE: i32 = std::mem::size_of::<na::Point3<f32>>() as i32;

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
    vertex_count: i32,
    capacity: i32,
    first: i32,
    vertex_array: glow::VertexArray,
    gl: Arc<glow::Context>,
}

impl GlLineStrip {
    pub fn with_capacity(gl: Arc<glow::Context>, capacity: usize) -> Self {
        let capacity = capacity as i32;
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();

        let vertex_array = opengl::init_vao(&gl, || unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_storage(
                glow::ARRAY_BUFFER,
                (capacity + 1) * POINT_SIZE,
                None,
                glow::DYNAMIC_STORAGE_BIT,
            );

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, POINT_SIZE, 0);
            gl.enable_vertex_attrib_array(0);
        });

        Self {
            vertex_buffer,
            vertex_count: 0,
            capacity,
            vertex_array,
            first: 0,
            gl,
        }
    }

    pub fn new(gl: Arc<glow::Context>, strip: &[na::Point3<f32>]) -> Self {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();

        let vertex_array = opengl::init_vao(&gl, || unsafe {
            let raw_points = utils::slice_as_raw(strip);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, POINT_SIZE, 0);
            gl.enable_vertex_attrib_array(0);
        });

        let length = strip.len().to_i32().unwrap();

        Self {
            vertex_buffer,
            vertex_count: length,
            capacity: length,
            vertex_array,
            first: 0,
            gl,
        }
    }

    pub fn recapacitate(&mut self, capacity: usize) {
        let capacity = capacity as i32;

        if capacity == self.capacity {
            return;
        }

        let new_buffer = unsafe {
            let new_buffer = self.gl.create_buffer().unwrap();
            self.gl.delete_vertex_array(self.vertex_array);

            self.vertex_array = opengl::init_vao(&self.gl, || {
                self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(new_buffer));
                self.gl.buffer_storage(
                    glow::ARRAY_BUFFER,
                    (capacity + 1) * POINT_SIZE,
                    None,
                    glow::DYNAMIC_STORAGE_BIT,
                );

                self.gl
                    .vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, POINT_SIZE, 0);
                self.gl.enable_vertex_attrib_array(0);
            });

            self.gl.delete_buffer(self.vertex_buffer);
            new_buffer
        };

        self.first = 0;
        self.capacity = capacity;
        self.vertex_buffer = new_buffer;
        self.vertex_count = 0;
    }

    pub fn push_vertex(&mut self, vertex: &na::Point3<f32>) {
        let slot = (self.first + self.vertex_count) % self.capacity;
        let offset = POINT_SIZE * slot;

        unsafe {
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            self.gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                offset,
                utils::slice_as_raw(vertex.coords.as_slice()),
            )
        };

        if self.vertex_count == self.capacity {
            if slot == 0 {
                unsafe {
                    self.gl.buffer_sub_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        POINT_SIZE * self.capacity,
                        utils::slice_as_raw(vertex.coords.as_slice()),
                    )
                };

                self.first = 1;
            } else {
                self.first += 1;
            }
        } else {
            self.vertex_count += 1;
        }
    }
}

impl GlDrawable for GlLineStrip {
    fn draw(&self) {
        let first_draw_count = if self.vertex_count == self.capacity && self.first != 0 {
            self.vertex_count - self.first + 1
        } else {
            self.vertex_count - self.first
        };

        opengl::with_vao(&self.gl, self.vertex_array, || unsafe {
            self.gl
                .draw_arrays(glow::LINE_STRIP, self.first, first_draw_count);
            self.gl.draw_arrays(glow::LINE_STRIP, 0, self.first);
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
