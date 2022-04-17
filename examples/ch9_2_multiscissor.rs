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
struct MultiScissorApp {
    program: GLuint,
    vao: GLuint,
    position_buffer: GLuint,
    index_buffer: GLuint,
    uniform_buffer: GLuint,
}

impl Application for MultiScissorApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Multiple Scissors".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let cstring = |str| std::ffi::CString::new(str).unwrap();

        let vs_src = cstring(
            r#"
            #version 420 core

            in vec4 position;

            out VS_OUT
            {
                vec4 color;
            } vs_out;

            void main(void)
            {
                gl_Position = position;
                vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
            }"#,
        );

        let gs_src = cstring(
            r#"
            #version 420 core

            layout (triangles, invocations = 4) in;
            layout (triangle_strip, max_vertices = 3) out;

            layout (std140, binding = 0) uniform transform_block
            {
                mat4 mvp_matrix[4];
            };

            in VS_OUT
            {
                vec4 color;
            } gs_in[];

            out GS_OUT
            {
                vec4 color;
            } gs_out;

            void main(void)
            {
                for (int i = 0; i < gl_in.length(); i++)
                {
                    gs_out.color = gs_in[i].color;
                    gl_Position = mvp_matrix[gl_InvocationID] *
                                gl_in[i].gl_Position;
                    gl_ViewportIndex = gl_InvocationID;
                    EmitVertex();
                }
                EndPrimitive();
            }"#,
        );

        let fs_src = cstring(
            r#"
            #version 420 core

            out vec4 color;

            in GS_OUT
            {
                vec4 color;
            } fs_in;

            void main(void)
            {
                color = fs_in.color;
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

            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            #[rustfmt::skip]
            let vertex_indices: &[GLushort] = &[
                0, 1, 2,
                2, 1, 3,
                2, 3, 4,
                4, 3, 5,
                4, 5, 6,
                6, 5, 7,
                6, 7, 0,
                0, 7, 1,
                6, 0, 2,
                2, 4, 6,
                7, 5, 3,
                7, 3, 1,
            ];

            #[rustfmt::skip]
            let vertex_positions: &[GLfloat] = &[
                -0.25f32, -0.25f32, -0.25f32,
                -0.25f32,  0.25f32, -0.25f32,
                 0.25f32, -0.25f32, -0.25f32,
                 0.25f32,  0.25f32, -0.25f32,
                 0.25f32, -0.25f32,  0.25f32,
                 0.25f32,  0.25f32,  0.25f32,
                -0.25f32, -0.25f32,  0.25f32,
                -0.25f32,  0.25f32,  0.25f32,
            ];

            gl::GenBuffers(1, &mut self.position_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.position_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(vertex_positions) as GLsizeiptr,
                vertex_positions.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
            gl::EnableVertexAttribArray(0);

            gl::GenBuffers(1, &mut self.index_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(vertex_indices) as GLsizeiptr,
                vertex_indices.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );

            gl::GenBuffers(1, &mut self.uniform_buffer);
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.uniform_buffer);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                (4 * size_of::<Mat4>()) as GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );

            gl::Enable(gl::CULL_FACE);
            // gl::FrontFace(gl::CW);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
    }

    fn render(&self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();
        let one = 1.0f32;

        unsafe {
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            // Turn on scissor testing
            gl::Enable(gl::SCISSOR_TEST);

            // Each rectangle will be 7/16 of the screen
            let AppConfig { width, height, .. } = self.info();
            let viewport_width = (7 * width / 16) as i32;
            let viewport_height = (7 * height / 16) as i32;

            // Four rectangles - lower left first...
            gl::ScissorIndexed(0, 0, 0, viewport_width, viewport_height);

            // Lower right...
            gl::ScissorIndexed(
                1,
                width as i32 - viewport_width,
                0,
                viewport_width,
                viewport_height,
            );

            // Upper left...
            gl::ScissorIndexed(
                2,
                0,
                height as i32 - viewport_height,
                viewport_width,
                viewport_height,
            );

            // Upper right...
            gl::ScissorIndexed(
                3,
                width as i32 - viewport_width,
                height as i32 - viewport_height,
                viewport_width,
                viewport_height,
            );

            let proj_matrix = perspective(50.0, width as f32 / height as f32, 0.1, 1000.0);

            gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, self.uniform_buffer);
            let mv_matrix_array = std::slice::from_raw_parts_mut(
                gl::MapBufferRange(
                    gl::UNIFORM_BUFFER,
                    0,
                    (4 * size_of::<Mat4>()) as GLsizeiptr,
                    gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
                ) as *mut Mat4,
                4,
            );

            for i in 0..4 {
                mv_matrix_array[i] = proj_matrix
                    * translate(0.0, 0.0, -2.0)
                    * rotate_with_axis(current_time as f32 * 45.0 * (i + 1) as f32, 0.0, 1.0, 0.0)
                    * rotate_with_axis(current_time as f32 * 81.0 * (i + 1) as f32, 1.0, 0.0, 0.0);
            }

            gl::UnmapBuffer(gl::UNIFORM_BUFFER);

            gl::UseProgram(self.program);

            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_SHORT, null());
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.program);
            let bufs = [self.index_buffer, self.position_buffer, self.uniform_buffer];
            gl::DeleteBuffers(bufs.len() as GLsizei, bufs.as_ptr());
        }
    }
}

fn main() {
    MultiScissorApp::default().run()
}
