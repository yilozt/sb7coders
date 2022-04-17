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
struct App {
    render_prog: u32,
    border_prog: u32,
    vao: u32,
    vertices_buf: u32,
    indices_buf: u32,
    uniforms: Uniforms,
}

#[derive(Default)]
struct Uniforms {
    mv_mat: i32,
    proj_mat: i32,
}

impl Application for App {
    fn startup(&mut self) {
        let vs = shader::load("my_test/shaders/render.vert", gl::VERTEX_SHADER, true);
        let fs = shader::load("my_test/shaders/render.frag", gl::FRAGMENT_SHADER, true);
        self.render_prog = program::link_from_shaders(&[vs, fs], true);

        let vs = shader::load("my_test/shaders/render.vert", gl::VERTEX_SHADER, true);
        let fs = shader::load("my_test/shaders/border.frag", gl::FRAGMENT_SHADER, true);
        self.border_prog = program::link_from_shaders(&[vs, fs], true);

        fn get_loc(program: u32, name: &str) -> i32 {
            let name = std::ffi::CString::new(name).unwrap();
            unsafe { gl::GetUniformLocation(program, name.as_ptr()) }
        }

        self.uniforms.mv_mat = get_loc(self.render_prog, "mv_mat");
        self.uniforms.proj_mat = get_loc(self.render_prog, "proj_mat");

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

        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            gl::GenBuffers(1, &mut self.vertices_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertices_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertex_positions) as GLsizeiptr,
                vertex_positions.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
            gl::EnableVertexAttribArray(0);

            gl::GenBuffers(1, &mut self.indices_buf);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indices_buf);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(vertex_indices) as GLsizeiptr,
                vertex_indices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
    }

    fn render(&self, current_time: f64) {
        let t = current_time as f32;
        let black = [0.0, 0.0, 0.0, 0.0f32].as_ptr();

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &1.0);

            gl::Enable(gl::STENCIL_TEST);

            let proj_mat = perspective(
                50.0,
                {
                    let info = self.info();
                    info.width as f32 / info.height as f32
                },
                0.1,
                1000.0,
            );

            let render_cube = |i: i32, j: i32| {
                gl::ClearBufferiv(gl::STENCIL, 0, &0);

                let mv_mat = lookat(
                    vec3!(4.0 * t.cos(), 2.0, 4.0 * t.sin()),
                    vec3!(0.0, 0.0, 0.0),
                    vec3!(0.0, 1.0, 0.0),
                ) * translate(i as f32 * 1.1, 0.0, j as f32 * 1.1);

                gl::UseProgram(self.render_prog);
                gl::UniformMatrix4fv(
                    self.uniforms.mv_mat,
                    1,
                    gl::FALSE,
                    addr_of!(mv_mat) as *const f32,
                );
                gl::UniformMatrix4fv(
                    self.uniforms.proj_mat,
                    1,
                    gl::FALSE,
                    addr_of!(proj_mat) as *const f32,
                );
                gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
                gl::StencilOp(gl::KEEP, gl::REPLACE, gl::REPLACE);
                gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, null());

                gl::UseProgram(self.border_prog);
                let mv_mat = mv_mat * scale(1.3, 1.3, 1.3);
                gl::UniformMatrix4fv(
                    self.uniforms.mv_mat,
                    1,
                    gl::FALSE,
                    addr_of!(mv_mat) as *const f32,
                );
                gl::UniformMatrix4fv(
                    self.uniforms.proj_mat,
                    1,
                    gl::FALSE,
                    addr_of!(proj_mat) as *const f32,
                );
                gl::StencilFunc(gl::GREATER, 1, 0xFF);
                gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
                gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, null());
            };

            for i in -1..2 {
                for j in -1..2 {
                    render_cube(i, j);
                }
            }

            gl::Disable(gl::STENCIL);

            gl::UseProgram(self.render_prog);
            let mv_mat = lookat(
                vec3!(4.0 * t.cos(), 2.0, 4.0 * t.sin()),
                vec3!(0.0, 0.0, 0.0),
                vec3!(0.0, 1.0, 0.0),
            ) * scale(4.5, 0.2, 4.5);
            gl::UniformMatrix4fv(
                self.uniforms.mv_mat,
                1,
                gl::FALSE,
                addr_of!(mv_mat) as *const f32,
            );
            gl::UniformMatrix4fv(
                self.uniforms.proj_mat,
                1,
                gl::FALSE,
                addr_of!(proj_mat) as *const f32,
            );
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, null());
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.border_prog);
            gl::DeleteProgram(self.render_prog);
            gl::DeleteBuffers(1, &self.vertices_buf);
            gl::DeleteBuffers(1, &self.indices_buf);
        }
    }
}

fn main() {
    App::default().run();
}
