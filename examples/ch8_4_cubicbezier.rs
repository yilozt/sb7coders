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
struct CubicbezierApp {
    tess_program: GLuint,
    draw_cp_program: GLuint,
    patch_vao: GLuint,
    patch_buffer: GLuint,
    cage_indices: GLuint,
    patch_data: [Vec3; 16],

    show_points: bool,
    show_cage: bool,
    wireframe: bool,
    paused: bool,

    uniforms: Uniforms,
}

#[derive(Default)]
struct Uniforms {
    patch: PatchUniforms,
    control_point: ControlPointUniforms,
}

#[derive(Default)]
struct PatchUniforms {
    mv_matrix: GLint,
    proj_matrix: GLint,
    mvp: GLint,
}

#[derive(Default)]
struct ControlPointUniforms {
    draw_color: GLint,
    mvp: GLint,
}

impl Application for CubicbezierApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Cubic Bezier Patch".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        self.load_shaders();

        unsafe {
            gl::GenVertexArrays(1, &mut self.patch_vao);
            gl::BindVertexArray(self.patch_vao);

            gl::GenBuffers(1, &mut self.patch_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.patch_buffer);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&self.patch_data) as GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
            gl::EnableVertexAttribArray(0);

            #[rustfmt::skip]
            let indices: &[GLushort] = &[
                0, 1, 1, 2, 2, 3,
                4, 5, 5, 6, 6, 7,
                8, 9, 9, 10, 10, 11,
                12, 13, 13, 14, 14, 15,

                0, 4, 4, 8, 8, 12,
                1, 5, 5, 9, 9, 13,
                2, 6, 6, 10, 10, 14,
                3, 7, 7, 11, 11, 15
            ];

            gl::GenBuffers(1, &mut self.cage_indices);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.cage_indices);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(indices) as GLsizeiptr,
                indices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
        }
    }

    fn render(&self, current_time: f64) {
        unsafe {
            let gray = [0.1f32, 0.1, 0.1, 0.1];
            let one = 1.0f32;

            static mut LAST_TIME: f64 = 0.0;
            static mut TOTAL_TIME: f64 = 0.0;

            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let t = TOTAL_TIME as f32;

            #[rustfmt::skip]
            static PATCH_INITIALIZER: &[f32] = &[
                -1.0f32,  -1.0f32,  0.0f32,
                -0.33f32, -1.0f32,  0.0f32,
                 0.33f32, -1.0f32,  0.0f32,
                 1.0f32,  -1.0f32,  0.0f32,

                -1.0f32,  -0.33f32, 0.0f32,
                -0.33f32, -0.33f32, 0.0f32,
                 0.33f32, -0.33f32, 0.0f32,
                 1.0f32,  -0.33f32, 0.0f32,

                -1.0f32,   0.33f32, 0.0f32,
                -0.33f32,  0.33f32, 0.0f32,
                 0.33f32,  0.33f32, 0.0f32,
                 1.0f32,   0.33f32, 0.0f32,

                -1.0f32,   1.0f32,  0.0f32,
                -0.33f32,  1.0f32,  0.0f32,
                 0.33f32,  1.0f32,  0.0f32,
                 1.0f32,   1.0f32,  0.0f32,
            ];

            gl::ClearBufferfv(gl::COLOR, 0, gray.as_ptr());
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::Enable(gl::DEPTH_TEST);

            let p = gl::MapBufferRange(
                gl::ARRAY_BUFFER,
                0,
                size_of_val(&self.patch_data) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
            ) as *mut Vec3;
            std::ptr::copy(
                PATCH_INITIALIZER.as_ptr() as *mut u8,
                p as *mut u8,
                size_of_val(&self.patch_data),
            );

            for i in 0..16 {
                let fi = i as f32 / 16.0;
                (*p.add(i))[2] = (t * (0.2 + fi * 0.3)).sin();
            }

            gl::UnmapBuffer(gl::ARRAY_BUFFER);

            gl::BindVertexArray(self.patch_vao);

            gl::UseProgram(self.tess_program);

            let proj_matrix = perspective(
                50.0f32,
                {
                    let info = self.info();
                    info.width as f32 / info.height as f32
                },
                1.0f32,
                1000.0f32,
            );

            let mv_matrix = translate(0.0, 0.0, -4.0)
                * rotate_with_axis(t * 10.0, 0.0, 1.0, 0.0)
                * rotate_with_axis(t * 17.0, 1.0, 0.0, 0.0);

            let mvp = proj_matrix * mv_matrix;

            gl::UniformMatrix4fv(
                self.uniforms.patch.mv_matrix,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const GLfloat,
            );
            gl::UniformMatrix4fv(
                self.uniforms.patch.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );
            gl::UniformMatrix4fv(
                self.uniforms.patch.mvp,
                1,
                gl::FALSE,
                addr_of!(mvp) as *const GLfloat,
            );

            match self.wireframe {
                true => gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE),
                false => gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL),
            };

            gl::PatchParameteri(gl::PATCH_VERTICES, 16);
            gl::DrawArrays(gl::PATCHES, 0, 16);

            gl::UseProgram(self.draw_cp_program);
            gl::UniformMatrix4fv(
                self.uniforms.control_point.mvp,
                1,
                gl::FALSE,
                addr_of!(mvp) as *const GLfloat,
            );

            if self.show_points {
                let color = vec4!(0.2f32, 0.7f32, 0.9f32, 1.0f32);
                gl::PointSize(9.0);
                gl::Uniform4fv(
                    self.uniforms.control_point.draw_color,
                    1,
                    addr_of!(color) as *const GLfloat,
                );
                gl::DrawArrays(gl::POINTS, 0, 16);
            }

            if self.show_cage {
                let color = vec4!(0.7f32, 0.9f32, 0.2f32, 1.0f32);
                gl::Uniform4fv(
                    self.uniforms.control_point.draw_color,
                    1,
                    addr_of!(color) as *const GLfloat,
                );
                gl::DrawElements(gl::LINES, 48, gl::UNSIGNED_SHORT, null());
            }

            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
    }

    fn on_key(&mut self, _key: glfw::Key, _press: glfw::Action) {
        if let glfw::Action::Press = _press {
            match _key {
                glfw::Key::C => self.show_cage = !self.show_cage,
                glfw::Key::X => self.show_points = !self.show_points,
                glfw::Key::W => self.wireframe = !self.wireframe,
                glfw::Key::P => self.paused = !self.paused,
                glfw::Key::R => self.load_shaders(),
                _ => {}
            }
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.tess_program);
            gl::DeleteProgram(self.draw_cp_program);
            gl::DeleteVertexArrays(1, &self.patch_vao);
            gl::DeleteBuffers(1, &self.patch_buffer);
            gl::DeleteBuffers(1, &self.cage_indices);
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        if let Some(win) = imgui::Window::new("Help")
            .position([2.0, 2.0], imgui::Condition::Once)
            .begin(ui)
        {
            ui.text(format!(
                "W: Toggle wireframe, wireframe: {:?}",
                self.wireframe
            ));
            ui.text(format!(
                "C: Toggle control cage, show_cage: {:?}",
                self.show_cage
            ));
            ui.text(format!(
                "X: Toggle control points, show_points: {:?}",
                self.show_points
            ));
            ui.text(format!("P: Paused, paused: {:?}", self.paused));
            win.end();
        }
    }
}

impl CubicbezierApp {
    fn load_shaders(&mut self) {
        unsafe {
            if self.tess_program != 0 {
                gl::DeleteProgram(self.tess_program);
            }

            self.tess_program = program::link_from_shaders(
                &[
                    shader::load(
                        "media/shaders/cubicbezier/cubicbezier.vs.glsl",
                        gl::VERTEX_SHADER,
                        true,
                    ),
                    shader::load(
                        "media/shaders/cubicbezier/cubicbezier.tcs.glsl",
                        gl::TESS_CONTROL_SHADER,
                        true,
                    ),
                    shader::load(
                        "media/shaders/cubicbezier/cubicbezier.tes.glsl",
                        gl::TESS_EVALUATION_SHADER,
                        true,
                    ),
                    shader::load(
                        "media/shaders/cubicbezier/cubicbezier.fs.glsl",
                        gl::FRAGMENT_SHADER,
                        true,
                    ),
                ],
                true,
            );

            let get_loc = |program, name| {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(program, name.as_ptr())
            };

            self.uniforms.patch.mv_matrix = get_loc(self.tess_program, "mv_matrix");
            self.uniforms.patch.proj_matrix = get_loc(self.tess_program, "proj_matrix");
            self.uniforms.patch.mvp = get_loc(self.tess_program, "mvp");

            if self.draw_cp_program != 0 {
                gl::DeleteProgram(self.draw_cp_program);
            }

            self.draw_cp_program = program::link_from_shaders(
                &[
                    shader::load(
                        "media/shaders/cubicbezier/draw-control-points.vs.glsl",
                        gl::VERTEX_SHADER,
                        true,
                    ),
                    shader::load(
                        "media/shaders/cubicbezier/draw-control-points.fs.glsl",
                        gl::FRAGMENT_SHADER,
                        true,
                    ),
                ],
                true,
            );

            self.uniforms.control_point.draw_color = get_loc(self.draw_cp_program, "draw_color");
            self.uniforms.control_point.mvp = get_loc(self.draw_cp_program, "mvp");
        }
    }
}

fn main() {
    CubicbezierApp::default().run()
}
