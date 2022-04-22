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

const NUM_ELEMENTS: usize = 1024;

struct PrefixsumApp {
    prefix_sum_prog: GLuint,
    data_buffer: [GLuint; 2],
    input_data: [f32; NUM_ELEMENTS],
    output_data: [f32; NUM_ELEMENTS],
}

impl Default for PrefixsumApp {
    fn default() -> Self {
        Self {
            prefix_sum_prog: 0,
            data_buffer: [0; 2],
            input_data: [0.0; NUM_ELEMENTS],
            output_data: [0.0; NUM_ELEMENTS],
        }
    }
}

impl Application for PrefixsumApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - 1D Prefix Sum".into(),
            ..AppConfig::default()
        }
    }

    fn startup(&mut self) {
        unsafe {
            gl::GenBuffers(2, self.data_buffer.as_mut_ptr());

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.data_buffer[0]);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                size_of_val(&self.input_data) as GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.data_buffer[1]);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                size_of_val(&self.output_data) as GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );
        }

        for e in self.input_data.iter_mut() {
            *e = rand::random();
        }

        Self::prefix_sum(&mut self.input_data, &mut self.output_data);

        self.load_shaders();
    }

    fn render(&mut self, _current_time: f64) {
        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, color::Black.as_ptr());

            gl::BindBufferRange(
                gl::SHADER_STORAGE_BUFFER,
                0,
                self.data_buffer[0],
                0,
                size_of_val(&self.input_data) as GLsizeiptr,
            );
            gl::BufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                0,
                size_of_val(&self.input_data) as GLsizeiptr,
                self.input_data.as_ptr() as *const std::ffi::c_void,
            );

            gl::BindBufferRange(
                gl::SHADER_STORAGE_BUFFER,
                1,
                self.data_buffer[1],
                0,
                size_of_val(&self.output_data) as GLsizeiptr,
            );

            gl::UseProgram(self.prefix_sum_prog);
            gl::DispatchCompute(1, 1, 1);

            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
            gl::Finish();
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        let ptr = unsafe {
            let ptr = gl::MapNamedBufferRange(
                self.data_buffer[1],
                0,
                size_of_val(&self.output_data) as GLsizeiptr,
                gl::MAP_READ_BIT,
            ) as *const f32;
            std::slice::from_raw_parts(ptr, self.output_data.len())
        };

        ui.text(format!("SUM: {:?}", &ptr[0..=15]));

        unsafe { gl::UnmapNamedBuffer(self.data_buffer[1]) };
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press | glfw::Action::Repeat = press {
            match key {
                glfw::Key::R => self.load_shaders(),
                _ => {}
            }
        }
    }
}

impl PrefixsumApp {
    fn prefix_sum(input_data: &mut [f32], output_data: &mut [f32]) {
        let mut f = 0.0;

        for i in 0..input_data.len() {
            f += input_data[i];
            output_data[i] = f;
        }
    }

    fn load_shaders(&mut self) {
        let cs = shader::load(
            "media/shaders/prefixsum/prefixsum.cs.glsl",
            gl::COMPUTE_SHADER,
            true,
        );

        self.prefix_sum_prog = program::link_from_shaders(&[cs], true);

        unsafe {
            gl::ShaderStorageBlockBinding(self.prefix_sum_prog, 0, 0);
            gl::ShaderStorageBlockBinding(self.prefix_sum_prog, 1, 1);
        }
    }
}

fn main() {
    PrefixsumApp::default().run();
}
