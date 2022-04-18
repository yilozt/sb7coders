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
struct BlendMatrixApp {
    program: GLuint,
    vao: GLuint,
    position_buffer: GLuint,
    index_buffer: GLuint,
    mv_location: GLint,
    proj_location: GLint,
}

impl Application for BlendMatrixApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Blending Functions".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_source = cstring(
            r#"
            #version 410 core

            in vec4 position;

            out VS_OUT
            {
                vec4 color0;
                vec4 color1;
            } vs_out;

            uniform mat4 mv_matrix;
            uniform mat4 proj_matrix;

            void main(void)
            {
                gl_Position = proj_matrix * mv_matrix * position;
                vs_out.color0 = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
                vs_out.color1 = vec4(0.5, 0.5, 0.5, 0.0) - position * 2.0;
            }"#,
        );

        let fs_source = cstring(
            r#"
            #version 410 core

            layout (location = 0, index = 0) out vec4 color0;
            layout (location = 0, index = 1) out vec4 color1;

            in VS_OUT
            {
                vec4 color0;
                vec4 color1;
            } fs_in;

            void main(void)
            {
                color0 = vec4(fs_in.color0.xyz, 1.0);
                color1 = vec4(fs_in.color0.xyz, 1.0);
            }"#,
        );

        unsafe {
            self.program = gl::CreateProgram();
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
            gl::CompileShader(fs);

            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, fs);

            gl::LinkProgram(self.program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let get_loc = |name| {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(self.program, name.as_ptr())
            };

            self.mv_location = get_loc("mv_matrix");
            self.proj_location = get_loc("proj_matrix");

            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            #[rustfmt::skip]
            let vertex_indices : &[GLushort] = &[
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
            let vertex_positions: &[f32] = &[
                -0.25f32, -0.25f32, -0.25f32,
                -0.25f32,  0.25f32, -0.25f32,
                 0.25f32, -0.25f32, -0.25f32,
                 0.25f32,  0.25f32, -0.25f32,
                 0.25f32, -0.25f32,  0.25f32,
                 0.25f32,  0.25f32,  0.25f32,
                -0.25f32, -0.25f32,  0.25f32,
                -0.25f32,  0.25f32,  0.25f32,
            ];

            gl::GenBuffers(1, &mut self.position_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.position_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertex_positions) as GLsizeiptr,
                vertex_positions.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
            gl::EnableVertexAttribArray(0);

            gl::GenBuffers(1, &mut self.index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(vertex_indices) as GLsizeiptr,
                vertex_indices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );

            gl::Enable(gl::CULL_FACE);
            // gl::FrontFace(gl::CW);

            // gl::Enable(gl::DEPTH_TEST);
            // gl::DepthFunc(gl::LEQUAL);
        }
    }

    fn render(&mut self, current_time: f64) {
        let orange = [0.6, 0.4, 0.1, 1.0f32].as_ptr();
        let one = 1.0f32;
        let t = current_time as f32;

        static BLEND_FUNC: &[GLenum] = &[
            gl::ZERO,
            gl::ONE,
            gl::SRC_COLOR,
            gl::ONE_MINUS_SRC_COLOR,
            gl::DST_COLOR,
            gl::ONE_MINUS_DST_COLOR,
            gl::SRC_ALPHA,
            gl::ONE_MINUS_SRC_ALPHA,
            gl::DST_ALPHA,
            gl::ONE_MINUS_DST_ALPHA,
            gl::CONSTANT_COLOR,
            gl::ONE_MINUS_CONSTANT_COLOR,
            gl::CONSTANT_ALPHA,
            gl::ONE_MINUS_CONSTANT_ALPHA,
            gl::SRC_ALPHA_SATURATE,
            gl::SRC1_COLOR,
            gl::ONE_MINUS_SRC1_COLOR,
            gl::SRC1_ALPHA,
            gl::ONE_MINUS_SRC1_ALPHA,
        ];
        static X_SCALE: f32 = 20.0 / BLEND_FUNC.len() as f32;
        static Y_SCALE: f32 = 16.0 / BLEND_FUNC.len() as f32;

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, orange);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::UseProgram(self.program);

            let proj_matrix = perspective(
                50.0,
                {
                    let info = self.info();
                    info.width as f32 / info.height as f32
                },
                0.1,
                1000.0,
            );
            gl::UniformMatrix4fv(
                self.proj_location,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );

            gl::Enable(gl::BLEND);
            gl::BlendColor(0.2, 0.5, 0.7, 0.5);

            for j in 0..BLEND_FUNC.len() {
                for i in 0..BLEND_FUNC.len() {
                    let mv_matrix =
                        translate(9.5 - X_SCALE * i as f32, 7.5 - Y_SCALE * j as f32, -18.0)
                            * rotate_with_axis(t * -45.0, 0.0, 1.0, 0.0)
                            * rotate_with_axis(t * -21.0, 1.0, 0.0, 0.0);
                    gl::UniformMatrix4fv(
                        self.mv_location,
                        1,
                        gl::FALSE,
                        addr_of!(mv_matrix) as *const GLfloat,
                    );
                    gl::BlendFunc(BLEND_FUNC[i], BLEND_FUNC[j]);
                    gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, null());
                }
            }
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(1, &self.index_buffer);
            gl::DeleteBuffers(1, &self.position_buffer);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

fn main() {
    BlendMatrixApp::default().run()
}
