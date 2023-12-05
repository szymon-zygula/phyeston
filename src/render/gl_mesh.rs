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

pub struct GlPointCloud {
    vertex_buffer: glow::Buffer,
    vertex_array: std::mem::MaybeUninit<glow::VertexArray>,
    point_count: usize,
    gl: Arc<glow::Context>,
}

impl GlPointCloud {
    pub fn new(gl: Arc<glow::Context>, vertices: &[na::Point3<f32>]) -> Self {
        let mut mesh = Self::new_uninit(Arc::clone(&gl), vertices.len());

        mesh.vertex_array = std::mem::MaybeUninit::new(opengl::init_vao(&gl, || {
            mesh.update_points(vertices);

            unsafe {
                mesh.gl.vertex_attrib_pointer_f32(
                    0,
                    3,
                    glow::FLOAT,
                    false,
                    3 * std::mem::size_of::<f32>() as i32,
                    0,
                );
                mesh.gl.enable_vertex_attrib_array(0);
            }
        }));

        mesh
    }

    fn new_uninit(gl: Arc<glow::Context>, point_count: usize) -> GlPointCloud {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();

        Self {
            point_count,
            vertex_buffer,
            vertex_array: std::mem::MaybeUninit::uninit(),
            gl,
        }
    }

    pub fn update_points(&mut self, points: &[na::Point3<f32>]) {
        let raw_points = utils::slice_as_raw(points);

        unsafe {
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);
        }
    }
}

impl Drop for GlPointCloud {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array.assume_init());
            self.gl.delete_buffer(self.vertex_buffer);
        }
    }
}

impl GlDrawable for GlPointCloud {
    fn draw(&self) {
        unsafe {
            opengl::with_vao(&self.gl, self.vertex_array.assume_init(), || {
                self.gl
                    .draw_arrays(glow::POINTS, 0, self.point_count as i32);
            });
        }
    }
}

pub struct GlLines(GlPointCloud);

impl GlLines {
    pub fn new(gl: Arc<glow::Context>, vertices: &[na::Point3<f32>]) -> Self {
        GlLines(GlPointCloud::new(gl, vertices))
    }

    pub fn update_points(&mut self, points: &[na::Point3<f32>]) {
        self.0.update_points(points)
    }
}

impl GlDrawable for GlLines {
    fn draw(&self) {
        unsafe {
            opengl::with_vao(&self.0.gl, self.0.vertex_array.assume_init(), || {
                self.0
                    .gl
                    .draw_arrays(glow::LINES, 0, self.0.point_count as i32);
            });
        }
    }
}

pub struct GlTesselationBicubicPatch {
    gl: Arc<glow::Context>,
    vertex_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
}

impl GlTesselationBicubicPatch {
    const VERTEX_COUNT: i32 = 16;

    pub fn new(gl: Arc<glow::Context>, surface_points: &[[na::Point3<f32>; 4]; 4]) -> Self {
        let (vertex_array, vertex_buffer) = Self::create_vao_vbo(&gl, surface_points);
        Self {
            gl,
            vertex_array,
            vertex_buffer,
        }
    }

    fn create_vao_vbo(
        gl: &glow::Context,
        input: &[[na::Point3<f32>; 4]; 4],
    ) -> (glow::VertexArray, glow::Buffer) {
        let raw_input = utils::slice_as_raw(input);
        opengl::create_vao_vbo_points(gl, raw_input)
    }
}

impl GlDrawable for GlTesselationBicubicPatch {
    fn draw(&self) {
        opengl::with_vao(&self.gl, self.vertex_array, || unsafe {
            self.gl.patch_parameter_i32(glow::PATCH_VERTICES, 16);
            self.gl.draw_arrays(glow::PATCHES, 0, Self::VERTEX_COUNT);
            self.gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
        });
    }
}

impl Drop for GlTesselationBicubicPatch {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
        }
    }
}
