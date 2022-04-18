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
use sb7::prelude::*;

const NUM_DRAWS: usize = 50000;

#[rustfmt::skip]
struct DrawArraysIndirectCommand {
    count:          u32,   // count of vertices in a instance
    prim_count:     u32,   // count of instances to render
    first:          u32,   // first vertices in a instance to render
    base_instance:  u32,   // first instance to render
}

#[derive(Default)]
#[rustfmt::skip]
struct App {
    render_program:         u32,
    object:                 Object,
    indirect_draw_buffer:   u32,
    draw_index_buffer:      u32,

    uniforms:               Uniforms,

    mode:                   Mode,
    paused:                 bool,
    vsync:                  bool,
}

#[derive(Default)]
#[repr(C)]
struct Uniforms {
    time: i32,
    view_matrix: i32,
    proj_matrix: i32,
    viewproj_matrix: i32,
}

#[derive(Debug)]
enum Mode {
    MultiDraw,
    SeparateDraws,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::MultiDraw
    }
}

impl Mode {
    fn toggle(&mut self) {
        *self = match self {
            Mode::MultiDraw => Mode::SeparateDraws,
            Mode::SeparateDraws => Mode::MultiDraw,
        }
    }
}

impl App {
    fn load_shaders(&mut self) {
        self.render_program = program::link_from_shaders(
            &[
                shader::load(
                    "media/shaders/multidrawindirect/render.vs.glsl",
                    gl::VERTEX_SHADER,
                    true,
                ),
                shader::load(
                    "media/shaders/multidrawindirect/render.fs.glsl",
                    gl::FRAGMENT_SHADER,
                    true,
                ),
            ],
            true,
        );

        unsafe {
            let get_location = |name| {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(self.render_program, name.as_ptr())
            };

            self.uniforms.time = get_location("time");
            self.uniforms.view_matrix = get_location("view_matrix");
            self.uniforms.proj_matrix = get_location("proj_matrix");
            self.uniforms.viewproj_matrix = get_location("viewproj_matrix");
        }
    }
}

impl Application for App {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Asteroids".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        self.load_shaders();

        self.object.load("media/objects/asteroids.sbm");

        unsafe {
            gl::BindVertexArray(self.object.get_vao());

            gl::GenBuffers(1, &mut self.indirect_draw_buffer);
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.indirect_draw_buffer);
            gl::BufferData(
                gl::DRAW_INDIRECT_BUFFER,
                (NUM_DRAWS * size_of::<DrawArraysIndirectCommand>()) as _,
                null(),
                gl::STATIC_DRAW,
            );

            let cmd: *mut DrawArraysIndirectCommand = gl::MapBufferRange(
                gl::DRAW_INDIRECT_BUFFER,
                0,
                (NUM_DRAWS * size_of::<DrawArraysIndirectCommand>()) as _,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
            ) as _;

            for i in 0..NUM_DRAWS {
                let cmd = cmd.add(i);
                ((*cmd).first, (*cmd).count) = self
                    .object
                    .get_sub_object_info(i % self.object.get_sub_object_count() as usize);
                (*cmd).prim_count = 1;
                (*cmd).base_instance = i as _;
            }

            gl::UnmapBuffer(gl::DRAW_INDIRECT_BUFFER);

            gl::BindVertexArray(self.object.get_vao());

            gl::GenBuffers(1, &mut self.draw_index_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.draw_index_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (NUM_DRAWS * size_of::<u32>()) as _,
                null(),
                gl::STATIC_DRAW,
            );

            let draw_index: *mut u32 = gl::MapBufferRange(
                gl::ARRAY_BUFFER,
                0,
                (NUM_DRAWS * size_of::<u32>()) as _,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
            ) as _;

            for i in 0..NUM_DRAWS {
                *draw_index.add(i) = i as u32;
            }

            gl::UnmapBuffer(gl::ARRAY_BUFFER);

            gl::VertexAttribIPointer(10, 1, gl::UNSIGNED_INT, 0, 0 as _);
            gl::VertexAttribDivisor(10, 1);
            gl::EnableVertexAttribArray(10);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);

            gl::Enable(gl::CULL_FACE);
        }
    }

    fn render(&mut self, current_time: f64) {
        let one = 1.0f32;
        let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();

        static mut LAST_TIME: f64 = 0.0;
        static mut TOTAL_TIME: f64 = 0.0;

        unsafe {
            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let t = TOTAL_TIME as f32;
            // let i = (TOTAL_TIME * 3.) as i32;

            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            let view_matrix = lookat(
                vec3!(
                    100.0 * (t * 0.023).cos(),
                    100.0 * (t * 0.023).cos(),
                    300.0 * (t * 0.037).sin() - 600.0
                ),
                vec3!(0., 0., 260.),
                vec3!(0.1 - (t * 0.1).cos() * 0.3, 1.0, 0.0).normalize(),
            );
            let AppConfig { width, height, .. } = self.info();
            let proj_matrix = perspective(50., width as f32 / height as f32, 0.1, 2000.0);
            let viewproj_matrix = proj_matrix * view_matrix;

            gl::UseProgram(self.render_program);

            gl::Uniform1f(self.uniforms.time, t);
            gl::UniformMatrix4fv(
                self.uniforms.view_matrix,
                1,
                gl::FALSE,
                addr_of!(view_matrix) as _,
            );
            gl::UniformMatrix4fv(
                self.uniforms.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as _,
            );
            gl::UniformMatrix4fv(
                self.uniforms.viewproj_matrix,
                1,
                gl::FALSE,
                addr_of!(viewproj_matrix) as _,
            );

            gl::BindVertexArray(self.object.get_vao());

            match self.mode {
                Mode::MultiDraw => {
                    gl::MultiDrawArraysIndirect(gl::TRIANGLES, null(), NUM_DRAWS as _, 0);
                }
                Mode::SeparateDraws => {
                    for i in 0..NUM_DRAWS {
                        let (first, count) = self
                            .object
                            .get_sub_object_info(i % self.object.get_sub_object_count() as usize);
                        gl::DrawArraysInstancedBaseInstance(
                            gl::TRIANGLES,
                            first as _,
                            count as _,
                            1,
                            i as _,
                        );
                    }
                }
            }
        }
    }

    #[rustfmt::skip]
    fn ui(&mut self, ui: &imgui::Ui) {
        if let Some(win) = imgui::Window::new("Settings")
            .position([10., 10.], imgui::Condition::Appearing)
            .begin(ui)
        {
            #[inline(always)]
            fn toggle_btn(id: i32, ui: &imgui::Ui, action: &mut dyn FnMut() -> ()) {
                let id = ui.push_id(id);
                if ui.button("Toggle") {
                    action();
                };
                id.end();
            }

            toggle_btn(0, &ui, &mut|| {
                self.mode.toggle();
            });
            ui.same_line(); ui.text(format!("Current Mode: {:?}", self.mode));

            toggle_btn(1, &ui,&mut || {
                self.paused = !self.paused;
            });
            ui.same_line(); ui.text(format!("Paused: {:?}", self.paused));

            toggle_btn(2, &ui,&mut || {
                self.vsync = !self.vsync;
                self.set_vsync(self.vsync);
            });
            ui.same_line(); ui.text(format!("Vsync: {:?}", self.vsync));

            win.end();
        };
    }
}

fn main() {
    App::default().run();
}
