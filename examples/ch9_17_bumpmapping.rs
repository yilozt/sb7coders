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

use std::io::Write;

use sb7::prelude::*;

#[derive(Default)]
struct BumpMappingApp {
    program: GLuint,
    textures: Textures,
    uniforms: Uniforms,
    object: Object,
    paused: bool,
}

#[derive(Default)]
struct Uniforms {
    mv_matrix: GLint,
    proj_matrix: GLint,
    light_pos: GLint,
}

#[derive(Default)]
struct Textures {
    color: GLuint,
    normals: GLuint,
}

impl Application for BumpMappingApp {
    fn init(&self) -> AppConfig {
        AppConfig {
            title: "OpenGL SuperBible - Bump Mapping".into(),
            ..Default::default()
        }
    }

    fn startup(&mut self) {
        self.load_shaders();

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            self.textures.color = ktx::file::load("media/textures/ladybug_co.ktx").unwrap().0;
            gl::ActiveTexture(gl::TEXTURE1);
            self.textures.normals = ktx::file::load("media/textures/ladybug_nm.ktx").unwrap().0;
        }

        self.object.load("media/objects/ladybug.sbm");
    }

    fn render(&mut self, current_time: f64) {
        let gray = [0.1, 0.1, 0.1, 0.1].as_ptr();
        let ones = [1.0f32].as_ptr();

        unsafe {
            static mut LAST_TIME: f64 = 0.0;
            static mut TOTAL_TIME: f64 = 0.0;
            if !self.paused {
                TOTAL_TIME += current_time - LAST_TIME;
            }
            LAST_TIME = current_time;

            let f = TOTAL_TIME as f32;

            gl::ClearBufferfv(gl::COLOR, 0, gray);
            gl::ClearBufferfv(gl::DEPTH, 0, ones);

            gl::Enable(gl::DEPTH_TEST);

            gl::UseProgram(self.program);

            let info = self.info();
            let proj_matrix =
                perspective(50.0, info.width as f32 / info.height as f32, 0.1, 1000.0);
            gl::UniformMatrix4fv(
                self.uniforms.proj_matrix,
                1,
                gl::FALSE,
                addr_of!(proj_matrix) as *const GLfloat,
            );

            let mv_matrix = lookat(
                vec3!(f.sin() * 8.0, 4.0, f.cos() * 8.0),
                vec3!(0.0, 0.0, 0.0),
                vec3!(0.0, 1.0, 0.0),
            );
            gl::UniformMatrix4fv(
                self.uniforms.mv_matrix,
                1,
                gl::FALSE,
                addr_of!(mv_matrix) as *const GLfloat,
            );

            let light_pos = vec3!(f.sin() * 20.0, 20.0 + f.cos() * 20.0, 20.0);
            gl::Uniform3fv(
                self.uniforms.light_pos,
                1,
                addr_of!(light_pos) as *const GLfloat,
            );
            self.object.render();
        }
    }

    fn shutdown(&mut self) {
        self.object.free();
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteTextures(1, &self.textures.color);
            gl::DeleteTextures(1, &self.textures.normals);
        }
    }

    fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
        if let glfw::Action::Press = press {
            match key {
                glfw::Key::R => self.load_shaders(),
                glfw::Key::S => self.make_screenshort(),
                glfw::Key::P => self.paused = !self.paused,
                _ => {}
            }
        }
    }
}

impl BumpMappingApp {
    fn load_shaders(&mut self) {
        let vs = shader::load(
            "media/shaders/bumpmapping/bumpmapping.vs.glsl",
            gl::VERTEX_SHADER,
            true,
        );
        let fs = shader::load(
            "media/shaders/bumpmapping/bumpmapping.fs.glsl",
            gl::FRAGMENT_SHADER,
            true,
        );
        if self.program != 0 {
            unsafe { gl::DeleteProgram(self.program) };
        }
        self.program = program::link_from_shaders(&[vs, fs], true);

        fn get_location(prog: GLuint, name: &str) -> GLint {
            let name = std::ffi::CString::new(name).unwrap();
            unsafe { gl::GetUniformLocation(prog, name.as_ptr()) }
        }

        self.uniforms.mv_matrix = get_location(self.program, "mv_matrix");
        self.uniforms.proj_matrix = get_location(self.program, "proj_matrix");
        self.uniforms.light_pos = get_location(self.program, "light_pos");
    }

    fn make_screenshort(&self) {
        let info = self.info();
        let data_size = info.width * info.height * 3;
        let mut data = Box::new(vec![0u8; data_size]);

        #[repr(packed)]
        #[derive(Default)]
        pub struct TgaHeader {
            pub identsize: u8,  // Size of following ID field
            pub cmaptype: u8,   // Color map type 0 = none
            pub imagetype: u8,  // Image type 2 = rgb
            pub cmapstart: u16, // First entry in palette
            pub cmapsize: u16,  // Number of entries in palette
            pub cmapbpp: u8,    // Number of bits per palette entry
            pub xorigin: u16,   // X origin
            pub yorigin: u16,   // Y origin
            pub width: u16,     // Width in pixels
            pub height: u16,    // Height in pixels
            pub bpp: u8,        // Bits per pixel
            pub descriptor: u8, // Descriptor bits
        }

        unsafe {
            gl::ReadPixels(
                0, // Origin
                0,
                info.width as GLsizei, // Size
                info.height as GLsizei,
                gl::BGR,                                    // Format
                gl::UNSIGNED_BYTE,                          // Type
                data.as_mut_ptr() as *mut std::ffi::c_void, // Data
            );
        }

        let tga_header = unsafe {
            std::slice::from_raw_parts(
                &TgaHeader {
                    imagetype: 2,
                    width: info.width as u16,
                    height: info.height as u16,
                    bpp: 24,
                    ..Default::default()
                } as *const TgaHeader as *const u8,
                size_of::<TgaHeader>(),
            )
        };

        let mut f_out = std::fs::File::create("screenshort.tga").unwrap();
        f_out.write(tga_header).unwrap();
        f_out.write(&data).unwrap();
    }
}

fn main() {
    BumpMappingApp::default().run();
}
