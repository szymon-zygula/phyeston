use glow::{self, HasContext};
use std::{path::Path, sync::Arc};

const SHADERS_PATH: &str = "shaders/";
const SHADERS_EXTENSION: &str = "glsl";

pub struct Shader {
    kind: u32,
    handle: glow::Shader,
    gl: Arc<glow::Context>,
}

impl Shader {
    pub fn from_name(gl: Arc<glow::Context>, name: &str, kind: u32) -> Shader {
        let mut path = Path::new(SHADERS_PATH).join(name);
        path.set_extension(SHADERS_EXTENSION);
        Self::from_path(gl, &path, kind)
    }

    pub fn from_path(gl: Arc<glow::Context>, shader_path: &Path, kind: u32) -> Shader {
        let err_msg = format!("Failed to load shader source code ({:?})", shader_path);
        let shader_source = std::fs::read_to_string(shader_path).expect(&err_msg);

        let handle = unsafe {
            let handle = gl.create_shader(kind).unwrap();
            gl.shader_source(handle, &shader_source);
            gl.compile_shader(handle);

            if !gl.get_shader_compile_status(handle) {
                panic!(
                    "Error compiling shader ({}): {}",
                    shader_path.to_str().unwrap(),
                    gl.get_shader_info_log(handle)
                );
            }

            handle
        };

        Shader { kind, handle, gl }
    }

    pub fn handle(&self) -> glow::Shader {
        self.handle
    }

    pub fn kind(&self) -> u32 {
        self.kind
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { self.gl.delete_shader(self.handle) };
    }
}
