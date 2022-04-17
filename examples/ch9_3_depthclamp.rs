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
struct DepthclampApp {
    program: GLuint,
    mv_location: GLint,
    proj_location: GLint,
    explode_factor_location: GLint,
    object: Object,
}

impl Application for DepthclampApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Depth Clamping".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_source = cstring(
            r#"
            #version 410 core

            layout (location = 0) in vec4 position;
            layout (location = 1) in vec3 normal;

            out VS_OUT
            {
                vec3 normal;
                vec4 color;
            } vs_out;

            uniform mat4 mv_matrix;
            uniform mat4 proj_matrix;
            uniform float explode_factor;

            void main(void)
            {
                gl_Position = proj_matrix * mv_matrix * position * vec4(vec3(explode_factor), 1.0);
                vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
                vs_out.normal = normalize(mat3(mv_matrix) * normal);
            }"#,
        );

        let fs_source = cstring(
            r#"
            #version 410 core

            out vec4 color;

            in VS_OUT
            {
                vec3 normal;
                vec4 color;
            } fs_in;

            void main(void)
            {
                color = vec4(1.0) * abs(normalize(fs_in.normal).z);
            }"#,
        );

        unsafe {
            self.program = gl::CreateProgram();
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
            gl::CompileShader(fs);

            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, fs);

            gl::LinkProgram(self.program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let get_loc = |name| {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(self.program, name.as_ptr())
            };

            self.mv_location = get_loc("mv_matrix");
            self.proj_location = get_loc("proj_matrix");
            self.explode_factor_location = get_loc("explode_factor");

            self.object.load("media/objects/dragon.sbm");

            gl::Enable(gl::CULL_FACE);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
    }

    fn render(&self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();
        let one = 1.0f32;
        let f = current_time as f32;

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::UseProgram(self.program);

            let proj_matrix = perspective(
                50.0,
                {
                    let info = self.info();
                    info.width as f32 / info.height as f32
                },
                0.1,
                1000.0,
            );
            gl::UniformMatrix4fv(
                self.proj_location,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );

            gl::Enable(gl::DEPTH_CLAMP);

            let mv_matrix = translate(0.0, 0.0, -10.0)
                * rotate_with_axis(f * 45.0, 0.0, 1.0, 0.0)
                * rotate_with_axis(f * 81.0, 1.0, 0.0, 0.0);
            gl::UniformMatrix4fv(
                self.mv_location,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const GLfloat,
            );

            gl::Uniform1f(
                self.explode_factor_location,
                (f * 3.0).sin() * (f * 4.0).cos() * 0.7 + 1.1,
            );

            self.object.render();
        }
    }

    fn shutdown(&mut self) {
        self.object.free();
        unsafe { gl::DeleteProgram(self.program) };
    }
}

fn main() {
    DepthclampApp::default().run()
}
