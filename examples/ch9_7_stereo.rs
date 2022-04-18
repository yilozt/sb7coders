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
struct StereoApp {
    view_program: GLuint,
    // show_light_depth_program: GLuint,
    uniforms: Uniforms,

    objects: [Object; 4],

    light_view_matrix: Mat4,
    light_proj_matrix: Mat4,

    camera_view_matrix: [Mat4; 2],
    camera_proj_matrix: Mat4,

    quad_vao: GLuint,

    separation: f32,

    mode: RenderMode,

    paused: bool,
}

enum RenderMode {
    Full,
    Light,
    Depth,
}
impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Full
    }
}

#[derive(Default)]
struct Object {
    obj: sb7::object::Object,
    model_matrix: Mat4,
}

#[derive(Default)]
struct Uniforms {
    // light: LightUniforms,
    view: ViewUniforms,
}

// #[derive(Default)]
// struct LightUniforms {
//     mvp: GLint,
// }

#[derive(Default)]
struct ViewUniforms {
    mv_matrix: GLint,
    proj_matrix: GLint,
    shadow_matrix: GLint,
    full_shading: GLint,
    specular_albedo: GLint,
    diffuse_albedo: GLint,
}

impl Application for StereoApp {
    fn init(&self) -> AppConfig {
        let mut conf = AppConfig::default();
        conf.title = "OpenGL SuperBible - Texture Coordinates".into();
        conf.flags.fullscreen = true;
        conf.flags.stereo = true;
        conf
    }

    fn startup(&mut self) {
        self.load_shaders();

        let object_names = [
            "media/objects/dragon.sbm",
            "media/objects/sphere.sbm",
            "media/objects/cube.sbm",
            "media/objects/torus.sbm",
        ];

        for (i, object_name) in object_names.iter().enumerate() {
            self.objects[i].obj.load(object_name);
        }

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::GenVertexArrays(1, &mut self.quad_vao);
            gl::BindVertexArray(self.quad_vao)
        }
    }

    fn render(&mut self, current_time: f64) {
        // let zeros = [0.0, 0.0, 0.0, 0.0f32].as_ptr();

        unsafe {
            static mut LAST_TIME: f64 = 0.0;
            static mut TOTAL_TIME: f64 = 0.0;

            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let info = self.info();
            let f = TOTAL_TIME as f32 + 30.0;

            let light_position = vec3!(20.0, 20.0, 20.0);
            let view_position = vec3!(0.0, 0.0, 40.0);

            self.light_proj_matrix = frustum(-1.0, 1.0, -1.0, 1.0, 1.0, 200.0f32);
            self.light_view_matrix =
                lookat(light_position, vec3!(0.0, 0.0, 0.0), vec3!(0.0, 1.0, 0.0));

            self.camera_proj_matrix =
                perspective(50.0, info.width as f32 / info.height as f32, 0.1, 200.0);

            self.camera_view_matrix[0] = lookat(
                view_position - vec3!(self.separation, 0.0, 0.0),
                vec3!(0.0, 0.0, -50.0),
                vec3!(0.0, 1.0, 0.0),
            );
            self.camera_view_matrix[1] = lookat(
                view_position + vec3!(self.separation, 0.0, 0.0),
                vec3!(0.0, 0.0, -50.0),
                vec3!(0.0, 1.0, 0.0),
            );

            self.objects[0].model_matrix = rotate_with_axis(f * 14.5, 0.0, 1.0, 0.0)
                * rotate_with_axis(20.0, 1.0, 0.0, 0.0)
                * translate(0.0, -4.0, 0.0);

            self.objects[1].model_matrix = rotate_with_axis(f * 3.7, 0.0, 1.0, 0.0)
                * translate((f * 0.37).sin() * 12.0, (f * 0.37).cos() * 12.0, 0.0)
                * scale(2.0, 2.0, 2.0);

            self.objects[2].model_matrix = rotate_with_axis(f * 6.45, 0.0, 1.0, 0.0)
                * translate((f * 0.25).sin() * 10.0, (f * 0.25).cos() * 10.0, 0.0)
                * rotate_with_axis(f * 99.0, 0.0, 0.0, 1.0)
                * scale(2.0, 2.0, 2.0);

            self.objects[3].model_matrix = rotate_with_axis(f * 5.25, 0.0, 1.0, 0.0)
                * translate((f * 0.51).sin() * 14.0, (f * 0.51).cos() * 14.0, 0.0)
                * rotate_with_axis(f * 120.3, 0.707106, 0.0, 0.707106)
                * scale(2.0, 2.0, 2.0);

            gl::Enable(gl::DEPTH_TEST);

            self.render_scene(TOTAL_TIME);
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press = press {
            match key {
                glfw::Key::Kp1 => self.mode = RenderMode::Full,
                glfw::Key::Kp2 => self.mode = RenderMode::Light,
                glfw::Key::Kp3 => self.mode = RenderMode::Depth,
                glfw::Key::R => self.load_shaders(),
                glfw::Key::P => self.paused = !self.paused,
                _ => {}
            }
        }
        if let glfw::Action::Repeat = press {
            match key {
                glfw::Key::Z => self.separation += 0.05,
                glfw::Key::X => self.separation -= 0.05,
                _ => {}
            }
        }
    }
}

impl StereoApp {
    fn load_shaders(&mut self) {
        if self.view_program != 0 {
            unsafe { gl::DeleteProgram(self.view_program) };
        }

        self.view_program = program::link_from_shaders(
            &[
                shader::load(
                    "media/shaders/stereo/stereo-render.vs.glsl",
                    VERTEX_SHADER,
                    true,
                ),
                shader::load(
                    "media/shaders/stereo/stereo-render.fs.glsl",
                    FRAGMENT_SHADER,
                    true,
                ),
            ],
            true,
        );

        fn get_loc(program: GLuint, name: &str) -> GLint {
            let name = std::ffi::CString::new(name).unwrap();
            unsafe { gl::GetUniformLocation(program, name.as_ptr()) }
        }

        self.uniforms.view.proj_matrix = get_loc(self.view_program, "proj_matrix");
        self.uniforms.view.mv_matrix = get_loc(self.view_program, "mv_matrix");
        self.uniforms.view.shadow_matrix = get_loc(self.view_program, "shadow_matrix");
        self.uniforms.view.full_shading = get_loc(self.view_program, "full_shading");
        self.uniforms.view.specular_albedo = get_loc(self.view_program, "specular_albedo");
        self.uniforms.view.diffuse_albedo = get_loc(self.view_program, "diffuse_albedo");
    }

    fn render_scene(&mut self, _current_time: f64) {
        let ones = [1.0f32].as_ptr();
        // let zero = [0.0f32];
        let gray = [0.1, 0.1, 0.1, 0.0f32].as_ptr();
        let info = self.info();

        let scale_basic_matrix = Mat4::from_vec([
            vec4!(0.5, 0.0, 0.0, 0.0),
            vec4!(0.0, 0.5, 0.0, 0.0),
            vec4!(0.0, 0.0, 0.5, 0.0),
            vec4!(0.5, 0.5, 0.5, 1.0),
        ]);

        // let light_vp_matrix = self.light_proj_matrix * self.light_proj_matrix;
        let shadow_sbpv_matrix =
            scale_basic_matrix * self.light_proj_matrix * self.light_view_matrix;

        unsafe {
            gl::Viewport(0, 0, info.width as GLsizei, info.height as GLsizei);
            gl::ClearBufferfv(gl::COLOR, 0, gray);
            gl::UseProgram(self.view_program);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::UniformMatrix4fv(
                self.uniforms.view.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(self.camera_proj_matrix) as *const GLfloat,
            );
            gl::DrawBuffer(gl::BACK);
        }

        let diffuse_colors = [
            vec3!(1.0, 0.6, 0.3f32),
            vec3!(0.2, 0.8, 0.9f32),
            vec3!(0.3, 0.9, 0.4f32),
            vec3!(0.5, 0.2, 1.0f32),
        ];

        unsafe {
            for j in 0..2 {
                let buffs = [gl::BACK_LEFT, gl::BACK_RIGHT];
                gl::DrawBuffer(buffs[j]);
                gl::ClearBufferfv(gl::COLOR, 0, gray);
                gl::ClearBufferfv(gl::DEPTH, 0, ones);
                for i in 0..4 {
                    let model_matrix = self.objects[i].model_matrix;
                    let shadow_matrix = shadow_sbpv_matrix * model_matrix;
                    gl::UniformMatrix4fv(
                        self.uniforms.view.shadow_matrix,
                        1,
                        gl::FALSE,
                        addr_of!(shadow_matrix) as *const GLfloat,
                    );
                    gl::UniformMatrix4fv(self.uniforms.view.mv_matrix, 1, gl::FALSE, {
                        let mat = self.camera_view_matrix[j] * self.objects[i].model_matrix;
                        addr_of!(mat) as *const GLfloat
                    });
                    gl::Uniform1i(
                        self.uniforms.view.full_shading,
                        match self.mode {
                            RenderMode::Full => 1,
                            _ => 0,
                        },
                    );
                    gl::Uniform3fv(
                        self.uniforms.view.diffuse_albedo,
                        1,
                        addr_of!(diffuse_colors[i]) as *const GLfloat,
                    );
                    self.objects[i].obj.render();
                }
            }
        }
    }

    fn new() -> Self {
        Self {
            separation: 2.0,
            ..Default::default()
        }
    }
}

fn main() {
    StereoApp::new().run()
}
