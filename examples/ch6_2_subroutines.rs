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

use sb7::application::*;

#[derive(Default)]
struct Uniforms {
    subroutine1: i32,
}

#[derive(Default)]
struct App {
    render_program: u32,
    vao: u32,
    subroutines: [u32; 2],
    uniforms: Uniforms,
}

impl App {
    fn load_shaders(&mut self) {
        if self.render_program != 0 {
            unsafe { gl::DeleteProgram(self.render_program) };
        }

        self.render_program = sb7::program::link_from_shaders(
            &[
                sb7::shader::load(
                    "media/shaders/subroutines/subroutines.vs.glsl",
                    gl::VERTEX_SHADER,
                    true,
                ),
                sb7::shader::load(
                    "media/shaders/subroutines/subroutines.fs.glsl",
                    gl::FRAGMENT_SHADER,
                    true,
                ),
            ],
            true,
        );

        unsafe {
            let name = std::ffi::CString::new("myFunction1").unwrap();
            self.subroutines[0] =
                gl::GetSubroutineIndex(self.render_program, gl::FRAGMENT_SHADER, name.as_ptr());
            let name = std::ffi::CString::new("myFunction2").unwrap();
            self.subroutines[1] =
                gl::GetSubroutineIndex(self.render_program, gl::FRAGMENT_SHADER, name.as_ptr());

            let name = std::ffi::CString::new("mySubroutineUniform").unwrap();
            self.uniforms.subroutine1 = gl::GetSubroutineUniformLocation(
                self.render_program,
                gl::FRAGMENT_SHADER,
                name.as_ptr(),
            );
        }
    }
}

impl Application for App {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Shader Subroutines".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        self.load_shaders();

        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);
        }
    }

    fn render(&mut self, current_time: f64) {
        let i = current_time as usize;
        unsafe {
            gl::UseProgram(self.render_program);

            gl::UniformSubroutinesuiv(gl::FRAGMENT_SHADER, 1, &self.subroutines[i & 1]);

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn on_key(&mut self, key: imgui_glfw_rs::glfw::Key, press: imgui_glfw_rs::glfw::Action) {
        if let imgui_glfw_rs::glfw::Action::Press = press {
            match key {
                imgui_glfw_rs::glfw::Key::R => self.load_shaders(),
                _ => {}
            }
        }
    }
}

fn main() {
    App::default().run();
}
