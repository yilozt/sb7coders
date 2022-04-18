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

use std::io::{Read, Write};
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

        unsafe {
            self.render_program = gl::CreateProgram();
            gl::ProgramParameteri(
                self.render_program,
                gl::PROGRAM_BINARY_RETRIEVABLE_HINT,
                gl::TRUE as _,
            );

            if let Ok(mut file) = std::fs::File::open("cache.bin") {
                // cache exits
                println!("loading program from cache.bin");

                let mut format = [0u8; 4];
                let mut data = Vec::new();
                file.read(&mut format).unwrap();
                file.read_to_end(&mut data).unwrap();

                let format = u32::from_ne_bytes(format);
                gl::ProgramBinary(
                    self.render_program,
                    format,
                    data[..].as_ptr() as _,
                    data.len() as _,
                );

                // Don't need link program when load from binary
                // gl::LinkProgram(self.render_program);
            } else {
                // compile form source
                let vs = sb7::shader::load(
                    "media/shaders/subroutines/subroutines.vs.glsl",
                    gl::VERTEX_SHADER,
                    true,
                );
                let fs = sb7::shader::load(
                    "media/shaders/subroutines/subroutines.fs.glsl",
                    gl::FRAGMENT_SHADER,
                    true,
                );

                gl::AttachShader(self.render_program, vs);
                gl::AttachShader(self.render_program, fs);
                gl::LinkProgram(self.render_program);
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);

                // query binary size and format
                let mut size = 0;
                gl::GetProgramiv(self.render_program, gl::PROGRAM_BINARY_LENGTH, &mut size);

                // alloc buffer to storage binary
                let mut buf: Vec<u8> = Vec::with_capacity(size as _);
                buf.resize(size as _, 0);

                // get binary data from program
                let mut format = 0;
                gl::GetProgramBinary(
                    self.render_program,
                    size,
                    std::ptr::null_mut(),
                    &mut format,
                    buf[..].as_mut_ptr() as _,
                );

                // save to file
                println!("saving program to cache.bin");
                let mut file = std::fs::File::create("cache.bin").unwrap();
                file.write(&format.to_ne_bytes()).unwrap();
                file.write(&buf).unwrap();
            }
        }

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
