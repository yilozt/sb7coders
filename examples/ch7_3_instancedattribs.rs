/*
 * Copyright � 2012-2015 Graham Sellers
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the Software),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED AS IS, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

use sb7::prelude::*;

#[derive(Default)]
struct App {
    square_buffer: u32,
    square_vao: u32,
    square_program: u32,
}

impl Application for App {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Instanced Attributes".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let square_vs_source = r#"
            #version 410 core

            layout (location = 0) in vec4 position;
            layout (location = 1) in vec4 instance_color;
            layout (location = 2) in vec4 instance_position;

            out Fragment
            {
                vec4 color;
            } fragment;

            void main(void)
            {
                gl_Position = (position + instance_position) * vec4(0.25, 0.25, 1.0, 1.0);
                fragment.color = instance_color;
            }                  
        "#;

        let square_fs_source = r#"
            #version 410 core
            precision highp float;
            
            in Fragment
            {
                vec4 color;
            } fragment;
            
            out vec4 color;
            
            void main(void)
            {
                color = fragment.color;
            }
        "#;

        #[rustfmt::skip]
        let square_vertices = [
            -1.0f32, -1.0f32, 0.0f32, 1.0f32,
             1.0f32, -1.0f32, 0.0f32, 1.0f32,
             1.0f32,  1.0f32, 0.0f32, 1.0f32,
            -1.0f32,  1.0f32, 0.0f32, 1.0f32
        ];

        #[rustfmt::skip]
        let instance_colors = [
            1.0f32, 0.0f32, 0.0f32, 1.0f32,
            0.0f32, 1.0f32, 0.0f32, 1.0f32,
            0.0f32, 0.0f32, 1.0f32, 1.0f32,
            1.0f32, 1.0f32, 0.0f32, 1.0f32
        ];

        #[rustfmt::skip]
        let instance_positions = [
            -2.0f32, -2.0f32, 0.0f32, 0.0f32,
             2.0f32, -2.0f32, 0.0f32, 0.0f32,
             2.0f32,  2.0f32, 0.0f32, 0.0f32,
            -2.0f32,  2.0f32, 0.0f32, 0.0f32
        ];

        unsafe {
            gl::CreateVertexArrays(1, &mut self.square_vao);
            gl::BindVertexArray(self.square_vao);

            gl::GenBuffers(1, &mut self.square_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.square_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of_val(&square_vertices)
                    + size_of_val(&instance_colors)
                    + size_of_val(&instance_positions)) as _,
                null(),
                gl::STATIC_DRAW,
            );

            let mut offset = 0;
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                offset,
                size_of_val(&square_vertices) as _,
                square_vertices.as_ptr() as _,
            );
            offset += size_of_val(&square_vertices) as isize;

            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                offset,
                size_of_val(&instance_colors) as _,
                instance_colors.as_ptr() as _,
            );
            offset += size_of_val(&instance_colors) as isize;

            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                offset,
                size_of_val(&instance_positions) as _,
                instance_positions.as_ptr() as _,
            );

            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 0, 0 as _);
            gl::VertexAttribPointer(
                1,
                4,
                gl::FLOAT,
                gl::FALSE,
                0,
                size_of_val(&square_vertices) as _,
            );
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                0,
                (size_of_val(&square_vertices) + size_of_val(&instance_colors)) as _,
            );

            gl::EnableVertexArrayAttrib(self.square_vao, 0);
            gl::EnableVertexArrayAttrib(self.square_vao, 1);
            gl::EnableVertexArrayAttrib(self.square_vao, 2);

            gl::VertexAttribDivisor(1, 1);
            gl::VertexAttribDivisor(2, 1);

            self.square_program = gl::CreateProgram();

            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            let src = std::ffi::CString::new(square_vs_source).unwrap();
            gl::ShaderSource(vs, 1, &src.as_ptr(), null());
            gl::CompileShader(vs);
            gl::AttachShader(self.square_program, vs);
            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            let src = std::ffi::CString::new(square_fs_source).unwrap();
            gl::ShaderSource(fs, 1, &src.as_ptr(), null());
            gl::CompileShader(fs);
            gl::AttachShader(self.square_program, fs);

            gl::LinkProgram(self.square_program);
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.square_vao);
            gl::DeleteBuffers(1, &self.square_buffer);
            gl::DeleteProgram(self.square_program);
        }
    }

    fn render(&self, _current_time: f64) {
        unsafe {
            let black = [0.0, 0.0, 0.0, 1.0f32].as_ptr();

            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::UseProgram(self.square_program);
            gl::BindVertexArray(self.square_vao);
            gl::DrawArraysInstanced(gl::TRIANGLE_FAN, 0, 4, 4);
        }
    }
}

fn main() {
    App::default().run();
}
