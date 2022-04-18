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

const MANY_CUBES: bool = false;

#[derive(Default)]
struct LinesmoothApp {
    program: GLuint,
    vao: GLuint,
    position_buffer: GLuint,
    index_buffer: GLuint,
    mv_location: GLint,
    proj_location: GLint,
}

impl Application for LinesmoothApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Line Smoothing".into(),
            ..AppConfig::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_src = cstring(
            r#"
            #version 410 core

            in vec4 position;

            uniform mat4 mv_matrix;
            uniform mat4 proj_matrix;

            void main(void)
            {
                gl_Position = proj_matrix * mv_matrix * position;
            }"#,
        );

        let fs_src = cstring(
            r#"
            #version 410 core

            out vec4 color;

            void main(void)
            {
                color = vec4(1.0)  ;
            }"#,
        );

        unsafe {
            self.program = gl::CreateProgram();
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_src.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_src.as_ptr(), null());
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
                7, 3, 1,
            ];

            #[rustfmt::skip]
            let vertex_positions: &[GLfloat] = &[
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
        }
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();
        let info = self.info();
        let current_time = current_time as f32;

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, black);

            gl::UseProgram(self.program);

            let proj_matrix =
                perspective(50.0, info.width as f32 / info.height as f32, 0.1, 1000.0);
            gl::UniformMatrix4fv(
                self.proj_location,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );

            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::LINE_SMOOTH);

            match MANY_CUBES {
                true => {
                    for i in 0..24 {
                        let f = i as f32 + current_time * 0.3;

                        let mv_matrix = translate(0.0, 0.0, -20.0)
                            * rotate_with_axis(current_time * 45.0, 0.0, 1.0, 0.0)
                            * rotate_with_axis(current_time * 21.0, 1.0, 0.0, 0.0)
                            * translate(
                                (2.1 * f).sin() * 2.0,
                                (1.7 * f).cos() * 2.0,
                                (1.3 * f).sin() * (1.5 * f).cos() * 2.0,
                            );
                        gl::UniformMatrix4fv(
                            self.mv_location,
                            1,
                            gl::FALSE,
                            addr_of!(mv_matrix) as *const GLfloat,
                        );
                        gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, null());
                    }
                }
                false => {
                    // let f = current_time * 0.3;
                    let current_time = 3.15f32;
                    let mv_matrix = translate(0.0, 0.0, -4.0) *
                                    /*translate((2.1 * f).sin() * 0.5,
                                              (1.7 * f).cos() * 0.5,
                                              (1.3 * f).sin() * (1.5 * f).cos() * 2.0) * */
                                    rotate_with_axis(current_time * 45.0, 0.0, 1.0, 0.0) *
                                    rotate_with_axis(current_time * 81.0, 1.0, 0.0, 0.0);
                    gl::UniformMatrix4fv(
                        self.mv_location,
                        1,
                        gl::FALSE,
                        addr_of!(mv_matrix) as *const GLfloat,
                    );
                    gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, null());
                }
            }
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.index_buffer);
            gl::DeleteBuffers(1, &self.position_buffer);
            gl::DeleteProgram(self.program);
        }
    }
}

fn main() {
    LinesmoothApp::default().run();
}
