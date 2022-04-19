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
struct ShapedPointsApp {
    render_vao: GLuint,
    render_prog: GLuint,
}

impl Application for ShapedPointsApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Shaped Points".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();
        let fs_source = cstring(
            r#"
            #version 410 core

            layout (location = 0) out vec4 color;

            flat in int shape;

            void main(void)
            {
                color = vec4(1.0);
                vec2 p = gl_PointCoord * 2.0 - vec2(1.0);
                if (shape == 0)
                {
                    if (dot(p, p) > 1.0)
                        discard;
                }
                else if (shape == 1)
                {
                    if (dot(p, p) > sin(atan(p.y, p.x) * 5.0))
                        discard;
                }
                else if (shape == 2)
                {
                    if (abs(0.8 - dot(p, p)) > 0.2)
                        discard;
                }
                else if (shape == 3)
                {
                    if (abs(p.x) < abs(p.y))
                        discard;
                }
            }"#,
        );

        let vs_source = cstring(
            r#"
            #version 410 core

            flat out int shape;

            void main(void)
            {
                const vec4[4] position = vec4[4](vec4(-0.4, -0.4, 0.5, 1.0),
                                                 vec4( 0.4, -0.4, 0.5, 1.0),
                                                 vec4(-0.4,  0.4, 0.5, 1.0),
                                                 vec4( 0.4,  0.4, 0.5, 1.0));
                gl_Position = position[gl_VertexID];
                shape = gl_VertexID;
            }"#,
        );

        unsafe {
            self.render_prog = gl::CreateProgram();

            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
            gl::CompileShader(fs);

            gl::AttachShader(self.render_prog, vs);
            gl::AttachShader(self.render_prog, fs);

            gl::LinkProgram(self.render_prog);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            gl::CreateVertexArrays(1, &mut self.render_vao);
            gl::BindVertexArray(self.render_vao);
        }
    }

    fn render(&mut self, _current_time: f64) {
        let black = [0.0, 0.0, 0.0, 0.0].as_ptr();

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, black);

            gl::UseProgram(self.render_prog);

            gl::PointSize(200.0);
            gl::BindVertexArray(self.render_vao);
            gl::DrawArrays(gl::POINTS, 0, 4);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.render_prog);
            gl::DeleteVertexArrays(1, &self.render_vao);
        }
    }
}

fn main() {
    ShapedPointsApp::default().run()
}
