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
struct HDRExposureApp {
    texture: GLuint,
    program: GLuint,
    vao: GLuint,
    exposure: f32,
}

impl HDRExposureApp {
    fn new() -> Self {
        Self {
            exposure: 1.0,
            ..Default::default()
        }
    }
}

impl Application for HDRExposureApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - HDR Exposure".into(),
            ..AppConfig::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_source = cstring(
            r#"
            #version 420 core                                                              
                                                                                        
            void main(void)                                                                
            {                                                                              
                const vec4 vertices[] = vec4[](vec4(-1.0, -1.0, 0.5, 1.0),                 
                                            vec4( 1.0, -1.0, 0.5, 1.0),                 
                                            vec4(-1.0,  1.0, 0.5, 1.0),                 
                                            vec4( 1.0,  1.0, 0.5, 1.0));                
                                                                                        
                gl_Position = vertices[gl_VertexID];
            }"#,
        );

        let fs_source = cstring(
            r#"
            #version 430 core                                                              
                                                                                           
            uniform sampler2D s;                                                           
                                                                                           
            layout (location = 0) uniform float exposure;
            
            out vec4 color;                                                                
                                                                                           
            void main(void)                                                                
            {                                                                              
                vec4 c = texture(s, 2.0 * gl_FragCoord.xy / textureSize(s, 0));                  
                c.xyz = vec3(1.0) - exp(-c.xyz * exposure);                                
                color = c;                                                                 
            }"#,
        );

        unsafe {
            // Generate a name of the texture
            gl::GenTextures(1, &mut self.texture);

            // Load texture from file
            ktx::file::load_with_tex("media/textures/treelights_2k.ktx", self.texture).unwrap();

            // Now bind it to the context using the GL_TEXTURE_2D binding point
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            self.program = gl::CreateProgram();

            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
            gl::CompileShader(fs);

            _print_shader_log("fs", fs);

            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, fs);

            gl::LinkProgram(self.program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            gl::CreateVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }

    fn render(&mut self, _current_time: f64) {
        let green = [0.0, 0.25, 0.0, 1.0].as_ptr();
        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, green);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::UseProgram(self.program);
            gl::Uniform1f(0, self.exposure);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        if let Some(win) = imgui::Window::new("Debug")
            .position([10.0, 10.0], imgui::Condition::Once)
            .begin(ui)
        {
            ui.text(format!(
                "Exposure = {:.2} (Numpad +/- to change)",
                self.exposure
            ));
            win.end()
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press | glfw::Action::Repeat = press {
            match key {
                glfw::Key::KpAdd => self.exposure += 0.1,
                glfw::Key::KpSubtract => self.exposure -= 0.1,
                _ => {}
            }
        }
    }
}

fn main() {
    HDRExposureApp::new().run();
}
