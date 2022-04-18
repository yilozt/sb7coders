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
struct GscullingApp {
    program: GLuint,
    mv_location: GLint,
    mvp_location: GLint,
    viewpoint_location: GLint,
    object: Object,
}

impl Application for GscullingApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Geometry Shader Culling".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_src = cstring(
            r#"
            #version 410 core

            // Incoming per vertex... position and normal
            layout (location = 0) in vec4 vVertex;
            layout (location = 1) in vec3 vNormal;

            out Vertex
            {
                vec3 normal;
                vec4 color;
            } vertex;

            uniform vec3 vLightPosition = vec3(-10.0, 40.0, 200.0);
            uniform mat4 mvMatrix;

            void main(void)
            {
                // Get surface normal in eye coordinates
                vec3 vEyeNormal = mat3(mvMatrix) * normalize(vNormal);

                // Get vertex position in eye coordinates
                vec4 vPosition4 = mvMatrix * vVertex;
                vec3 vPosition3 = vPosition4.xyz / vPosition4.w;

                // Get vector to light source
                vec3 vLightDir = normalize(vLightPosition - vPosition3);

                // Dot product gives us diffuse intensity
                vertex.color = vec4(0.7, 0.6, 1.0, 1.0) * abs(dot(vEyeNormal, vLightDir));

                gl_Position = vVertex;
                vertex.normal = vNormal;
            }"#,
        );

        let gs_src = cstring(
            r#"
            #version 410 core

            layout (triangles) in;
            layout (triangle_strip, max_vertices = 3) out;

            in Vertex
            {
                vec3 normal;
                vec4 color;
            } vertex[];

            out vec4 color;

            uniform vec3 vLightPosition;
            uniform mat4 mvpMatrix;
            uniform mat4 mvMatrix;

            uniform vec3 viewpoint;

            void main(void)
            {
                int n;

                vec3 ab = gl_in[1].gl_Position.xyz - gl_in[0].gl_Position.xyz;
                vec3 ac = gl_in[2].gl_Position.xyz - gl_in[0].gl_Position.xyz;
                vec3 normal = normalize(cross(ab, ac));
                vec3 transformed_normal = (mat3(mvMatrix) * normal);
                vec4 worldspace = /* mvMatrix * */ gl_in[0].gl_Position;
                vec3 vt = normalize(viewpoint - worldspace.xyz);

                if (dot(normal, vt) > 0.0) {
                    for (n = 0; n < 3; n++) {
                        gl_Position = mvpMatrix * gl_in[n].gl_Position;
                        color = vertex[n].color;
                        EmitVertex();
                    }
                    EndPrimitive();
                }
            }"#,
        );

        let fs_src = cstring(
            r#"
            #version 410 core

            in vec4 color;

            out vec4 output_color;

            void main(void)
            {
                output_color = color;
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

            self.mv_location = get_loc("mvMatrix");
            self.mvp_location = get_loc("mvpMatrix");
            self.viewpoint_location = get_loc("viewpoint");

            self.object.load("media/objects/dragon.sbm");

            gl::Disable(gl::CULL_FACE);
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

            let mv_matrix = translate(0.0f32, 0.0f32, -20.0f32)
                * rotate_with_axis(current_time as f32 * 5.0f32, 0.0f32, 1.0f32, 0.0f32)
                * rotate_with_axis(current_time as f32 * 100.0f32, 1.0f32, 0.0f32, 0.0f32);

            let mvp = proj_matrix * mv_matrix;

            gl::UniformMatrix4fv(
                self.mv_location,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const GLfloat,
            );

            gl::UniformMatrix4fv(
                self.mvp_location,
                1,
                gl::FALSE,
                addr_of!(mvp) as *const GLfloat,
            );

            let viewpoint = [
                (f * 2.1f32).sin() * 70.0f32,
                (f * 1.4f32).cos() * 70.0f32,
                (f * 0.7f32).sin() * 70.0f32,
            ];
            gl::Uniform3fv(self.viewpoint_location, 1, viewpoint.as_ptr());

            self.object.render();
        }
    }

    fn shutdown(&mut self) {
        self.object.free();
        unsafe { gl::DeleteProgram(self.program) };
    }
}

fn main() {
    GscullingApp::default().run()
}
