use super::{color::Color, shader::Shader};
use glow::{self, HasContext};
use itertools::Itertools;
use std::sync::Arc;

pub struct GlProgram {
    handle: glow::Program,
    gl: Arc<glow::Context>,
}

macro_rules! fn_set_uniform {
    ($type:ty, $fn_name:ident) => {
        pub fn $fn_name(&self, name: &str, data: $type) {
            unsafe {
                let location = self.gl.get_uniform_location(self.handle, name).unwrap();
                self.gl.$fn_name(Some(&location), false, data);
            }
        }
    };
}

impl GlProgram {
    pub fn with_shaders(gl: Arc<glow::Context>, shaders: &[&Shader]) -> Self {
        let handle = unsafe { gl.create_program() }.unwrap();

        unsafe {
            for shader in shaders {
                gl.attach_shader(handle, shader.handle());
            }

            gl.link_program(handle);

            if !gl.get_program_link_status(handle) {
                panic!("Error linking shader: {}", gl.get_program_info_log(handle));
            }

            for shader in shaders {
                gl.detach_shader(handle, shader.handle());
            }
        }

        GlProgram { handle, gl }
    }

    pub fn with_shader_names(gl: Arc<glow::Context>, shader_paths: &[(&str, u32)]) -> Self {
        let shaders = shader_paths
            .iter()
            .map(|(name, kind)| Shader::from_name(Arc::clone(&gl), name, *kind))
            .collect_vec();

        Self::with_shaders(gl, &shaders.iter().collect::<Vec<&Shader>>())
    }

    pub fn vertex_fragment(gl: Arc<glow::Context>, vertex_name: &str, fragment_name: &str) -> Self {
        Self::with_shader_names(
            gl,
            &[
                (vertex_name, glow::VERTEX_SHADER),
                (fragment_name, glow::FRAGMENT_SHADER),
            ],
        )
    }

    fn_set_uniform!(&[f32], uniform_matrix_2_f32_slice);
    fn_set_uniform!(&[f32], uniform_matrix_3_f32_slice);
    fn_set_uniform!(&[f32], uniform_matrix_4_f32_slice);

    pub fn uniform_f32(&self, name: &str, data: f32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_1_f32(Some(&location), data);
        }
    }

    pub fn uniform_u32(&self, name: &str, data: u32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_1_u32(Some(&location), data);
        }
    }

    pub fn uniform_i32(&self, name: &str, data: i32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_1_i32(Some(&location), data);
        }
    }

    pub fn uniform_3_f32(&self, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_3_f32(Some(&location), x, y, z);
        }
    }

    pub fn uniform_4_f32(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_4_f32(Some(&location), x, y, z, w);
        }
    }

    pub fn uniform_3_f32_slice(&self, name: &str, slice: &[f32]) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_3_f32_slice(Some(&location), slice);
        }
    }

    pub fn uniform_4_f32_slice(&self, name: &str, slice: &[f32]) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_4_f32_slice(Some(&location), slice);
        }
    }

    pub fn uniform_color(&self, name: &str, color: &Color) {
        self.uniform_3_f32(name, color.r, color.g, color.b);
    }

    pub fn handle(&self) -> glow::Program {
        self.handle
    }

    pub fn enable(&self) {
        unsafe {
            self.gl.use_program(Some(self.handle));
        }
    }
}

impl Drop for GlProgram {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.handle);
        }
    }
}
