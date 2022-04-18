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

const POSITION_A: usize = 0;
const POSITION_B: usize = 1;
const VELOCITY_A: usize = 2;
const CONNECTION: usize = 4;

const POINTS_X: usize = 50;
const POINTS_Y: usize = 50;
const POINTS_TOTAL: usize = POINTS_X * POINTS_Y;
const CONNECTIONS_TOTAL: usize = (POINTS_X - 1) * POINTS_Y + (POINTS_Y - 1) * POINTS_X;

#[derive(Default)]
struct SpringMassApp {
    m_vao: [GLuint; 2],
    m_vbo: [GLuint; 5],
    m_index_buffer: GLuint,
    m_pos_tbo: [GLuint; 2],
    m_update_program: GLuint,
    m_render_program: GLuint,
    m_iteration_index: GLuint,

    draw_points: bool,
    draw_lines: bool,
    iterations_per_frame: u32,
}

impl SpringMassApp {
    fn new() -> Self {
        Self {
            draw_lines: true,
            draw_points: true,
            iterations_per_frame: 16,
            ..Default::default()
        }
    }

    fn load_shaders(&mut self) {
        let vs = shader::load(
            "media/shaders/springmass/update.vs.glsl",
            gl::VERTEX_SHADER,
            true,
        );

        unsafe {
            if self.m_update_program != 0 {
                gl::DeleteProgram(self.m_update_program);
            }

            self.m_update_program = gl::CreateProgram();
            gl::AttachShader(self.m_update_program, vs);

            let tf_varyings = [
                std::ffi::CString::new("tf_position_mass").unwrap(),
                std::ffi::CString::new("tf_velocity").unwrap(),
            ];
            let tf_varyings: Vec<_> = tf_varyings.iter().map(|s| s.as_ptr()).collect();

            gl::TransformFeedbackVaryings(
                self.m_update_program,
                2,
                tf_varyings.as_ptr(),
                gl::SEPARATE_ATTRIBS,
            );

            gl::LinkProgram(self.m_update_program);

            gl::DeleteShader(vs);

            let vs = shader::load(
                "media/shaders/springmass/render.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/springmass/render.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );

            if self.m_render_program != 0 {
                gl::DeleteShader(self.m_render_program);
            }
            self.m_render_program = gl::CreateProgram();
            gl::AttachShader(self.m_render_program, vs);
            gl::AttachShader(self.m_render_program, fs);

            gl::LinkProgram(self.m_render_program);
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }
    }
}

impl Application for SpringMassApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Spring-Mass Simulator".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        self.load_shaders();

        unsafe {
            let mut initial_positions = [Vec4::default(); POINTS_TOTAL];
            let initial_velocities = [Vec3::default(); POINTS_TOTAL];
            let mut connection_vectors = [IVec4::default(); POINTS_TOTAL];

            let mut n = 0;
            for j in 0..POINTS_Y {
                let fj = j as f32 / POINTS_Y as f32;
                for i in 0..POINTS_X {
                    let fi = i as f32 / POINTS_X as f32;

                    initial_positions[n] = vec4!(
                        (fi - 0.5) * POINTS_X as f32,
                        (fj - 0.5) * POINTS_Y as f32,
                        0.6 * fi.sin() * fj.cos(),
                        1.0
                    );

                    connection_vectors[n] = vec4!(-1);

                    if j != (POINTS_Y - 1) {
                        if i != 0 {
                            connection_vectors[n][0] = n as i32 - 1;
                        }

                        if j != 0 {
                            connection_vectors[n][1] = n as i32 - POINTS_X as i32;
                        }

                        if i != POINTS_X - 1 {
                            connection_vectors[n][2] = n as i32 + 1;
                        }

                        if j != POINTS_Y - 1 {
                            connection_vectors[n][3] = n as i32 + POINTS_X as i32;
                        }
                    }
                    n += 1;
                }
            }

            gl::GenVertexArrays(2, self.m_vao.as_mut_ptr());
            gl::GenBuffers(5, self.m_vbo.as_mut_ptr());

            for i in 0..2 {
                gl::BindVertexArray(self.m_vao[i]);

                gl::BindBuffer(gl::ARRAY_BUFFER, self.m_vbo[POSITION_A + i]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    size_of_val(&initial_positions) as _,
                    initial_positions.as_ptr() as _,
                    gl::DYNAMIC_COPY,
                );
                gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 0, null());
                gl::EnableVertexAttribArray(0);

                gl::BindBuffer(gl::ARRAY_BUFFER, self.m_vbo[VELOCITY_A + i]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    size_of_val(&initial_velocities) as _,
                    initial_velocities.as_ptr() as _,
                    gl::DYNAMIC_COPY,
                );
                gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());
                gl::EnableVertexAttribArray(1);

                gl::BindBuffer(gl::ARRAY_BUFFER, self.m_vbo[CONNECTION]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    size_of_val(&connection_vectors) as _,
                    connection_vectors.as_ptr() as _,
                    gl::DYNAMIC_COPY,
                );
                gl::VertexAttribIPointer(2, 4, gl::INT, 0, null());
                gl::EnableVertexAttribArray(2);
            }

            gl::GenTextures(2, self.m_pos_tbo.as_mut_ptr());
            gl::BindTexture(gl::TEXTURE_BUFFER, self.m_pos_tbo[0]);
            gl::TexBuffer(
                gl::TEXTURE_BUFFER,
                gl::RGBA32F,
                self.m_vbo[POSITION_A as usize],
            );
            gl::BindTexture(gl::TEXTURE_BUFFER, self.m_pos_tbo[1]);
            gl::TexBuffer(
                gl::TEXTURE_BUFFER,
                gl::RGBA32F,
                self.m_vbo[POSITION_B as usize],
            );

            let lines = (POINTS_X - 1) * (POINTS_Y) + (POINTS_Y - 1) * POINTS_X;

            gl::GenBuffers(1, &mut self.m_index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.m_index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (lines * 2 * size_of::<i32>()) as GLsizeiptr,
                null(),
                gl::STATIC_DRAW,
            );

            let e = std::slice::from_raw_parts_mut(
                gl::MapBufferRange(
                    gl::ELEMENT_ARRAY_BUFFER,
                    0,
                    (lines * 2 * size_of::<i32>()) as GLsizeiptr,
                    gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
                ) as *mut i32,
                lines as usize * 2 * size_of::<i32>(),
            );
            let mut index = 0;
            for j in 0..POINTS_Y {
                for i in 0..POINTS_X - 1 {
                    e[index] = (i + j * POINTS_X) as i32;
                    index += 1;
                    e[index] = (1 + i + j * POINTS_X) as i32;
                    index += 1;
                }
            }
            for i in 0..POINTS_X {
                for j in 0..POINTS_Y - 1 {
                    e[index] = (i + j * POINTS_X) as i32;
                    index += 1;
                    e[index] = (POINTS_X + i + j * POINTS_X) as i32;
                    index += 1;
                }
            }
        }
    }

    fn render(&mut self, _current_time: f64) {
        unsafe {
            gl::UseProgram(self.m_update_program);

            gl::Enable(gl::RASTERIZER_DISCARD);

            for _ in 0..self.iterations_per_frame {
                gl::BindVertexArray(self.m_vao[self.m_iteration_index as usize & 1]);
                gl::BindTexture(
                    gl::TEXTURE_BUFFER,
                    self.m_pos_tbo[self.m_iteration_index as usize & 1],
                );
                (&mut *(self as *const _ as *mut Self)).m_iteration_index += 1;
                gl::BindBufferBase(
                    gl::TRANSFORM_FEEDBACK_BUFFER,
                    0,
                    self.m_vbo[(POSITION_A + (self.m_iteration_index as usize & 1)) as usize],
                );
                gl::BindBufferBase(
                    gl::TRANSFORM_FEEDBACK_BUFFER,
                    1,
                    self.m_vbo[(VELOCITY_A + (self.m_iteration_index as usize & 1)) as usize],
                );
                gl::BeginTransformFeedback(gl::POINTS);
                gl::DrawArrays(gl::POINTS, 0, POINTS_TOTAL as _);
                gl::EndTransformFeedback();
            }

            gl::Disable(gl::RASTERIZER_DISCARD);

            let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();

            gl::ClearBufferfv(gl::COLOR, 0, black);

            gl::UseProgram(self.m_render_program);

            if self.draw_points {
                gl::PointSize(4.0);
                gl::DrawArrays(gl::POINTS, 0, POINTS_TOTAL as _);
            }

            if self.draw_lines {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.m_index_buffer);
                gl::DrawElements(
                    gl::LINES,
                    (CONNECTIONS_TOTAL * 2) as _,
                    gl::UNSIGNED_INT,
                    null(),
                );
            }
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.m_update_program);
            gl::DeleteBuffers(5, self.m_vbo.as_ptr());
            gl::DeleteVertexArrays(2, self.m_vao.as_ptr());
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        if let Some(win) = imgui::Window::new("Settings")
            .position([10.0, 10.0], imgui::Condition::Once)
            .begin(ui)
        {
            ui.checkbox("draw_points", &mut self.draw_points);
            ui.checkbox("draw_lines", &mut self.draw_lines);
            win.end()
        }
    }
}

fn main() {
    SpringMassApp::new().run();
}
