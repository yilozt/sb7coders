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
struct App {
    program: GLuint,
    mv_location: GLint,
    proj_location: GLint,
    explode_factor_location: GLint,

    object: Object,
}

impl Application for App {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Exploder".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_src = cstring(
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

            void main(void)
            {
                gl_Position = proj_matrix * mv_matrix * position;
                vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
                vs_out.normal = normalize(mat3(mv_matrix) * normal);
            }"#,
        );

        let gs_src = cstring(
            r#"
            #version 410 core

            layout (triangles) in;
            layout (triangle_strip, max_vertices = 3) out;

            in VS_OUT
            {
                vec3 normal;
                vec4 color;
            } gs_in[];

            out GS_OUT
            {
                vec3 normal;
                vec4 color;
            } gs_out;

            uniform float explode_factor = 0.2;

            void main(void)
            {
                vec3 ab = gl_in[1].gl_Position.xyz - gl_in[0].gl_Position.xyz;
                vec3 ac = gl_in[2].gl_Position.xyz - gl_in[0].gl_Position.xyz;
                vec3 face_normal = -normalize(cross(ab, ac));
                for (int i = 0; i < gl_in.length(); i++)
                {
                    gl_Position = gl_in[i].gl_Position + vec4(face_normal * explode_factor, 0.0);
                    gs_out.normal = gs_in[i].normal;
                    gs_out.color = gs_in[i].color;
                    EmitVertex();
                }
                EndPrimitive();
            }"#,
        );

        let fs_src = cstring(
            r#"
            #version 410 core

            out vec4 color;

            in GS_OUT
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
            gl::ShaderSource(vs, 1, &vs_src.as_ptr(), null());
            gl::CompileShader(vs);

            let gs = gl::CreateShader(gl::GEOMETRY_SHADER);
            gl::ShaderSource(gs, 1, &gs_src.as_ptr(), null());
            gl::CompileShader(gs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_src.as_ptr(), null());
            gl::CompileShader(fs);

            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, gs);
            gl::AttachShader(self.program, fs);

            gl::LinkProgram(self.program);

            gl::DeleteShader(vs);
            gl::DeleteShader(gs);
            gl::DeleteShader(fs);

            let get_loc = |name| {
                let name = cstring(name);
                gl::GetUniformLocation(self.program, name.as_ptr())
            };

            self.mv_location = get_loc("mv_matrix");
            self.proj_location = get_loc("proj_matrix");
            self.explode_factor_location = get_loc("explode_factor");

            self.object.load("media/objects/torus.sbm");

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 0.0f32];
        let one = 1.0f32;
        let f = current_time as f32;

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, black.as_ptr());
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::UseProgram(self.program);

            let proj_matrix = perspective(
                50.0f32,
                {
                    let info = self.info();
                    info.width as f32 / info.height as f32
                },
                0.1f32,
                1000.0f32,
            );

            gl::UniformMatrix4fv(
                self.proj_location,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );

            let mv_matrix = translate(0.0, 0.0, -3.0)
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
                (f * 8.0).sin() * (f * 6.0).cos() * 0.7 + 0.1,
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
    App::default().run()
}
