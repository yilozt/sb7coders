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
struct TessModesApp {
    program_index: usize,
    vao: GLuint,
    program: [GLuint; 4],
}

impl Application for TessModesApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Tessellation Modes".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_src = &cstring(
            r#"
            #version 420 core

            void main(void)
            {
                const vec4 vertices[] = vec4[](vec4( 0.4, -0.4, 0.5, 1.0),
                                               vec4(-0.4, -0.4, 0.5, 1.0),
                                               vec4( 0.4,  0.4, 0.5, 1.0),
                                               vec4(-0.4,  0.4, 0.5, 1.0));
                gl_Position = vertices[gl_VertexID];
            }"#,
        );

        let tcs_src_triangles = &cstring(
            r#"
            #version 420 core

            layout (vertices = 3) out;

            void main(void)
            {
                if (gl_InvocationID == 0)
                {
                    gl_TessLevelInner[0] = 5.0;
                    gl_TessLevelOuter[0] = 8.0;
                    gl_TessLevelOuter[1] = 8.0;
                    gl_TessLevelOuter[2] = 8.0;
                }
                gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
            }"#,
        );

        let tes_src_triangles = &cstring(
            r#"
            #version 420 core
            
            layout (triangles) in;
            
            void main(void)
            {
                gl_Position = (gl_TessCoord.x * gl_in[0].gl_Position) +
                              (gl_TessCoord.y * gl_in[1].gl_Position) +
                              (gl_TessCoord.z * gl_in[2].gl_Position);
            }"#,
        );

        let tes_src_triangles_as_points = &cstring(
            r#"
            #version 420 core
            
            layout (triangles, point_mode) in;
            
            void main(void)
            {
                gl_Position = (gl_TessCoord.x * gl_in[0].gl_Position) +
                              (gl_TessCoord.y * gl_in[1].gl_Position) +
                              (gl_TessCoord.z * gl_in[2].gl_Position);
            }"#,
        );

        let tcs_src_quads = &cstring(
            r#"
            #version 420 core

            layout (vertices = 4) out;

            void main(void)
            {
                if (gl_InvocationID == 0)
                {
                    gl_TessLevelInner[0] = 9.0;
                    gl_TessLevelInner[1] = 7.0;
                    gl_TessLevelOuter[0] = 3.0;
                    gl_TessLevelOuter[1] = 5.0;
                    gl_TessLevelOuter[2] = 3.0;
                    gl_TessLevelOuter[3] = 5.0;
                }
                gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
            }"#,
        );

        let tes_src_quads = &cstring(
            r#"
            #version 420 core

            layout (quads) in;

            void main(void)
            {
                vec4 p1 = mix(gl_in[0].gl_Position, gl_in[1].gl_Position, gl_TessCoord.x);
                vec4 p2 = mix(gl_in[2].gl_Position, gl_in[3].gl_Position, gl_TessCoord.x);
                gl_Position = mix(p1, p2, gl_TessCoord.y);
            }"#,
        );

        let tcs_src_isolines = &cstring(
            r#"
            #version 420 core

            layout (vertices = 4) out;

            void main(void)
            {
                if (gl_InvocationID == 0)
                {
                    gl_TessLevelOuter[0] = 5.0;
                    gl_TessLevelOuter[1] = 5.0;
                }
                gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
            }"#,
        );

        /* let tes_src_isolines = &cstring(
            r#"
            #version 420 core

            layout (isolines, equal_spacing, cw) in;

            void main(void)
            {
                vec4 p1 = mix(gl_in[0].gl_Position, gl_in[1].gl_Position, gl_TessCoord.x);
                vec4 p2 = mix(gl_in[2].gl_Position, gl_in[3].gl_Position, gl_TessCoord.x);
                gl_Position = mix(p1, p2, gl_TessCoord.y);
            }"#,
        ); */

        let tes_src_isolines = &cstring(
            r#"
            #version 420 core

            layout (isolines) in;

            void main(void)
            {
                float r = (gl_TessCoord.y + gl_TessCoord.x / gl_TessLevelOuter[0]);
                float t = gl_TessCoord.x * 2.0 * 3.14159;
                gl_Position = vec4(sin(t) * r, cos(t) * r, 0.5, 1.0);
            }"#,
        );

        let fs_src = &cstring(
            r#"
            #version 420 core

            out vec4 color;

            void main(void)
            {
                color = vec4(1.0);
            }"#,
        );

        unsafe {
            #[rustfmt::skip]
            let sources = [
                (vs_src, tcs_src_quads, tes_src_quads, fs_src),
                (vs_src, tcs_src_triangles, tes_src_triangles, fs_src),
                (vs_src, tcs_src_triangles, tes_src_triangles_as_points, fs_src),
                (vs_src, tcs_src_isolines, tes_src_isolines, fs_src),
            ];

            for (i, (vs_source, tcs_source, tes_source, fs_source)) in
                sources.into_iter().enumerate()
            {
                let check_err = |i, shader| {
                    let mut buf: [u8; 1024] = [0; 1024];
                    gl::GetShaderInfoLog(shader, 1024, null_mut(), buf.as_mut_ptr() as *mut i8);
                    if buf[0] != 0 {
                        let mut types = 0;
                        gl::GetShaderiv(shader, gl::SHADER_TYPE, &mut types);
                        let types = match types as GLenum {
                            gl::VERTEX_SHADER => "vs",
                            gl::FRAGMENT_SHADER => "fs",
                            gl::TESS_CONTROL_SHADER => "tcs",
                            gl::TESS_EVALUATION_SHADER => "tes",
                            _ => unreachable!("unknown type of shader!"),
                        };
                        println!("{}[{}] : {}", types, i, std::str::from_utf8(&buf).unwrap());
                    }
                };
                self.program[i] = gl::CreateProgram();

                let vs = gl::CreateShader(gl::VERTEX_SHADER);
                gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
                gl::CompileShader(vs);
                check_err(i, vs);

                let tcs = gl::CreateShader(gl::TESS_CONTROL_SHADER);
                gl::ShaderSource(tcs, 1, &tcs_source.as_ptr(), null());
                gl::CompileShader(tcs);
                check_err(i, tcs);

                let tes = gl::CreateShader(gl::TESS_EVALUATION_SHADER);
                gl::ShaderSource(tes, 1, &tes_source.as_ptr(), null());
                gl::CompileShader(tes);
                check_err(i, tes);

                let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
                gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
                gl::CompileShader(fs);
                check_err(i, fs);

                gl::AttachShader(self.program[i], vs);
                gl::AttachShader(self.program[i], tcs);
                gl::AttachShader(self.program[i], tes);
                gl::AttachShader(self.program[i], fs);
                gl::LinkProgram(self.program[i]);

                gl::DeleteShader(vs);
                gl::DeleteShader(tcs);
                gl::DeleteShader(tes);
                gl::DeleteShader(fs);
            }

            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            gl::PatchParameteri(gl::PATCH_VERTICES, 4);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        }
    }

    fn render(&self, _current_time: f64) {
        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, color::Black.as_ptr());

            gl::UseProgram(self.program[self.program_index]);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::DrawArrays(gl::PATCHES, 0, 4);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);

            for prog in self.program {
                gl::DeleteProgram(prog);
            }
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press = press {
            match key {
                glfw::Key::M => self.program_index = (self.program_index + 1) % 4,
                _ => {}
            }
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        static NAMES: [&str; 4] = ["QUADS", "TRIANGLES", "QUADS AS POINTS", "ISOLINES"];
        if let Some(win) = imgui::Window::new("ui")
            .position([10.0, 10.0], imgui::Condition::Once)
            .begin(ui)
        {
            ui.text(format!("Mode: {} (M toggles)", NAMES[self.program_index]));

            win.end()
        }
    }
}

fn main() {
    TessModesApp::default().run()
}
