use crate::{render::texture::Texture, utils};
use glow::HasContext;
use std::sync::Arc;

fn texture_format(texture: &Texture) -> u32 {
    match texture.image {
        image::DynamicImage::ImageRgb8(_) => glow::RGB,
        image::DynamicImage::ImageRgba8(_) => glow::RGBA,
        _ => panic!("Unsupported texture format"),
    }
}

pub struct GlTexture {
    gl: Arc<glow::Context>,
    texture: glow::Texture,
}

impl GlTexture {
    pub fn new(gl: Arc<glow::Context>, texture: &Texture) -> Self {
        let handle = Self::create_and_bind(&gl);

        let gl_texture = Self {
            gl,
            texture: handle,
        };
        gl_texture.load(texture);
        gl_texture
    }

    pub fn new_float(
        gl: Arc<glow::Context>,
        texture: &Vec<f32>,
        width: usize,
        height: usize,
    ) -> Self {
        let handle = Self::create_and_bind(&gl);

        let gl_texture = Self {
            gl,
            texture: handle,
        };
        gl_texture.load_float(texture, width, height);
        gl_texture
    }

    fn create_and_bind(gl: &glow::Context) -> glow::Texture {
        unsafe {
            let texture = gl
                .create_texture()
                .unwrap_or_else(|msg| panic!("Failed to create GlTexture: {}", msg));
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            texture
        }
    }

    pub fn bind(&self) {
        unsafe { self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture)) }
    }

    pub fn bind_to_image_unit(&self, image_unit: u32) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 + image_unit);
        }
        self.bind();
    }

    pub fn load(&self, texture: &Texture) {
        let format = texture_format(texture);

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format as i32,
                texture.image.width() as i32,
                texture.image.height() as i32,
                0,
                format,
                glow::UNSIGNED_BYTE,
                Some(texture.image.as_bytes()),
            );
            self.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }

    pub fn load_float(&self, texture: &Vec<f32>, width: usize, height: usize) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::MIRRORED_REPEAT as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::MIRRORED_REPEAT as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R32F as i32,
                width as i32,
                height as i32,
                0,
                glow::RED,
                glow::FLOAT,
                Some(utils::slice_as_raw(texture.as_slice())),
            );
            self.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }

    pub fn handle(&self) -> glow::Texture {
        self.texture
    }
}

impl Drop for GlTexture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}

pub struct GlCubeTexture {
    gl: Arc<glow::Context>,
    handle: glow::Texture,
}

impl GlCubeTexture {
    pub fn new(gl: Arc<glow::Context>, textures: &[Texture; 6]) -> Self {
        let handle = Self::create_and_bind(&gl);

        let gl_texture = Self { gl, handle };
        gl_texture.load(textures);
        gl_texture
    }

    fn create_and_bind(gl: &glow::Context) -> glow::Texture {
        unsafe {
            let texture = gl
                .create_texture()
                .unwrap_or_else(|msg| panic!("Failed to create GlCubeTexture: {}", msg));
            gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));

            gl.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_WRAP_R,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_CUBE_MAP,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            texture
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl
                .bind_texture(glow::TEXTURE_CUBE_MAP, Some(self.handle))
        }
    }

    pub fn load(&self, textures: &[Texture; 6]) {
        let format = texture_format(&textures[0]);

        for texture in textures.iter().skip(1) {
            assert_eq!(texture_format(texture), format);
        }

        self.bind();

        unsafe {
            for (idx, texture) in textures.iter().enumerate() {
                self.gl.tex_image_2d(
                    glow::TEXTURE_CUBE_MAP_POSITIVE_X + idx as u32,
                    0,
                    format as i32,
                    texture.image.width() as i32,
                    texture.image.height() as i32,
                    0,
                    format,
                    glow::UNSIGNED_BYTE,
                    Some(texture.image.as_bytes()),
                );
            }

            self.gl.generate_mipmap(glow::TEXTURE_CUBE_MAP);
        }
    }
}

impl Drop for GlCubeTexture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.handle);
        }
    }
}
