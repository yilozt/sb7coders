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
#[rustfmt::skip]
struct BasicFboApp {
    program1:           GLuint,
    program2:           GLuint,
    vao:                GLuint,
    position_buffer:    GLuint,
    index_buffer:       GLuint,
    fbo:                GLuint,
    color_texture:      GLuint,
    depth_texture:      GLuint,
    mv_location:        GLint,
    proj_location:      GLint,
    mv_location2:       GLint,
    proj_location2:     GLint,
}

impl Application for BasicFboApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Basic Framebuffer Object".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_source = cstring(
            r#"
            #version 410 core

            layout (location = 0) in vec4 position;
            layout (location = 1) in vec2 texcoord;

            out VS_OUT
            {
                vec4 color;
                vec2 texcoord;
            } vs_out;

            uniform mat4 mv_matrix;
            uniform mat4 proj_matrix;

            void main(void)
            {
                gl_Position = proj_matrix * mv_matrix * position;
                vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
                vs_out.texcoord = texcoord;
            }"#,
        );

        let fs_source1 = cstring(
            r#"
            #version 410 core

            in VS_OUT
            {
                vec4 color;
                vec2 texcoord;
            } fs_in;

            out vec4 color;

            void main(void)
            {
                color = sin(fs_in.color * vec4(40.0, 20.0, 30.0, 1.0)) * 0.5 + vec4(0.5);
            }"#,
        );

        let fs_source2 = cstring(
            r#"
            #version 420 core

            uniform sampler2D tex;

            out vec4 color;

            in VS_OUT
            {
                vec4 color;
                vec2 texcoord;
            } fs_in;

            void main(void)
            {
                color = mix(fs_in.color, texture(tex, fs_in.texcoord), 0.7);
            }"#,
        );

        unsafe {
            self.program1 = gl::CreateProgram();

            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_source1.as_ptr(), null());
            gl::CompileShader(fs);

            gl::AttachShader(self.program1, vs);
            gl::AttachShader(self.program1, fs);

            gl::LinkProgram(self.program1);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            self.program2 = gl::CreateProgram();

            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_source2.as_ptr(), null());
            gl::CompileShader(fs);

            gl::AttachShader(self.program2, vs);
            gl::AttachShader(self.program2, fs);

            gl::LinkProgram(self.program2);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            unsafe fn get_loc(program: GLuint, name: &str) -> GLint {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(program, name.as_ptr())
            }

            self.mv_location = get_loc(self.program1, "mv_matrix");
            self.proj_location = get_loc(self.program1, "proj_matrix");
            self.mv_location2 = get_loc(self.program2, "mv_matrix");
            self.proj_location2 = get_loc(self.program2, "proj_matrix");

            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            #[rustfmt::skip]
            let vertex_indices: &[GLushort] = &[
                0, 1, 2,
                2, 1, 3,
                2, 3, 4,
                4, 3, 5,
                4, 5, 6,
                6, 5, 7,
                6, 7, 0,
                0, 7, 1,
                6, 0, 2,
                2, 4, 6,
                7, 5, 3,
                7, 3, 1
            ];

            #[rustfmt::skip]
            let vertex_data: &[f32] = &[
                // Position                 Tex Coord
                -0.25f32, -0.25f32,  0.25f32,      0.0f32, 1.0f32,
                -0.25f32, -0.25f32, -0.25f32,      0.0f32, 0.0f32,
                 0.25f32, -0.25f32, -0.25f32,      1.0f32, 0.0f32,

                 0.25f32, -0.25f32, -0.25f32,      1.0f32, 0.0f32,
                 0.25f32, -0.25f32,  0.25f32,      1.0f32, 1.0f32,
                -0.25f32, -0.25f32,  0.25f32,      0.0f32, 1.0f32,

                 0.25f32, -0.25f32, -0.25f32,      0.0f32, 0.0f32,
                 0.25f32,  0.25f32, -0.25f32,      1.0f32, 0.0f32,
                 0.25f32, -0.25f32,  0.25f32,      0.0f32, 1.0f32,

                 0.25f32,  0.25f32, -0.25f32,      1.0f32, 0.0f32,
                 0.25f32,  0.25f32,  0.25f32,      1.0f32, 1.0f32,
                 0.25f32, -0.25f32,  0.25f32,      0.0f32, 1.0f32,

                 0.25f32,  0.25f32, -0.25f32,      1.0f32, 0.0f32,
                -0.25f32,  0.25f32, -0.25f32,      0.0f32, 0.0f32,
                 0.25f32,  0.25f32,  0.25f32,      1.0f32, 1.0f32,

                -0.25f32,  0.25f32, -0.25f32,      0.0f32, 0.0f32,
                -0.25f32,  0.25f32,  0.25f32,      0.0f32, 1.0f32,
                 0.25f32,  0.25f32,  0.25f32,      1.0f32, 1.0f32,

                -0.25f32,  0.25f32, -0.25f32,      1.0f32, 0.0f32,
                -0.25f32, -0.25f32, -0.25f32,      0.0f32, 0.0f32,
                -0.25f32,  0.25f32,  0.25f32,      1.0f32, 1.0f32,

                -0.25f32, -0.25f32, -0.25f32,      0.0f32, 0.0f32,
                -0.25f32, -0.25f32,  0.25f32,      0.0f32, 1.0f32,
                -0.25f32,  0.25f32,  0.25f32,      1.0f32, 1.0f32,

                -0.25f32,  0.25f32, -0.25f32,      0.0f32, 1.0f32,
                 0.25f32,  0.25f32, -0.25f32,      1.0f32, 1.0f32,
                 0.25f32, -0.25f32, -0.25f32,      1.0f32, 0.0f32,

                 0.25f32, -0.25f32, -0.25f32,      1.0f32, 0.0f32,
                -0.25f32, -0.25f32, -0.25f32,      0.0f32, 0.0f32,
                -0.25f32,  0.25f32, -0.25f32,      0.0f32, 1.0f32,

                -0.25f32, -0.25f32,  0.25f32,      0.0f32, 0.0f32,
                 0.25f32, -0.25f32,  0.25f32,      1.0f32, 0.0f32,
                 0.25f32,  0.25f32,  0.25f32,      1.0f32, 1.0f32,

                 0.25f32,  0.25f32,  0.25f32,      1.0f32, 1.0f32,
                -0.25f32,  0.25f32,  0.25f32,      0.0f32, 1.0f32,
                -0.25f32, -0.25f32,  0.25f32,      0.0f32, 0.0f32,
            ];

            gl::GenBuffers(1, &mut self.position_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.position_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertex_data) as GLsizeiptr,
                vertex_data.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (5 * size_of::<GLfloat>()) as GLsizei,
                null(),
            );
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * size_of::<GLfloat>()) as GLsizei,
                (3 * size_of::<GLfloat>()) as *const std::ffi::c_void,
            );
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            gl::GenBuffers(1, &mut self.index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(vertex_indices) as GLsizeiptr,
                vertex_indices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );

            gl::Enable(gl::CULL_FACE);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);

            gl::GenFramebuffers(1, &mut self.fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

            gl::GenTextures(1, &mut self.color_texture);
            gl::BindTexture(gl::TEXTURE_2D, self.color_texture);
            gl::TexStorage2D(gl::TEXTURE_2D, 9, gl::RGBA8, 512, 512);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::GenTextures(1, &mut self.depth_texture);
            gl::BindTexture(gl::TEXTURE_2D, self.depth_texture);
            gl::TexStorage2D(gl::TEXTURE_2D, 9, gl::DEPTH_COMPONENT32F, 512, 512);

            gl::FramebufferTexture(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                self.color_texture,
                0,
            );
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, self.depth_texture, 0);

            // gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
        }
    }

    fn render(&self, current_time: f64) {
        let blue = [0.0, 0.0, 0.3, 1.0f32].as_ptr();
        let one = 1.0f32;

        let proj_matrix = perspective(
            50.0,
            {
                let info = self.info();
                info.width as f32 / info.height as f32
            },
            0.1,
            1000.0,
        );

        let f = current_time as f32 * 0.3;

        let mv_matrix = translate(0.0, 0.0, -4.0)
            * translate(
                (2.1 * f).sin() * 0.5,
                (1.7 * f).cos() * 0.5,
                (1.3 * f).sin() * (1.5 * f).cos() * 2.0,
            )
            * rotate_with_axis(current_time as f32 * 45.0, 0.0, 1.0, 0.0)
            * rotate_with_axis(current_time as f32 * 81.0, 1.0, 0.0, 0.0);

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

            gl::Viewport(0, 0, 512, 512);
            gl::ClearBufferfv(gl::COLOR, 0, color::Green.as_ptr());
            gl::ClearBufferfi(gl::DEPTH_STENCIL, 0, 1.0f32, 0);

            gl::UseProgram(self.program1);

            gl::UniformMatrix4fv(
                self.proj_location,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );
            gl::UniformMatrix4fv(
                self.mv_location,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const GLfloat,
            );
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            let AppConfig { width, height, .. } = self.info();
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);
            gl::ClearBufferfv(gl::COLOR, 0, blue);

            gl::BindTexture(gl::TEXTURE_2D, self.color_texture);

            gl::UseProgram(self.program2);

            gl::UniformMatrix4fv(
                self.proj_location2,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );
            gl::UniformMatrix4fv(
                self.mv_location2,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const GLfloat,
            );

            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.program1);
            gl::DeleteProgram(self.program2);
            gl::DeleteBuffers(1, &self.index_buffer);
            gl::DeleteBuffers(1, &self.position_buffer);
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteTextures(1, &self.color_texture);
            gl::DeleteTextures(1, &self.depth_texture);
        }
    }
}

fn main() {
    BasicFboApp::default().run();
}
