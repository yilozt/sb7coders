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
struct Uniforms {
    proj_matrix: GLint,
    mv_matrix: GLint,
    clip_plane: GLint,
    clip_sphere: GLint,
}

#[derive(Default)]
struct ClipDistanceApp {
    render_program: GLuint,
    paused: bool,
    object: Object,
    uniforms: Uniforms,
}

impl ClipDistanceApp {
    fn load_shaders(&mut self) {
        unsafe {
            if self.render_program != 0 {
                gl::DeleteProgram(self.render_program);
                self.render_program = 0;
            }

            self.render_program = program::link_from_shaders(
                &[
                    shader::load(
                        "media/shaders/clipdistance/render.vs.glsl",
                        gl::VERTEX_SHADER,
                        true,
                    ),
                    shader::load(
                        "media/shaders/clipdistance/render.fs.glsl",
                        gl::FRAGMENT_SHADER,
                        true,
                    ),
                ],
                true,
            );

            let get_loc = |name: &str| {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(self.render_program, name.as_ptr())
            };

            self.uniforms.proj_matrix = get_loc("proj_matrix");
            self.uniforms.mv_matrix = get_loc("mv_matrix");
            self.uniforms.clip_plane = get_loc("clip_plane");
            self.uniforms.clip_sphere = get_loc("clip_sphere");
        }
    }
}

impl Application for ClipDistanceApp {
    fn startup(&mut self) {
        self.load_shaders();
        self.object.load("media/objects/dragon.sbm");
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();
        let one = 1.0f32;

        static mut LAST_TIME: f64 = 0.0;
        static mut TOTAL_TIME: f64 = 0.0;

        unsafe {
            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let f = TOTAL_TIME as f32;

            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::UseProgram(self.render_program);

            let proj_matrix = perspective(
                50.0f32,
                {
                    let AppConfig { width, height, .. } = self.info();
                    width as f32 / height as f32
                },
                0.1f32,
                1000.0f32,
            );

            let mv_matrix = translate(0.0f32, 0.0f32, -15.0f32)
                * rotate_with_axis(f * 0.34f32, 0.0f32, 1.0f32, 0.0f32)
                * translate(0.0f32, -4.0f32, 0.0f32);

            let plane_matrix = rotate_with_axis(f * 6.0f32, 1.0f32, 0.0f32, 0.0f32)
                * rotate_with_axis(f * 7.3f32, 0.0f32, 1.0f32, 0.0f32);

            let mut plane = plane_matrix[0];
            plane[3] = 0.0f32;
            plane.normalize();

            let clip_sphere = vec4!(
                (f * 0.7f32).sin() * 3.0f32,
                (f * 1.9f32).cos() * 3.0f32,
                (f * 0.1f32).sin() * 3.0f32,
                (f * 1.7f32).cos() + 2.5f32,
            );

            gl::UniformMatrix4fv(
                self.uniforms.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const f32,
            );
            gl::UniformMatrix4fv(
                self.uniforms.mv_matrix,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const f32,
            );
            gl::Uniform4fv(self.uniforms.clip_plane, 1, addr_of!(plane) as *const f32);
            gl::Uniform4fv(
                self.uniforms.clip_sphere,
                1,
                addr_of!(clip_sphere) as *const f32,
            );

            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CLIP_DISTANCE0);
            gl::Enable(gl::CLIP_DISTANCE1);

            self.object.render();
        }
    }

    fn shutdown(&mut self) {
        self.object.free();
        unsafe {
            gl::DeleteProgram(self.render_program);
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press = press {
            match key {
                glfw::Key::R => self.load_shaders(),
                glfw::Key::P => self.paused = !self.paused,
                _ => {}
            }
        }
    }
}

fn main() {
    ClipDistanceApp::default().run()
}
