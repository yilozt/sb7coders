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
struct NoperspectiveApp {
    program: GLuint,
    vao: GLuint,
    tex_checker: GLuint,
    paused: bool,
    use_perspective: bool,
    uniforms: Uniforms,
}

#[derive(Default)]
struct Uniforms {
    mvp: GLint,
    use_perspective: GLint,
}

impl Application for NoperspectiveApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Perspective".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_source = cstring(
            r#"
            #version 410 core

            out VS_OUT
            {
                vec2 tc;
                noperspective vec2 tc_np;
            } vs_out;

            uniform mat4 mvp;

            void main(void)
            {
                const vec4 vertices[] = vec4[](vec4(-0.5, -0.5, 0.0, 1.0),
                                               vec4( 0.5, -0.5, 0.0, 1.0),
                                               vec4(-0.5,  0.5, 0.0, 1.0),
                                               vec4( 0.5,  0.5, 0.0, 1.0));

                vec2 tc = (vertices[gl_VertexID].xy + vec2(0.5));
                vs_out.tc = tc;
                vs_out.tc_np = tc;
                gl_Position = mvp * vertices[gl_VertexID];
            }"#,
        );

        let fs_source = cstring(
            r#"
            #version 410 core

            out vec4 color;

            uniform sampler2D tex_checker;

            uniform bool use_perspective = true;

            in VS_OUT
            {
                vec2 tc;
                noperspective vec2 tc_np;
            } fs_in;

            void main(void)
            {
                vec2 tc = mix(fs_in.tc_np, fs_in.tc, bvec2(use_perspective));
                color = texture(tex_checker, tc).rrrr;
            }"#,
        );

        unsafe {
            // let mut buffer = [0u8; 1024];

            self.program = gl::CreateProgram();
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
            gl::CompileShader(vs);

            // gl::GetShaderInfoLog(vs, 1024, null_mut(), buffer.as_ptr() as *mut i8);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
            gl::CompileShader(fs);

            // gl::GetShaderInfoLog(fs, 1024, null_mut(), buffer.as_ptr() as *mut i8);

            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, fs);

            gl::LinkProgram(self.program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let get_uniform_location = |prog, name| {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(prog, name.as_ptr())
            };

            self.uniforms.mvp = get_uniform_location(self.program, "mvp");
            self.uniforms.use_perspective = get_uniform_location(self.program, "use_perspective");

            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            #[rustfmt::skip]
            let checker_data: &[GLubyte] = &[
                0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
                0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
                0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
                0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
            ];

            gl::GenTextures(1, &mut self.tex_checker);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_checker);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::R8, 8, 8);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                8,
                8,
                gl::RED,
                gl::UNSIGNED_BYTE,
                checker_data.as_ptr() as *const std::ffi::c_void,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
        }
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();
        let one = 1.0f32;

        unsafe {
            static mut LAST_TIME: f64 = 0.0;
            static mut TOTAL_TIME: f64 = 0.0;

            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let t = TOTAL_TIME as f32 * 14.3;

            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            let mv_matrix = translate(0.0, 0.0, -1.5) * rotate_with_axis(t, 0.0, 1.0, 0.0);
            let proj_matrix = perspective(
                60.0,
                {
                    let info = self.info();
                    info.width as f32 / info.height as f32
                },
                0.1,
                1000.0,
            );
            let mvp_matrix = proj_matrix * mv_matrix;

            gl::UseProgram(self.program);

            gl::UniformMatrix4fv(
                self.uniforms.mvp,
                1,
                gl::FALSE,
                addr_of!(mvp_matrix) as *const GLfloat,
            );
            gl::Uniform1i(
                self.uniforms.use_perspective,
                match self.use_perspective {
                    true => 1,
                    false => 0,
                },
            );

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press = press {
            match key {
                glfw::Key::M => self.use_perspective = !self.use_perspective,
                glfw::Key::P => self.paused = !self.paused,
                _ => {}
            }
        }
    }
}

fn main() {
    NoperspectiveApp::default().run()
}
