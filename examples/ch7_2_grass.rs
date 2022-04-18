/*
 * Copyright � 2012-2015 Graham Sellers
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
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
    grass_buffer: u32,
    grass_vao: u32,

    grass_program: u32,

    tex_grass_color: u32,
    tex_grass_length: u32,
    tex_grass_orientation: u32,
    tex_grass_bend: u32,

    uniforms: Uniforms,
}

#[derive(Default)]
struct Uniforms {
    mvp_matrix: i32,
}

impl Application for App {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Grass".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        let grass_vs_src = r#"
            // Vertex Shader
            // Graham Sellers
            // OpenGL SuperBible
            #version 460 core

            // Incoming per vertex position
            in vec4 vVertex;

            // Output varyings
            out vec4 color;

            uniform mat4 mvpMatrix;

            layout (binding = 0) uniform sampler1D grasspalette_texture;
            layout (binding = 1) uniform sampler2D length_texture;
            layout (binding = 2) uniform sampler2D orientation_texture;
            layout (binding = 3) uniform sampler2D grasscolor_texture;
            layout (binding = 4) uniform sampler2D bend_texture;

            int random(int seed, int iterations) {
                int value = seed;
                int n;

                for (n = 0; n < iterations; n++) {
                    value = ((value >> 7) ^ (value << 9)) * 15485863;
                }

                return value;
            }

            vec4 random_vector(int seed) {
                int r = random(gl_InstanceID, 4);
                int g = random(r, 2);
                int b = random(g, 2);
                int a = random(b, 2);

                return vec4(float(r & 0x3FF) / 1024.0,
                            float(g & 0x3FF) / 1024.0,
                            float(b & 0x3FF) / 1024.0,
                            float(a & 0x3FF) / 1024.0);
            }

            mat4 construct_rotation_matrix(float angle) {
                float st = sin(angle);
                float ct = cos(angle);

                return mat4(vec4(ct, 0.0, st, 0.0),
                            vec4(0.0, 1.0, 0.0, 0.0),
                            vec4(-st, 0.0, ct, 0.0),
                            vec4(0.0, 0.0, 0.0, 1.0));
            }

            void main(void) {
                vec4 offset = vec4(float(gl_InstanceID >> 10) - 512.0,
                                         0.0f,
                                         float(gl_InstanceID & 0x3FF) - 512.0,
                                         0.0f);
                int number1 = random(gl_InstanceID, 3);
                int number2 = random(number1, 2);
                offset += vec4(float(number1 & 0xFF) / 256.0,
                               0.0f,
                               float(number2 & 0xFF) / 256.0,
                               0.0f);
                // float angle = float(random(number2, 2) & 0x3FF) / 1024.0; 

                vec2 texcoord = offset.xz / 1024.0 + vec2(0.5);

                // float bend_factor = float(random(number2, 7) & 0x3FF) / 1024.0;
                float bend_factor = texture(bend_texture, texcoord).r * 2.0;
                float bend_amount = cos(vVertex.y);

                float angle = texture(orientation_texture, texcoord).r * 2.0 * 3.141592;
                mat4 rot = construct_rotation_matrix(angle);
                vec4 position = (rot * (vVertex + vec4(0.0, 0.0, bend_amount * bend_factor, 0.0))) + offset;

                position *= vec4(1.0, texture(length_texture, texcoord).r * 0.9 + 0.3, 1.0, 1.0);

                gl_Position = mvpMatrix * position; // (rot * position);
                // color = vec4(random_vector(gl_InstanceID).xyz * vec3(0.1, 0.5, 0.1) + vec3(0.1, 0.4, 0.1), 1.0);
                // color = texture(orientation_texture, texcoord);
                color = texture(grasspalette_texture, texture(grasscolor_texture, texcoord).r) + 
                        vec4(random_vector(gl_InstanceID).xyz * vec3(0.1, 0.5, 0.1), 1.0); 
            }
        "#;

        let grass_fs_src = r#"
            // Fragment Shader
            // Graham Sellers
            // OpenGL SuperBible
            #version 460 core

            in vec4 color;

            out vec4 output_color;

            void main(void) {
                output_color = color;
            }
        "#;

        #[rustfmt::skip]
        let grass_blade = [
            -0.3f32,  0.0f32,
             0.3f32,  0.0f32,
            -0.20f32, 1.0f32,
             0.1f32,  1.3f32,
            -0.05f32, 2.3f32,
             0.0f32,  3.3f32,
        ];

        unsafe {
            gl::CreateVertexArrays(1, &mut self.grass_vao);
            gl::BindVertexArray(self.grass_vao);

            gl::CreateBuffers(1, &mut self.grass_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.grass_buffer);
            gl::NamedBufferData(
                self.grass_buffer,
                size_of_val(&grass_blade) as _,
                grass_blade.as_ptr() as _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, null());
            gl::EnableVertexAttribArray(0);

            self.grass_program = gl::CreateProgram();

            let vs_src = std::ffi::CString::new(grass_vs_src).unwrap();
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vs, 1, &vs_src.as_ptr(), null());
            gl::CompileShader(vs);

            let fs_src = std::ffi::CString::new(grass_fs_src).unwrap();
            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fs, 1, &fs_src.as_ptr(), null());
            gl::CompileShader(fs);

            gl::AttachShader(self.grass_program, vs);
            gl::AttachShader(self.grass_program, fs);
            gl::LinkProgram(self.grass_program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let c_str = std::ffi::CString::new("mvpMatrix").unwrap();
            self.uniforms.mvp_matrix = gl::GetUniformLocation(self.grass_program, c_str.as_ptr());

            let load = |path| ktx::file::load(path).unwrap().0;
            gl::ActiveTexture(gl::TEXTURE1);
            self.tex_grass_length = load("media/textures/grass_length.ktx");
            gl::ActiveTexture(gl::TEXTURE2);
            self.tex_grass_orientation = load("media/textures/grass_orientation.ktx");
            gl::ActiveTexture(gl::TEXTURE3);
            self.tex_grass_color = load("media/textures/grass_color.ktx");
            gl::ActiveTexture(gl::TEXTURE4);
            self.tex_grass_bend = load("media/textures/grass_bend.ktx");

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
    }

    fn render(&mut self, current_time: f64) {
        unsafe {
            let t = (current_time * 0.02) as f32;
            let r = 550f32;

            let black = [0., 0., 0., 0., 1.0f32].as_ptr();
            let one = 1f32;
            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            let mv_matrix = lookat(
                vec3!(t.sin() * r, 25.0, t.cos() * r),
                vec3!(0.0, -50., 0.),
                vec3!(0.0, 1.0, 0.0),
            );
            let AppConfig { width, height, .. } = self.info();
            let prj_matrix = perspective(45., width as f32 / height as f32, 0.01, 1000.);
            let mvp_matrix = prj_matrix * mv_matrix;

            gl::UseProgram(self.grass_program);
            gl::UniformMatrix4fv(
                self.uniforms.mvp_matrix,
                1,
                gl::FALSE,
                addr_of!(mvp_matrix) as _,
            );

            gl::BindVertexArray(self.grass_vao);
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 6, 1024 * 1024);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteProgram(self.grass_program);
            gl::DeleteVertexArrays(1, &self.grass_vao);
            gl::DeleteBuffers(1, &self.grass_buffer);
            gl::DeleteTextures(1, &self.tex_grass_bend);
            gl::DeleteTextures(1, &self.tex_grass_color);
            gl::DeleteTextures(1, &self.tex_grass_length);
            gl::DeleteTextures(1, &self.tex_grass_orientation);
        }
    }
}

fn main() {
    App::default().run()
}
