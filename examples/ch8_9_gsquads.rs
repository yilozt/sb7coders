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

pub unsafe fn print_shader_log(shader: GLuint) {
    let mut buf: Vec<u8> = vec![];
    let mut len = 0;

    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    buf.resize(len as usize, 0);
    gl::GetShaderInfoLog(shader, len, null_mut(), buf.as_mut_ptr() as *mut GLchar);

    println!("{}", std::str::from_utf8(&buf).unwrap())
}

#[derive(Default)]
struct GsQuadsApp {
    program_fans: GLuint,
    program_linesadjacency: GLuint,
    vao: GLuint,
    mvp_loc_fans: GLint,
    mvp_loc_linesadj: GLint,
    vid_offset_loc_fans: GLint,
    vid_offset_loc_linesadj: GLint,
    mode: i32,
    vid_offset: i32,
    paused: bool,
}

impl Application for GsQuadsApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Quad Rendering".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            self.load_shaders();
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_fans);
            gl::DeleteProgram(self.program_linesadjacency);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }

    fn render(&mut self, current_time: f64) {
        unsafe {
            let black = [0.0, 0.25, 0.0, 1.0f32].as_ptr();

            static mut LAST_TIME: f64 = 0.0;
            static mut TOTAL_TIME: f64 = 0.0;

            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let t = TOTAL_TIME as f32;

            gl::ClearBufferfv(gl::COLOR, 0, black);

            let mv_matrix = translate(0.0, 0.0, -2.0)
                * rotate_with_axis(t * 5.0, 0.0, 0.0, 1.0)
                * rotate_with_axis(t * 30.0, 1.0, 0.0, 0.0);

            let proj_matrix = perspective(
                50.0,
                {
                    let i = self.info();
                    i.width as f32 / i.height as f32
                },
                0.1,
                1000.0,
            );

            let mvp = proj_matrix * mv_matrix;

            match self.mode {
                0 => {
                    gl::UseProgram(self.program_fans);
                    gl::UniformMatrix4fv(
                        self.mvp_loc_fans,
                        1,
                        gl::FALSE,
                        addr_of!(mvp) as *const GLfloat,
                    );
                    gl::Uniform1i(self.vid_offset_loc_fans, self.vid_offset);
                    gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
                }
                1 => {
                    gl::UseProgram(self.program_linesadjacency);
                    gl::UniformMatrix4fv(
                        self.mvp_loc_linesadj,
                        1,
                        gl::FALSE,
                        addr_of!(mvp) as *const GLfloat,
                    );
                    gl::Uniform1i(self.vid_offset_loc_linesadj, self.vid_offset);
                    gl::DrawArrays(gl::LINES_ADJACENCY, 0, 4);
                }
                _ => unreachable!(),
            }
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press = press {
            match key {
                glfw::Key::Kp1 => self.mode = 0,
                glfw::Key::Kp2 => self.mode = 1,
                glfw::Key::KpAdd => self.vid_offset += 1,
                glfw::Key::KpSubtract => self.vid_offset -= 1,
                glfw::Key::P => self.paused = !self.paused,
                glfw::Key::R => self.load_shaders(),
                glfw::Key::M => self.mode = (self.mode + 1) % 2,
                _ => {}
            }
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        match self.mode {
            0 => ui.text("Drawing quads using GL_TRIANGLE_FAN"),
            1 => ui.text("Drawing quads using geometry shaders and GL_LINES_ADJACENCY"),
            _ => {}
        }

        ui.text("1, 2: Choose mode (M toggles)");
        ui.text("P: Pause");
        ui.text("Numpad +, -: Rotate quad vertices");
    }
}

impl GsQuadsApp {
    fn load_shaders(&mut self) {
        unsafe {
            unsafe fn get_uniform_loc(prog: GLuint, name: &str) -> GLint {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(prog, name.as_ptr())
            }

            if self.program_fans != 0 {
                gl::DeleteProgram(self.program_fans);
            }

            self.program_fans = gl::CreateProgram();

            let vs = shader::load(
                "media/shaders/gsquads/quadsasfans.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/gsquads/quadsasfans.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );

            gl::AttachShader(self.program_fans, vs);
            gl::AttachShader(self.program_fans, fs);

            gl::LinkProgram(self.program_fans);

            self.mvp_loc_fans = get_uniform_loc(self.program_fans, "mvp");
            self.vid_offset_loc_fans = get_uniform_loc(self.program_fans, "vid_offset");

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            if self.program_linesadjacency != 0 {
                gl::DeleteProgram(self.program_linesadjacency);
            }

            self.program_linesadjacency = gl::CreateProgram();

            let vs = shader::load(
                "media/shaders/gsquads/quadsaslinesadj.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let gs = shader::load(
                "media/shaders/gsquads/quadsaslinesadj.gs.glsl",
                gl::GEOMETRY_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/gsquads/quadsaslinesadj.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );

            gl::AttachShader(self.program_linesadjacency, vs);
            gl::AttachShader(self.program_linesadjacency, gs);
            gl::AttachShader(self.program_linesadjacency, fs);

            gl::LinkProgram(self.program_linesadjacency);

            self.mvp_loc_linesadj = get_uniform_loc(self.program_fans, "mvp");
            self.vid_offset_loc_linesadj = get_uniform_loc(self.program_fans, "vid_offset");

            gl::DeleteShader(vs);
            gl::DeleteShader(gs);
            gl::DeleteShader(fs);
        }
    }
}

fn main() {
    GsQuadsApp::default().run()
}
