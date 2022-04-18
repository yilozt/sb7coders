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
struct DismapApp {
    program: GLuint,
    vao: GLuint,
    tex_displacement: GLuint,
    tex_color: GLuint,
    dmap_depth: f32,
    enable_displacement: bool,
    wireframe: bool,
    enable_fog: bool,
    paused: bool,
    uniforms: Uniforms,
}

#[derive(Default)]
struct Uniforms {
    mvp_matrix: GLint,
    mv_matrix: GLint,
    proj_matrix: GLint,
    dmap_depth: GLint,
    enable_fog: GLint,
}

impl DismapApp {
    fn new() -> Self {
        Self {
            enable_displacement: true,
            enable_fog: true,
            ..Default::default()
        }
    }

    fn load_shaders(&mut self) {
        unsafe {
            if self.program != 0 {
                gl::DeleteProgram(self.program);
            }

            let vs = shader::load(
                "media/shaders/dispmap/dispmap.vs.glsl",
                gl::VERTEX_SHADER,
                true,
            );
            let tcs = shader::load(
                "media/shaders/dispmap/dispmap.tcs.glsl",
                gl::TESS_CONTROL_SHADER,
                true,
            );
            let tes = shader::load(
                "media/shaders/dispmap/dispmap.tes.glsl",
                gl::TESS_EVALUATION_SHADER,
                true,
            );
            let fs = shader::load(
                "media/shaders/dispmap/dispmap.fs.glsl",
                gl::FRAGMENT_SHADER,
                true,
            );

            self.program = gl::CreateProgram();
            gl::AttachShader(self.program, vs);
            gl::AttachShader(self.program, tcs);
            gl::AttachShader(self.program, tes);
            gl::AttachShader(self.program, fs);

            gl::LinkProgram(self.program);

            let get_loc = |name| {
                let name = std::ffi::CString::new(name).unwrap();
                gl::GetUniformLocation(self.program, name.as_ptr())
            };

            self.uniforms.mv_matrix = get_loc("mv_matrix");
            self.uniforms.proj_matrix = get_loc("proj_matrix");
            self.uniforms.mvp_matrix = get_loc("mvp_matrix");
            self.uniforms.dmap_depth = get_loc("dmap_depth");
            self.uniforms.enable_fog = get_loc("enable_fog");
            self.dmap_depth = 6.0;
        }
    }
}

impl Application for DismapApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Displacement Mapping".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        self.load_shaders();

        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            gl::PatchParameteri(gl::PATCH_VERTICES, 4);

            gl::Enable(gl::CULL_FACE);

            self.tex_displacement = ktx::file::load("media/textures/terragen1.ktx").unwrap().0;
            gl::ActiveTexture(gl::TEXTURE1);
            self.tex_color = ktx::file::load("media/textures/terragen_color.ktx")
                .unwrap()
                .0;
        }
    }

    fn render(&mut self, current_time: f64) {
        let black = [0.85, 0.95, 1.0, 1.0f32].as_ptr();
        let one = 1.0f32;
        unsafe {
            static mut LAST_TIME: f64 = 0.0;
            static mut TOTAL_TIME: f64 = 0.0;

            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let t = (TOTAL_TIME * 0.03) as f32;
            let r = ((t * 5.37).sin() * 15.0 + 16.0) as f32;
            let h = ((t * 4.79).cos() * 2.0 + 3.2) as f32;

            gl::ClearBufferfv(gl::COLOR, 0, black);
            gl::ClearBufferfv(gl::DEPTH, 0, &one);

            let mv_matrix = lookat(
                vec3!(t.sin() * r, h, t.cos() * r),
                vec3!(0.0, 0.0, 0.0),
                vec3!(0.0, 1.0, 0.0),
            );
            let proj_matrix = perspective(
                60.0,
                {
                    let AppConfig { width, height, .. } = self.info();
                    width as f32 / height as f32
                },
                0.1,
                1000.0,
            );
            let mvp_matrix = proj_matrix * mv_matrix;

            gl::UseProgram(self.program);

            gl::UniformMatrix4fv(
                self.uniforms.mv_matrix,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const f32,
            );
            gl::UniformMatrix4fv(
                self.uniforms.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const f32,
            );
            gl::UniformMatrix4fv(
                self.uniforms.mvp_matrix,
                1,
                gl::FALSE,
                addr_of!(mvp_matrix) as *const f32,
            );
            gl::Uniform1f(
                self.uniforms.dmap_depth,
                match self.enable_displacement {
                    true => self.dmap_depth,
                    false => 0.0,
                },
            );
            gl::Uniform1i(
                self.uniforms.enable_fog,
                match self.enable_fog {
                    true => 1,
                    false => 0,
                },
            );

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);

            match self.wireframe {
                true => gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE),
                false => gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL),
            }

            gl::DrawArraysInstanced(gl::PATCHES, 0, 4, 64 * 64);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.program);
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Repeat = press {
            match key {
                glfw::Key::KpAdd => self.dmap_depth += 0.1,
                glfw::Key::KpSubtract => self.dmap_depth -= 0.1,
                _ => {}
            }
        }
        if let glfw::Action::Press = press {
            match key {
                glfw::Key::KpAdd => self.dmap_depth += 0.1,
                glfw::Key::KpSubtract => self.dmap_depth -= 0.1,
                glfw::Key::F => self.enable_fog = !self.enable_fog,
                glfw::Key::D => self.enable_displacement = !self.enable_displacement,
                glfw::Key::W => self.wireframe = !self.wireframe,
                glfw::Key::P => self.paused = !self.paused,
                glfw::Key::R => self.load_shaders(),
                _ => {}
            }
        }
    }

    fn ui(&mut self, ui: &imgui::Ui) {
        if let Some(win) = imgui::Window::new("Help")
            .position([10.0, 10.0], imgui::Condition::Once)
            .begin(ui)
        {
            ui.text(format!(
                "(NumAdd / NumSubtract) dmap_depth: {:.2}",
                self.dmap_depth
            ));
            ui.text(format!("(F) enable_fog: {}", self.enable_fog));
            ui.text(format!(
                "(D) enable_displacement: {}",
                self.enable_displacement
            ));
            ui.text(format!("(W) wireframe: {}", self.wireframe));
            ui.text(format!("(P) paused: {}", self.paused));

            win.end();
        }
    }
}

fn main() {
    DismapApp::new().run()
}
