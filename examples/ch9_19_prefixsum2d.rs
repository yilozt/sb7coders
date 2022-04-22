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

const NUM_ELEMENTS: usize = 2048;

#[derive(Default)]
struct PrefixsumApp {
    prefix_sum_prog: GLuint,
    show_image_prog: GLuint,
    dummy_vao: GLuint,
    images: [GLuint; 3],
}

impl Application for PrefixsumApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - 2D Prefix Sum".into(),
            ..AppConfig::default()
        }
    }

    fn startup(&mut self) {
        unsafe {
            gl::GenTextures(3, self.images.as_mut_ptr());

            self.images[0] = ktx::file::load("media/textures/salad-gray.ktx").unwrap().0;

            for i in 1..3 {
                gl::BindTexture(gl::TEXTURE_2D, self.images[i]);
                gl::TexStorage2D(
                    gl::TEXTURE_2D,
                    1,
                    gl::R32F,
                    NUM_ELEMENTS as GLsizei,
                    NUM_ELEMENTS as GLsizei,
                );
            }

            gl::GenVertexArrays(1, &mut self.dummy_vao);
            gl::BindVertexArray(self.dummy_vao);
        }

        self.load_shaders();
    }

    fn render(&mut self, _current_time: f64) {
        unsafe {
            gl::UseProgram(self.prefix_sum_prog);

            gl::BindImageTexture(0, self.images[0], 0, gl::FALSE, 0, gl::READ_ONLY, gl::R32F);
            gl::BindImageTexture(1, self.images[1], 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::R32F);

            gl::DispatchCompute(NUM_ELEMENTS as GLuint, 1, 1);

            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            gl::BindImageTexture(0, self.images[1], 0, gl::FALSE, 0, gl::READ_ONLY, gl::R32F);
            gl::BindImageTexture(1, self.images[2], 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::R32F);

            gl::DispatchCompute(NUM_ELEMENTS as GLuint, 1, 1);

            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            gl::BindTexture(gl::TEXTURE_2D, self.images[2]);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.images[2]);

            gl::UseProgram(self.show_image_prog);

            gl::BindVertexArray(self.dummy_vao);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
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
    fn _prefix_sum(input_data: &mut [f32], output_data: &mut [f32]) {
        let mut f = 0.0;

        for i in 0..input_data.len() {
            f += input_data[i];
            output_data[i] = f;
        }
    }

    fn load_shaders(&mut self) {
        let cs = shader::load(
            "media/shaders/prefixsum2d/prefixsum2d.cs.glsl",
            gl::COMPUTE_SHADER,
            true,
        );

        self.prefix_sum_prog = program::link_from_shaders(&[cs], true);

        let vs = shader::load(
            "media/shaders/prefixsum2d/showimage.vs.glsl",
            gl::VERTEX_SHADER,
            true,
        );
        let fs = shader::load(
            "media/shaders/prefixsum2d/showimage.fs.glsl",
            gl::FRAGMENT_SHADER,
            true,
        );

        self.show_image_prog = program::link_from_shaders(&[vs, fs], true);
    }
}

fn main() {
    PrefixsumApp::default().run();
}
