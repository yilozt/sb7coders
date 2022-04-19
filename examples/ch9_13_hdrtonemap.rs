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

unsafe fn _print_shader_log(msg: &str, shader: GLuint) {
    let mut len = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    if len != 0 {
        let buf = [0u8; 1024];
        gl::GetShaderInfoLog(shader, 1024, null_mut(), buf.as_ptr() as *mut GLchar);
        println!("{} :", msg);
        println!(
            "{}",
            std::str::from_utf8(&buf).unwrap_or("invaild utf-8 log")
        );
    }
}

#[derive(Default)]
struct HDRTonemapApp {
    tex_src: GLuint,
    // tex_lut: GLuint,

    program_naive: GLuint,
    program_exposure: GLuint,
    program_adaptive: GLuint,
    vao: GLuint,
    exposure: f32,
    mode: i32,

    uniforms: Uniforms,
}

#[derive(Default)]
struct Uniforms {
    exposure: ExposureUniforms,
}

#[derive(Default)]
struct ExposureUniforms {
    exposure: GLint,
}

impl HDRTonemapApp {
    fn new() -> Self {
        Self {
            exposure: 1.0,
            ..Default::default()
        }
    }

    fn load_shaders(&mut self) {
        unsafe {
            if self.program_naive != 0 {
                gl::DeleteProgram(self.program_naive);
            }

            self.program_naive = gl::CreateProgram();

            let vs = shader::load(
                "media/shaders/hdrtonemap/tonemap.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/hdrtonemap/tonemap_naive.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );

            gl::AttachShader(self.program_naive, vs);
            gl::AttachShader(self.program_naive, fs);

            gl::LinkProgram(self.program_naive);

            gl::DeleteShader(fs);

            if self.program_adaptive != 0 {
                gl::DeleteProgram(self.program_adaptive);
            }

            self.program_adaptive = gl::CreateProgram();

            let fs = shader::load(
                "media/shaders/hdrtonemap/tonemap_adaptive.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );

            gl::AttachShader(self.program_adaptive, vs);
            gl::AttachShader(self.program_adaptive, fs);

            gl::LinkProgram(self.program_adaptive);

            gl::DeleteShader(fs);

            if self.program_exposure != 0 {
                gl::DeleteProgram(self.program_exposure);
            }

            self.program_exposure = gl::CreateProgram();

            let fs = shader::load(
                "media/shaders/hdrtonemap/tonemap_exposure.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );

            gl::AttachShader(self.program_exposure, vs);
            gl::AttachShader(self.program_exposure, fs);

            gl::LinkProgram(self.program_exposure);

            let name = std::ffi::CString::new("exposure").unwrap();
            self.uniforms.exposure.exposure =
                gl::GetUniformLocation(self.program_exposure, name.as_ptr());

            gl::DeleteShader(fs);
            gl::DeleteShader(vs);
        }
    }
}

impl Application for HDRTonemapApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - HDR Tone Mapping".into(),
            ..AppConfig::default()
        }
    }

    fn startup(&mut self) {
        unsafe {
            // Load texture from file
            self.tex_src = ktx::file::load("media/textures/treelights_2k.ktx")
                .unwrap()
                .0;

            // Now bind it to the context using the GL_TEXTURE_2D binding point
            gl::BindTexture(gl::TEXTURE_2D, self.tex_src);

            gl::CreateVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            self.load_shaders();

            /* let exposure_lut = [
                11.0f32, 6.0f32, 3.2f32, 2.8f32, 2.2f32, 1.90f32, 1.80f32, 1.80f32, 1.70f32,
                1.70f32, 1.60f32, 1.60f32, 1.50f32, 1.50f32, 1.40f32, 1.40f32, 1.30f32, 1.20f32,
                1.10f32, 1.00f32,
            ];

            gl::GenTextures(1, &mut self.tex_lut);
            gl::BindTexture(gl::TEXTURE_1D, self.tex_lut);
            gl::TexStorage1D(gl::TEXTURE_1D, 1, gl::R32F, 20);
            gl::TexSubImage1D(
                gl::TEXTURE_1D,
                0,
                0,
                20,
                gl::RED,
                gl::FLOAT,
                exposure_lut.as_ptr() as *const std::ffi::c_void,
            );
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(
                gl::TEXTURE_1D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            ); */
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_adaptive);
            gl::DeleteProgram(self.program_exposure);
            gl::DeleteProgram(self.program_naive);
            // gl::DeleteTextures(1, &self.tex_lut);
            gl::DeleteTextures(1, &self.tex_src);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }

    fn render(&mut self, _current_time: f64) {
        let green = [0.0, 0.25, 0.0, 1.0].as_ptr();
        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, green);

            gl::ActiveTexture(gl::TEXTURE1);
            // gl::BindTexture(gl::TEXTURE_1D, self.tex_lut);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_src);

            match self.mode {
                0 => gl::UseProgram(self.program_naive),
                1 => {
                    gl::UseProgram(self.program_exposure);
                    gl::Uniform1f(self.uniforms.exposure.exposure, self.exposure);
                }
                2 => gl::UseProgram(self.program_adaptive),
                _ => {}
            }
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press | glfw::Action::Repeat = press {
            match key {
                glfw::Key::Kp1 => self.mode = 0,
                glfw::Key::Kp2 => self.mode = 1,
                glfw::Key::Kp3 => self.mode = 2,
                glfw::Key::R => self.load_shaders(),
                glfw::Key::M => self.mode = (self.mode + 1) % 3,
                glfw::Key::KpAdd => self.exposure *= 1.1,
                glfw::Key::KpSubtract => self.exposure /= 1.1,
                _ => {}
            }
        }
    }
}

fn main() {
    HDRTonemapApp::new().run();
}
