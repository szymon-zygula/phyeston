use egui_winit::winit::dpi::PhysicalSize;
use glow::HasContext;
use std::sync::Arc;

pub struct Drawbuffer {
    gl: Arc<glow::Context>,
    framebuffer: glow::Framebuffer,
    rgb_texture: glow::Texture,
    depth_stencil_texture: glow::Texture,
    size: PhysicalSize<i32>,
}

impl Drawbuffer {
    pub fn new(gl: Arc<glow::Context>, width: i32, height: i32) -> Self {
        let framebuffer = unsafe { gl.create_framebuffer() }.unwrap();

        unsafe { gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer)) };
        let rgb_texture = unsafe { Self::attach_rgb(gl.as_ref(), width, height) };
        let depth_stencil_texture =
            unsafe { Self::attach_depth_stencil(gl.as_ref(), width, height) };
        unsafe { gl.bind_framebuffer(glow::FRAMEBUFFER, None) };

        Self {
            framebuffer,
            gl,
            rgb_texture,
            depth_stencil_texture,
            size: PhysicalSize { width, height },
        }
    }

    unsafe fn attach_rgb(gl: &glow::Context, width: i32, height: i32) -> glow::Texture {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as i32,
            width,
            height,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            None,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(texture),
            0,
        );

        texture
    }

    unsafe fn attach_depth_stencil(gl: &glow::Context, width: i32, height: i32) -> glow::Texture {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::DEPTH24_STENCIL8 as i32,
            width,
            height,
            0,
            glow::DEPTH_STENCIL,
            glow::UNSIGNED_INT_24_8,
            None,
        );
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::DEPTH_STENCIL_ATTACHMENT,
            glow::TEXTURE_2D,
            Some(texture),
            0,
        );

        texture
    }

    pub fn clear(&self) {
        unsafe {
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    pub fn draw_with<F: FnOnce()>(&self, f: F) {
        let old_viewport = unsafe {
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            self.gl.viewport(0, 0, self.size.width, self.size.height);
            let mut old_viewport: [i32; 4] = [0, 0, 0, 0];
            self.gl
                .get_parameter_i32_slice(glow::VIEWPORT, &mut old_viewport);
            old_viewport
        };

        f();

        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.viewport(
                old_viewport[0],
                old_viewport[1],
                old_viewport[2],
                old_viewport[3],
            );
        }
    }

    pub fn blit(&self, x: i32, y: i32) {
        unsafe {
            self.gl
                .bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.framebuffer));
            self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);

            self.gl.blit_framebuffer(
                0,
                0,
                self.size.width,
                self.size.height,
                x,
                y,
                x + self.size.width,
                y + self.size.height,
                glow::COLOR_BUFFER_BIT,
                glow::LINEAR,
            );

            self.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
    }

    pub fn size(&self) -> PhysicalSize<i32> {
        self.size
    }
}

impl Drop for Drawbuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.framebuffer);
            self.gl.delete_texture(self.rgb_texture);
            self.gl.delete_texture(self.depth_stencil_texture);
        }
    }
}
