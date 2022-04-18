/*
 * Copyright � 2012-2015 Graham Sellers
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

use std::ffi::CString;

use sb7::prelude::*;

#[derive(Default)]
struct App {
    vao: u32,
    position_buf: u32,
    index_buf: u32,
    prog: u32,
    proj_matrix: Mat4,
    mv_location: i32,
    proj_location: i32,
}

impl Application for App {
    fn startup(&mut self) {
        let vs_src = "
            #version 460 core
            
            in vec4 position;

            out VS_OUT {
                vec4 color;
            } vs_out;

            uniform mat4 mv_matrix; 
            uniform mat4 proj_matrix;

            void main(void) {
                gl_Position = proj_matrix * mv_matrix * position;
                vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
            }
        ";

        let fs_src = "
            #version 460 core

            out vec4 color;

            in VS_OUT {
                vec4 color;
            } fs_in;

            void main(void) {
                color = fs_in.color;
            }
        ";

        #[rustfmt::skip]
        let vertex_positions = [
            -0.25f32, -0.25f32, -0.25f32,
            -0.25f32,  0.25f32, -0.25f32,
             0.25f32, -0.25f32, -0.25f32,
             0.25f32,  0.25f32, -0.25f32,
             0.25f32, -0.25f32,  0.25f32,
             0.25f32,  0.25f32,  0.25f32,
            -0.25f32, -0.25f32,  0.25f32,
            -0.25f32,  0.25f32,  0.25f32,
        ];

        #[rustfmt::skip]
        let vertex_indices = [
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
            7, 3, 1u8
        ];

        unsafe {
            gl::CreateVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            gl::CreateBuffers(1, &mut self.position_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.position_buf);
            gl::NamedBufferData(
                self.position_buf,
                size_of_val(&vertex_positions) as _,
                vertex_positions.as_ptr() as _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * 4, null());
            gl::EnableVertexAttribArray(0);

            gl::CreateBuffers(1, &mut self.index_buf);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buf);
            gl::NamedBufferData(
                self.index_buf,
                size_of_val(&vertex_indices) as _,
                vertex_indices.as_ptr() as _,
                gl::STATIC_DRAW,
            );

            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            let vs_src = CString::new(vs_src).unwrap();
            gl::ShaderSource(vs, 1, &vs_src.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fs_src = CString::new(fs_src).unwrap();
            gl::ShaderSource(fs, 1, &fs_src.as_ptr(), null());
            gl::CompileShader(fs);

            self.prog = gl::CreateProgram();
            gl::AttachShader(self.prog, vs);
            gl::AttachShader(self.prog, fs);
            gl::LinkProgram(self.prog);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let c_str = CString::new("mv_matrix").unwrap();
            self.mv_location = gl::GetUniformLocation(self.prog, c_str.as_ptr());

            let c_str = CString::new("proj_matrix").unwrap();
            self.proj_location = gl::GetUniformLocation(self.prog, c_str.as_ptr());

            let AppConfig { width, height, .. } = AppConfig::default();
            self.on_resize(width as _, height as _);

            gl::Enable(gl::DEPTH_TEST);
            // gl::Enable(gl::CULL_FACE);
            gl::DepthFunc(gl::LEQUAL);
        }
    }

    fn on_resize(&mut self, w: i32, h: i32) {
        let aspect = w as f32 / h as f32;
        self.proj_matrix = perspective(20.0, aspect, 0.1, 1000.0);

        unsafe {
            gl::UseProgram(self.prog);
            gl::UniformMatrix4fv(
                self.proj_location,
                1,
                gl::FALSE,
                addr_of!(self.proj_matrix) as _,
            );
        }
    }

    fn render(&mut self, current_time: f64) {
        let green = [0.0, 0.25, 0.0, 1.0f32].as_ptr();
        let one = 1.0f32;

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, green);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::UseProgram(self.prog);

            let many_cubes = true;

            if many_cubes {
                for i in 0..24 {
                    let f = i as f32 + current_time as f32 * 0.3;
                    let mv_matrix = translate(0., 0., -20.)
                        * rotate_with_axis(current_time as f32 * 45., 0., 1., 0.)
                        * rotate_with_axis(current_time as f32 * 21., 1., 0., 0.0)
                        * translate(
                            (2.1 * f).sin() * 2.,
                            (1.7 * f).cos() * 2.,
                            (1.3 * f).sin() * (1.5 * f).cos() * 2.,
                        );
                    gl::UniformMatrix4fv(self.mv_location, 1, gl::FALSE, addr_of!(mv_matrix) as _);
                    gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_BYTE, 0 as _);
                }
            } else {
                let f = current_time as f32 * 0.3;
                let mv_matrix = translate(0., 0., -4.)
                    * translate(
                        (2.1 * f).sin() * 2.,
                        (1.7 * f).cos() * 2.,
                        (1.3 * f).sin() * (1.5 * f).cos() * 2.,
                    )
                    * rotate_with_axis(current_time as f32 * 45., 0., 1., 0.)
                    * rotate_with_axis(current_time as f32 * 81., 1., 0., 0.0);
                gl::UniformMatrix4fv(self.mv_location, 1, gl::FALSE, addr_of!(mv_matrix) as _);
                gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_BYTE, 0 as _);
            }
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.index_buf);
            gl::DeleteBuffers(1, &self.position_buf);
            gl::DeleteProgram(self.prog);
        }
    }
}

fn main() {
    App::default().run();
}
