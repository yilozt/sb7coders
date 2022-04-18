// Copyright � 2012-2015 Graham Sellers
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use sb7::prelude::*;

#[derive(Default)]
struct GsLayered {
    program_gslayers: GLuint,
    program_showlayers: GLuint,
    vao: GLuint,
    transform_ubo: GLuint,

    layered_fbo: GLuint,
    array_texture: GLuint,
    array_depth: GLuint,

    obj: Object,
}

impl Application for GsLayered {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Layered Rendering".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            self.load_shaders();

            self.obj.load("media/objects/torus.sbm");

            gl::GenBuffers(1, &mut self.transform_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.transform_ubo);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                17 * size_of::<Mat4>() as GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );

            gl::GenTextures(1, &mut self.array_texture);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.array_texture);
            gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8, 256, 256, 16);

            gl::GenTextures(1, &mut self.array_depth);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.array_depth);
            gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 1, gl::DEPTH_COMPONENT32, 256, 256, 16);

            gl::GenFramebuffers(1, &mut self.layered_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.layered_fbo);
            gl::FramebufferTexture(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                self.array_texture,
                0,
            );
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, self.array_depth, 0);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_gslayers);
            gl::DeleteProgram(self.program_showlayers);
            gl::DeleteBuffers(1, &self.transform_ubo);
            gl::DeleteTextures(1, &self.array_texture);
            gl::DeleteTextures(1, &self.array_depth);
            gl::DeleteFramebuffers(1, &self.layered_fbo);
        }
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();
        let gray = [0.1, 0.1, 0.1, 1.0f32].as_ptr();
        let one = 1.0f32;

        let info = self.info();
        let t = current_time as f32;

        #[repr(C)]
        struct TransformBuffer {
            proj_matrix: Mat4,
            mv_matrix: [Mat4; 16],
        }

        unsafe {
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, self.transform_ubo);

            let buffer = gl::MapBufferRange(
                gl::UNIFORM_BUFFER,
                0,
                17 * size_of::<Mat4>() as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
            ) as *mut TransformBuffer;

            (*buffer).proj_matrix = perspective(50.0, 1.0, 0.1, 1000.0);

            for i in 0..16 {
                let fi = (i as f32 + 12.0) / 16.0;
                (*buffer).mv_matrix[i] = translate(0.0, 0.0, -4.0)
                    * rotate_with_axis(t * fi * 25.0, 0.0, 0.0, 1.0)
                    * rotate_with_axis(t * fi * 30.0, 1.0, 0.0, 0.0);
            }

            gl::UnmapBuffer(gl::UNIFORM_BUFFER);

            let cao = gl::COLOR_ATTACHMENT0;

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.layered_fbo);
            gl::DrawBuffers(1, &cao);
            gl::Viewport(0, 0, 256, 256);
            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);

            gl::UseProgram(self.program_gslayers);

            self.obj.render();

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::DrawBuffer(gl::BACK);
            gl::UseProgram(self.program_showlayers);

            gl::Viewport(0, 0, info.width as GLsizei, info.height as GLsizei);
            gl::ClearBufferfv(gl::COLOR, 0, gray);

            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.array_texture);
            gl::Disable(gl::DEPTH_TEST);

            gl::BindVertexArray(self.vao);
            gl::DrawArraysInstanced(gl::TRIANGLE_FAN, 0, 4, 16);

            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
    }
}

impl GsLayered {
    fn load_shaders(&mut self) {
        if self.program_showlayers != 0 {
            unsafe { gl::DeleteProgram(self.program_showlayers) };
        }

        self.program_showlayers = program::link_from_shaders(
            &[
                shader::load(
                    "media/shaders/gslayers/showlayers.vs.glsl",
                    gl::VERTEX_SHADER,
                    true,
                ),
                shader::load(
                    "media/shaders/gslayers/showlayers.fs.glsl",
                    gl::FRAGMENT_SHADER,
                    true,
                ),
            ],
            true,
        );

        if self.program_gslayers != 0 {
            unsafe { gl::DeleteProgram(self.program_gslayers) };
        }

        self.program_gslayers = program::link_from_shaders(
            &[
                shader::load(
                    "media/shaders/gslayers/gslayers.vs.glsl",
                    gl::VERTEX_SHADER,
                    true,
                ),
                shader::load(
                    "media/shaders/gslayers/gslayers.gs.glsl",
                    gl::GEOMETRY_SHADER,
                    true,
                ),
                shader::load(
                    "media/shaders/gslayers/gslayers.fs.glsl",
                    gl::FRAGMENT_SHADER,
                    true,
                ),
            ],
            true,
        );
    }
}

fn main() {
    GsLayered::default().run()
}
