// Copyright � 2012-2015 Graham Sellers
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (theSoftware"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDEDAS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use sb7::prelude::*;

#[derive(Default)]
struct GsTessellateApp {
    program: GLuint,
    mv_location: GLint,
    mvp_location: GLint,
    stretch_location: GLint,
    vao: GLuint,
    buffer: GLuint,
}

impl Application for GsTessellateApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Geometry Shader Tessellation".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_src = cstring(
            r#"
            // Vertex Shader
            // OpenGL SuperBible
            #version 410 core

            // Incoming per vertex... position and normal
            in vec4 vVertex;

            void main(void)
            {
                gl_Position = vVertex;
            }"#,
        );

        let gs_src = cstring(
            r#"
            // Geometry Shader
            // Graham Sellers
            // OpenGL SuperBible
            #version 410 core


            layout (triangles) in;
            layout (triangle_strip, max_vertices = 12) out;

            uniform float stretch = 0.7;

            flat out vec4 color;

            uniform mat4 mvpMatrix;
            uniform mat4 mvMatrix;

            void make_face(vec3 a, vec3 b, vec3 c)
            {
                vec3 face_normal = normalize(cross(c - a, c - b));
                vec4 face_color = vec4(1.0, 0.4, 0.7, 1.0) * (mat3(mvMatrix) * face_normal).z;
                gl_Position = mvpMatrix * vec4(a, 1.0);
                color = face_color;
                EmitVertex();

                gl_Position = mvpMatrix * vec4(b, 1.0);
                color = face_color;
                EmitVertex();

                gl_Position = mvpMatrix * vec4(c, 1.0);
                color = face_color;
                EmitVertex();

                EndPrimitive();
            }

            void main(void)
            {
                int n;
                vec3 a = gl_in[0].gl_Position.xyz;
                vec3 b = gl_in[1].gl_Position.xyz;
                vec3 c = gl_in[2].gl_Position.xyz;

                vec3 d = (a + b) * stretch;
                vec3 e = (b + c) * stretch;
                vec3 f = (c + a) * stretch;

                a *= (2.0 - stretch);
                b *= (2.0 - stretch);
                c *= (2.0 - stretch);

                make_face(a, d, f);
                make_face(d, b, e);
                make_face(e, c, f);
                make_face(d, e, f);

                EndPrimitive();
            }"#,
        );

        let fs_src = cstring(
            r#"
            // Fragment Shader
            // Graham Sellers
            // OpenGL SuperBible
            #version 410 core

            flat in vec4 color;

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
            self.stretch_location = get_loc("stretch");

            #[rustfmt::skip]
            let tetrahedron_verts: &[GLfloat] = &[
                 0.000f32,  0.000f32,  1.000f32,
                 0.943f32,  0.000f32, -0.333f32,
                -0.471f32,  0.816f32, -0.333f32,
                -0.471f32, -0.816f32, -0.333f32
            ];

            #[rustfmt::skip]
            let tetrahedron_indices: &[GLushort] = &[
                0, 1, 2,
                0, 2, 3,
                0, 3, 1,
                3, 2, 1
            ];

            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            gl::GenBuffers(1, &mut self.buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (size_of_val(tetrahedron_verts) + size_of_val(tetrahedron_indices)) as GLsizeiptr,
                null(),
                gl::STATIC_DRAW,
            );
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                0,
                size_of_val(tetrahedron_indices) as GLsizeiptr,
                tetrahedron_indices.as_ptr() as *const std::ffi::c_void,
            );
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(tetrahedron_indices) as GLsizeiptr,
                size_of_val(tetrahedron_verts) as GLsizeiptr,
                tetrahedron_verts.as_ptr() as *const std::ffi::c_void,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                size_of_val(tetrahedron_indices) as *const std::ffi::c_void,
            );
            gl::EnableVertexAttribArray(0);

            gl::Enable(gl::CULL_FACE);
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

            let mv_matrix = translate(0.0, 0.0, -8.0)
                * rotate_with_axis(f * 71.0, 0.0, 1.0, 0.0)
                * rotate_with_axis(f * 10.0, 1.0, 0.0, 0.0);

            let mvp_matrix = proj_matrix * mv_matrix;

            gl::UniformMatrix4fv(
                self.mvp_location,
                1,
                gl::FALSE,
                addr_of!(mvp_matrix) as *const GLfloat,
            );

            gl::UniformMatrix4fv(
                self.mv_location,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const GLfloat,
            );

            gl::Uniform1f(self.stretch_location, (f * 4.0).sin() * 0.75 + 1.0);

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 12, gl::UNSIGNED_SHORT, null());
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.buffer);
        }
    }
}

fn main() {
    GsTessellateApp::default().run()
}
