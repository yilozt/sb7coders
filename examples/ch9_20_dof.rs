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

const FBO_SIZE: usize = 2048;
#[allow(unused)]
const FRUSTUM_DEPTH: usize = 1000;

#[derive(Default)]
struct DofApp {
    view_program: GLuint,
    filter_program: GLuint,
    display_program: GLuint,

    quad_vao: GLuint,

    uniforms: Uniforms,

    depth_fbo: GLuint,
    depth_tex: GLuint,
    color_tex: GLuint,
    temp_tex: GLuint,

    objects: [Object; 5],

    camera_view_matrix: Mat4,
    camera_proj_matrix: Mat4,

    paused: bool,

    focal_distance: f32,
    focal_depth: f32,
}

#[derive(Default)]
struct Object {
    obj: sb7::object::Object,
    model_matrix: Mat4,
    diffuse_albedo: Vec4,
}

#[derive(Default)]
struct Uniforms {
    dof: DofUniforms,
    view: ViewUniforms,
}

#[derive(Default)]
struct DofUniforms {
    focal_distance: GLint,
    focal_depth: GLint,
}

#[derive(Default)]
struct ViewUniforms {
    mv_matrix: GLint,
    proj_matrix: GLint,

    full_shading: GLint,
    diffuse_albedo: GLint,
}

impl Application for DofApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Depth Of Field".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        self.load_shaders();

        let object_names = [
            "media/objects/dragon.sbm",
            "media/objects/sphere.sbm",
            "media/objects/cube.sbm",
            "media/objects/cube.sbm",
            "media/objects/cube.sbm",
        ];

        let object_colors = [
            vec4!(1.0f32, 0.7f32, 0.8f32, 1.0f32),
            vec4!(0.7f32, 0.8f32, 1.0f32, 1.0f32),
            vec4!(0.3f32, 0.9f32, 0.4f32, 1.0f32),
            vec4!(0.6f32, 0.4f32, 0.9f32, 1.0f32),
            vec4!(0.8f32, 0.2f32, 0.1f32, 1.0f32),
        ];

        for i in 0..self.objects.len() {
            self.objects[i].obj.load(object_names[i]);
            self.objects[i].diffuse_albedo = object_colors[i];
        }

        unsafe {
            gl::GenFramebuffers(1, &mut self.depth_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);

            gl::GenTextures(1, &mut self.depth_tex);
            gl::BindTexture(gl::TEXTURE_2D, self.depth_tex);
            gl::TexStorage2D(
                gl::TEXTURE_2D,
                11,
                gl::DEPTH_COMPONENT32F,
                FBO_SIZE as GLsizei,
                FBO_SIZE as GLsizei,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::GenTextures(1, &mut self.color_tex);
            gl::BindTexture(gl::TEXTURE_2D, self.color_tex);
            gl::TexStorage2D(
                gl::TEXTURE_2D,
                1,
                gl::RGBA32F,
                FBO_SIZE as GLsizei,
                FBO_SIZE as GLsizei,
            );

            gl::GenTextures(1, &mut self.temp_tex);
            gl::BindTexture(gl::TEXTURE_2D, self.temp_tex);
            gl::TexStorage2D(
                gl::TEXTURE_2D,
                1,
                gl::RGBA32F,
                FBO_SIZE as GLsizei,
                FBO_SIZE as GLsizei,
            );

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, self.depth_tex, 0);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.color_tex, 0);

            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::Enable(gl::DEPTH_TEST);

            gl::GenVertexArrays(1, &mut self.quad_vao);
            gl::BindVertexArray(self.quad_vao);
        }
    }

    fn render(&mut self, current_time: f64) {
        let win_info = self.info();

        unsafe {
            static mut TOTAL_TIME: f64 = 0.0;
            static mut LAST_TIME: f64 = 0.0;
            
            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let f = (TOTAL_TIME + 30.0) as f32;

            let view_position = vec3!(0.0, 0.0, 40.0f32);

            self.camera_proj_matrix = perspective(
                50.0,
                win_info.width as f32 / win_info.height as f32,
                2.0,
                300.0,
            );

            self.camera_view_matrix =
                lookat(view_position, vec3!(0.0, 0.0, 0.0f32), vec3!(0.0, 1.0, 0.0));

            self.objects[0].model_matrix = translate(5.0, 0.0, 20.0f32)
                * rotate_with_axis(f * 14.5, 0.0, 1.0, 0.0)
                * rotate_with_axis(20.0, 1.0, 0.0, 0.0)
                * translate(0.0, -4.0, 0.0f32);

            self.objects[1].model_matrix = translate(-5.0, 0.0, 0.0f32)
                * rotate_with_axis(f * 14.5, 0.0, 1.0, 0.0)
                * rotate_with_axis(20.0, 1.0, 0.0, 0.0)
                * translate(0.0, -4.0, 0.0f32);

            self.objects[2].model_matrix = translate(-15.0, 0.0, -20.0f32)
                * rotate_with_axis(f * 14.5, 0.0, 1.0, 0.0)
                * rotate_with_axis(20.0, 1.0, 0.0, 0.0)
                * translate(0.0, -4.0, 0.0f32);

            self.objects[3].model_matrix = translate(-25.0, 0.0, -40.0f32)
                * rotate_with_axis(f * 14.5, 0.0, 1.0, 0.0)
                * rotate_with_axis(20.0, 1.0, 0.0, 0.0)
                * translate(0.0, -4.0, 0.0f32);

            self.objects[4].model_matrix = translate(-35.0, 0.0, -60.0f32)
                * rotate_with_axis(f * 14.5, 0.0, 1.0, 0.0)
                * rotate_with_axis(20.0, 1.0, 0.0, 0.0)
                * translate(0.0, -4.0, 0.0f32);

            gl::Enable(gl::DEPTH_TEST);
            self.render_scene();

            gl::UseProgram(self.filter_program);

            gl::BindImageTexture(
                0,
                self.color_tex,
                0,
                gl::FALSE,
                0,
                gl::READ_ONLY,
                gl::RGBA32F,
            );
            gl::BindImageTexture(
                1,
                self.temp_tex,
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA32F,
            );

            gl::DispatchCompute(win_info.height as GLuint, 1, 1);

            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            gl::BindImageTexture(
                0,
                self.temp_tex,
                0,
                gl::FALSE,
                0,
                gl::READ_ONLY,
                gl::RGBA32F,
            );
            gl::BindImageTexture(
                1,
                self.color_tex,
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA32F,
            );

            gl::DispatchCompute(win_info.width as GLuint, 1, 1);

            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.color_tex);
            gl::Disable(gl::DEPTH_TEST);
            gl::UseProgram(self.display_program);
            gl::Uniform1f(self.uniforms.dof.focal_distance, self.focal_distance);
            gl::Uniform1f(self.uniforms.dof.focal_depth, self.focal_depth);
            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        if let Some(win) = imgui::Window::new("Dof").always_auto_resize(true).begin(ui) {
            imgui::Slider::new("Focal distance", 0.0f32, 100.0f32)
                .build(ui, &mut self.focal_distance);
            imgui::Slider::new("Focal depth", 0.0f32, 100.0f32).build(ui, &mut self.focal_depth);
            ui.text(" ");
            ui.text("Q: Increas focal distance");
            ui.text("A: Decrease focal distance");
            ui.text("W: Increase focal depth");
            ui.text("S: Decrease focal depth");
            ui.text("P: Pause");
            win.end();
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press | glfw::Action::Repeat = press {
            match key {
                glfw::Key::Q => self.focal_distance *= 1.1,
                glfw::Key::A => self.focal_distance /= 1.1,
                glfw::Key::W => self.focal_depth *= 1.1,
                glfw::Key::S => self.focal_depth /= 1.1,
                glfw::Key::R => self.load_shaders(),
                glfw::Key::P => self.paused = !self.paused,
                _ => {}
            }
        }
    }
}

impl DofApp {
    fn render_scene(&mut self) {
        let win_info = self.info();
        let gray = [0.1, 0.1, 0.1, 0.1f32].as_ptr();
        let ones = [1.0f32].as_ptr();
        let attachments = [gl::COLOR_ATTACHMENT0].as_ptr();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);

            gl::DrawBuffers(1, attachments);
            gl::Viewport(0, 0, win_info.width as GLint, win_info.height as GLint);
            gl::ClearBufferfv(gl::COLOR, 0, gray);
            gl::ClearBufferfv(gl::DEPTH, 0, ones);
            gl::UseProgram(self.view_program);
            gl::UniformMatrix4fv(
                self.uniforms.view.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(self.camera_proj_matrix) as *const GLfloat,
            );

            for object in &self.objects {
                let view_matrix = self.camera_view_matrix * object.model_matrix;
                gl::UniformMatrix4fv(
                    self.uniforms.view.mv_matrix,
                    1,
                    gl::FALSE,
                    addr_of!(view_matrix) as *const GLfloat,
                );
                gl::Uniform3fv(
                    self.uniforms.view.diffuse_albedo,
                    1,
                    addr_of!(object.diffuse_albedo) as *const GLfloat,
                );
                self.objects[0].obj.render();
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    fn load_shaders(&mut self) {
        fn get_loc(program: GLuint, name: &str) -> GLint {
            let name = std::ffi::CString::new(name).unwrap();
            unsafe { gl::GetUniformLocation(program, name.as_ptr()) }
        }

        if self.view_program != 0 {
            unsafe { gl::DeleteProgram(self.view_program) };
        }

        self.view_program = program::link_from_shaders(
            &[
                shader::load("media/shaders/dof/render.vs.glsl", gl::VERTEX_SHADER, true),
                shader::load(
                    "media/shaders/dof/render.fs.glsl",
                    gl::FRAGMENT_SHADER,
                    true,
                ),
            ],
            true,
        );

        self.uniforms.view.proj_matrix = get_loc(self.view_program, "proj_matrix");
        self.uniforms.view.mv_matrix = get_loc(self.view_program, "mv_matrix");
        self.uniforms.view.full_shading = get_loc(self.view_program, "full_shading");
        self.uniforms.view.diffuse_albedo = get_loc(self.view_program, "diffuse_albedo");

        if self.display_program != 0 {
            unsafe { gl::DeleteProgram(self.display_program) };
        }

        self.display_program = program::link_from_shaders(
            &[
                shader::load("media/shaders/dof/display.vs.glsl", gl::VERTEX_SHADER, true),
                shader::load(
                    "media/shaders/dof/display.fs.glsl",
                    gl::FRAGMENT_SHADER,
                    true,
                ),
            ],
            true,
        );

        self.uniforms.dof.focal_distance = get_loc(self.display_program, "focal_distance");
        self.uniforms.dof.focal_depth = get_loc(self.display_program, "focal_depth");

        if self.filter_program != 0 {
            unsafe { gl::DeleteProgram(self.filter_program) };
        }

        self.filter_program = program::link_from_shaders(
            &[shader::load(
                "media/shaders/dof/gensat.cs.glsl",
                gl::COMPUTE_SHADER,
                true,
            )],
            true,
        );
    }

    fn new() -> Self {
        Self {
            focal_depth: 50.0,
            focal_distance: 40.0,
            ..Default::default()
        }
    }
}

fn main() {
    DofApp::new().run();
}
