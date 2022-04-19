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

use rand::Rng;
use sb7::prelude::*;

const NUM_STARTS: usize = 2000;

#[derive(Default)]
struct StarfieldApp {
    render_prog: GLuint,
    star_texture: GLuint,
    star_vao: GLuint,
    star_buffer: GLuint,

    uniforms: Uniforms,
}

#[derive(Default)]
struct Uniforms {
    time: GLint,
    proj_matrix: GLint,
}

impl Application for StarfieldApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Starfield".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let fs_source = cstring(
            r#"
            #version 410 core

            layout (location = 0) out vec4 color;

            uniform sampler2D tex_star;
            flat in vec4 starColor;

            void main(void)
            {
                color = starColor * texture(tex_star, gl_PointCoord);
            }"#,
        );

        let vs_source = cstring(
            r#"
            #version 410 core

            layout (location = 0) in vec4 position;
            layout (location = 1) in vec4 color;

            uniform float time;
            uniform mat4 proj_matrix;

            flat out vec4 starColor;

            void main(void)
            {
                vec4 newVertex = position;

                newVertex.z += time;
                newVertex.z = fract(newVertex.z);

                float size = (20.0 * newVertex.z * newVertex.z);

                starColor = smoothstep(1.0, 7.0, size) * color;

                newVertex.z = (999.9 * newVertex.z) - 1000.0;
                gl_Position = proj_matrix * newVertex;
                gl_PointSize = size;
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

            let name = std::ffi::CString::new("time").unwrap();
            self.uniforms.time = gl::GetUniformLocation(self.render_prog, name.as_ptr());
            let name = std::ffi::CString::new("proj_matrix").unwrap();
            self.uniforms.proj_matrix = gl::GetUniformLocation(self.render_prog, name.as_ptr());

            self.star_texture = ktx::file::load("media/textures/star.ktx").unwrap().0;

            gl::GenVertexArrays(1, &mut self.star_vao);
            gl::BindVertexArray(self.star_vao);

            struct StarT {
                position: Vec3,
                color: Vec3,
            }

            gl::GenBuffers(1, &mut self.star_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.star_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (NUM_STARTS * size_of::<StarT>()) as GLsizeiptr,
                null(),
                gl::STATIC_DRAW,
            );

            let start = std::slice::from_raw_parts_mut(
                gl::MapBufferRange(
                    gl::ARRAY_BUFFER,
                    0,
                    (NUM_STARTS * size_of::<StarT>()) as GLsizeiptr,
                    gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
                ) as *mut StarT,
                NUM_STARTS,
            );

            let mut rng = rand::thread_rng();

            for i in 0..start.len() {
                start[i].position[0] = (rng.gen::<f32>() * 2.0 - 1.0) * 100.0;
                start[i].position[1] = (rng.gen::<f32>() * 2.0 - 1.0) * 100.0;
                start[i].position[2] = rng.gen::<f32>();
                start[i].color[0] = rng.gen::<f32>() * 0.2 + 0.8;
                start[i].color[1] = rng.gen::<f32>() * 0.2 + 0.8;
                start[i].color[2] = rng.gen::<f32>() * 0.2 + 0.8;
            }

            gl::UnmapBuffer(gl::ARRAY_BUFFER);

            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<StarT>() as GLsizei,
                null(),
            );
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<StarT>() as GLsizei,
                size_of::<Vec3>() as *const std::ffi::c_void,
            );
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        }
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 0.0f32].as_ptr();
        let one = [1.0f32].as_ptr();
        let mut t = current_time as f32;
        let info = self.info();

        let proj_matrix = perspective(50.0, info.width as f32 / info.height as f32, 0.1, 1000.0);

        t *= 0.1;
        t -= t.floor();

        unsafe {
            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, one);

            gl::UseProgram(self.render_prog);

            gl::Uniform1f(self.uniforms.time, t);
            gl::UniformMatrix4fv(
                self.uniforms.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE);

            gl::BindVertexArray(self.star_vao);

            gl::Enable(gl::PROGRAM_POINT_SIZE);
            gl::DrawArrays(gl::POINTS, 0, NUM_STARTS as GLsizei);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.render_prog);
            gl::DeleteVertexArrays(1, &self.star_vao);
            gl::DeleteTextures(1, &self.star_texture);
            gl::DeleteBuffers(1, &self.star_buffer);
        }
    }
}

fn main() {
    StarfieldApp::default().run()
}
