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

const MAX_SCENE_WIDTH: i32 = 2048;
const MAX_SCENE_HEIGHT: i32 = 2048;
const SPHERE_COUNT: usize = 32;

unsafe fn _print_shader_log(msg: &str, shader: GLuint) {
    let mut len = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    if len != 0 {
        let buf = [0u8; 1024];
        gl::GetShaderInfoLog(shader, 1024, null_mut(), buf.as_ptr() as *mut GLchar);
        println!("{} :", msg);
        println!(
            "{}",
            std::str::from_utf8(&buf).unwrap_or("invaild utf-8 log")
        );
    }
}

#[derive(Default)]
struct HDRBloomApp {
    render_fbo: GLuint,
    filter_fbo: [GLuint; 2],

    tex_scene: GLuint,
    tex_depth: GLuint,
    tex_brightpass: GLuint,
    tex_filter: [GLuint; 2],

    program_render: GLuint,
    program_filter: GLuint,
    program_resolve: GLuint,
    vao: GLuint,
    exposure: f32,
    paused: bool,
    bloom_factor: f32,
    show_bloom: bool,
    show_scene: bool,
    show_prefilter: bool,
    bloom_thresh_min: f32,
    bloom_thresh_max: f32,

    uniforms: Uniforms,

    ubo_transform: GLuint,
    ubo_material: GLuint,

    object: Object,
}

#[derive(Default)]
struct Uniforms {
    scene: SceneUniforms,
    resolve: ResolveUniforms,
}

#[derive(Default)]
struct SceneUniforms {
    bloom_thresh_min: GLint,
    bloom_thresh_max: GLint,
}

#[derive(Default)]
struct ResolveUniforms {
    exposure: GLint,
    bloom_factor: GLint,
    scene_factor: GLint,
}

impl HDRBloomApp {
    fn new() -> Self {
        Self {
            exposure: 1.0,
            bloom_factor: 1.0,
            show_bloom: true,
            show_scene: true,
            bloom_thresh_max: 1.2,
            bloom_thresh_min: 0.8,
            ..Default::default()
        }
    }

    fn load_shaders(&mut self) {
        unsafe fn get_loc(prog: GLuint, name: &str) -> GLint {
            let name = std::ffi::CString::new(name).unwrap();
            gl::GetUniformLocation(prog, name.as_ptr())
        }

        unsafe {
            if self.program_render != 0 {
                gl::DeleteProgram(self.program_render);
            }

            let vs = shader::load(
                "media/shaders/hdrbloom/hdrbloom-scene.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/hdrbloom/hdrbloom-scene.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );
            self.program_render = program::link_from_shaders(&[vs, fs], true);

            self.uniforms.scene.bloom_thresh_min = get_loc(self.program_render, "bloom_thresh_min");
            self.uniforms.scene.bloom_thresh_max = get_loc(self.program_render, "bloom_thresh_max");

            if self.program_filter != 0 {
                gl::DeleteProgram(self.program_filter);
            }

            let vs = shader::load(
                "media/shaders/hdrbloom/hdrbloom-filter.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/hdrbloom/hdrbloom-filter.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );
            self.program_filter = program::link_from_shaders(&[vs, fs], true);

            if self.program_resolve != 0 {
                gl::DeleteProgram(self.program_resolve);
            }

            let vs = shader::load(
                "media/shaders/hdrbloom/hdrbloom-resolve.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/hdrbloom/hdrbloom-resolve.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );
            self.program_resolve = program::link_from_shaders(&[vs, fs], true);

            self.uniforms.resolve.exposure = get_loc(self.program_resolve, "exposure");
            self.uniforms.resolve.bloom_factor = get_loc(self.program_resolve, "bloom_factor");
            self.uniforms.resolve.scene_factor = get_loc(self.program_resolve, "scene_factor");
        }
    }
}

impl Application for HDRBloomApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - HDR Bloom".into(),
            ..AppConfig::default()
        }
    }

    fn startup(&mut self) {
        unsafe {
            let buffers = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];

            gl::CreateVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            self.load_shaders();

            gl::GenFramebuffers(1, &mut self.render_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.render_fbo);

            gl::GenTextures(1, &mut self.tex_scene);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_scene);
            gl::TexStorage2D(
                gl::TEXTURE_2D,
                1,
                gl::RGBA16F,
                MAX_SCENE_WIDTH,
                MAX_SCENE_HEIGHT,
            );
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.tex_scene, 0);
            gl::GenTextures(1, &mut self.tex_brightpass);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_brightpass);
            gl::TexStorage2D(
                gl::TEXTURE_2D,
                1,
                gl::RGBA16F,
                MAX_SCENE_WIDTH,
                MAX_SCENE_HEIGHT,
            );
            gl::FramebufferTexture(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT1,
                self.tex_brightpass,
                0,
            );
            gl::GenTextures(1, &mut self.tex_depth);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_depth);
            gl::TexStorage2D(
                gl::TEXTURE_2D,
                1,
                gl::DEPTH_COMPONENT32F,
                MAX_SCENE_WIDTH,
                MAX_SCENE_HEIGHT,
            );
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, self.tex_depth, 0);
            gl::DrawBuffers(2, buffers.as_ptr());

            gl::GenFramebuffers(2, self.filter_fbo.as_mut_ptr());
            gl::GenTextures(2, self.tex_filter.as_mut_ptr());
            for i in 0..2 {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.filter_fbo[i]);
                gl::BindTexture(gl::TEXTURE_2D, self.tex_filter[i]);
                let (w, h) = match i {
                    0 => (MAX_SCENE_HEIGHT, MAX_SCENE_WIDTH),
                    1 => (MAX_SCENE_WIDTH, MAX_SCENE_HEIGHT),
                    _ => unreachable!(),
                };
                gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGBA16F, w, h);
                gl::FramebufferTexture(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    self.tex_filter[i],
                    0,
                );
                gl::DrawBuffers(1, &gl::COLOR_ATTACHMENT0);
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            self.object.load("media/objects/torus.sbm");

            gl::GenBuffers(1, &mut self.ubo_transform);
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo_transform);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                ((2 + SPHERE_COUNT) * size_of::<Mat4>()) as GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );

            #[repr(C)]
            struct Material {
                diffuse_color: Vec3,
                _p1: u32, // pad,
                specular_color: Vec3,
                specular_power: f32,
                ambient_color: Vec3,
                _p2: u32, // pad
            }

            gl::GenBuffers(1, &mut self.ubo_material);
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo_material);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                (SPHERE_COUNT * size_of::<Material>()) as GLsizeiptr,
                null(),
                gl::STATIC_DRAW,
            );

            let m = std::slice::from_raw_parts_mut(
                gl::MapBufferRange(
                    gl::UNIFORM_BUFFER,
                    0,
                    (SPHERE_COUNT * size_of::<Material>()) as GLsizeiptr,
                    gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
                ) as *mut Material,
                SPHERE_COUNT,
            );
            let mut ambient = 0.002f32;
            for i in 0..m.len() {
                let fi = 3.14159267 * i as f32 / 8.0;
                m[i].diffuse_color = vec3!(
                    fi.sin() * 0.5 + 0.5,
                    (fi + 1.345).sin() * 0.5 + 0.5,
                    (fi + 2.567).sin() * 0.5 + 0.5
                );
                m[i].specular_color = vec3!(2.8, 2.8, 2.9);
                m[i].specular_power = 30.0;
                m[i].ambient_color = vec3!(ambient * 0.025, ambient * 0.025, ambient * 0.025);
                ambient *= 1.5;
            }
            gl::UnmapBuffer(gl::UNIFORM_BUFFER);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.render_fbo);
            gl::DeleteFramebuffers(2, self.filter_fbo.as_ptr());

            gl::DeleteTextures(1, &self.tex_scene);
            gl::DeleteTextures(1, &self.tex_depth);
            gl::DeleteTextures(1, &self.tex_brightpass);
            gl::DeleteTextures(2, self.tex_filter.as_ptr());

            gl::DeleteProgram(self.program_render);
            gl::DeleteProgram(self.program_filter);
            gl::DeleteProgram(self.program_resolve);

            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.ubo_transform);
            gl::DeleteBuffers(1, &self.ubo_material);
        }
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.0, 0.0, 0.0, 1.0].as_ptr();
        let one = 1.0f32;
        let info = self.info();

        unsafe {
            static mut LAST_TIME: f64 = 0.0;
            static mut TOTAL_TIME: f64 = 0.0;
            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;
            let t = TOTAL_TIME as f32;

            gl::Viewport(0, 0, info.width as GLsizei, info.height as GLsizei);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.render_fbo);
            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::COLOR, 1, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            gl::UseProgram(self.program_render);

            gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, self.ubo_transform);
            #[repr(C)]
            struct Transforms {
                mat_proj: Mat4,
                mat_view: Mat4,
                mat_model: [Mat4; SPHERE_COUNT],
            }
            let transforms = &mut *(gl::MapBufferRange(
                gl::UNIFORM_BUFFER,
                0,
                size_of::<Transforms>() as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT,
            ) as *mut Transforms);
            transforms.mat_proj =
                perspective(50.0, info.width as f32 / info.height as f32, 1.0, 1000.0);
            transforms.mat_view = translate(0.0, 0.0, -20.0);
            for i in 0..SPHERE_COUNT {
                let fi = 3.1415926 * i as f32 / 16.0;
                let r = if (i % 2) == 1 { 0.6 } else { 1.5 };
                transforms.mat_model[i] = translate(
                    (t + fi).cos() * 5.0 * r,
                    (t + fi * 4.0).sin() * 4.0,
                    (t + fi).sin() * 5.0 * r,
                ) * rotate(
                    (t + fi * 2.13).sin() * 75.0,
                    (t + fi * 1.37).cos() * 92.0,
                    0.0,
                );
            }
            gl::UnmapBuffer(gl::UNIFORM_BUFFER);
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 1, self.ubo_material);

            gl::Uniform1f(self.uniforms.scene.bloom_thresh_min, self.bloom_thresh_min);
            gl::Uniform1f(self.uniforms.scene.bloom_thresh_max, self.bloom_thresh_max);

            self.object.render_objects(0, SPHERE_COUNT as u32, 0);

            gl::Disable(gl::DEPTH_TEST);

            gl::UseProgram(self.program_filter);

            gl::BindVertexArray(self.vao);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.filter_fbo[0]);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_brightpass);
            gl::Viewport(0, 0, info.height as GLsizei, info.width as GLsizei);

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.filter_fbo[1]);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_filter[0]);
            gl::Viewport(0, 0, info.width as GLsizei, info.height as GLsizei);

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

            gl::UseProgram(self.program_resolve);

            gl::Uniform1f(self.uniforms.resolve.exposure, self.exposure);
            if self.show_prefilter {
                gl::Uniform1f(self.uniforms.resolve.bloom_factor, 0.0);
                gl::Uniform1f(self.uniforms.resolve.scene_factor, 1.0);
            } else {
                gl::Uniform1f(
                    self.uniforms.resolve.bloom_factor,
                    if self.show_bloom {
                        self.bloom_factor
                    } else {
                        0.0
                    },
                );
                gl::Uniform1f(
                    self.uniforms.resolve.scene_factor,
                    if self.show_scene { 1.0 } else { 0.0 },
                );
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.tex_filter[1]);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(
                gl::TEXTURE_2D,
                if self.show_prefilter {
                    self.tex_brightpass
                } else {
                    self.tex_scene
                },
            );

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press | glfw::Action::Repeat = press {
            match key {
                glfw::Key::B => self.show_bloom = !self.show_bloom,
                glfw::Key::V => self.show_scene = !self.show_scene,
                glfw::Key::A => self.bloom_factor += 0.1,
                glfw::Key::Z => self.bloom_factor -= 0.1,
                glfw::Key::S => self.bloom_thresh_min += 0.1,
                glfw::Key::X => self.bloom_thresh_min -= 0.1,
                glfw::Key::D => self.bloom_thresh_max += 0.1,
                glfw::Key::C => self.bloom_thresh_max -= 0.1,
                glfw::Key::R => self.load_shaders(),
                glfw::Key::N => self.show_prefilter = !self.show_prefilter,
                glfw::Key::KpAdd => self.exposure *= 1.1,
                glfw::Key::KpSubtract => self.exposure /= 1.1,
                _ => {}
            }
        }
    }
}

fn main() {
    HDRBloomApp::new().run();
}
