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

use gl::*;
use sb7::application::*;
use std::{ffi::CString, ptr::addr_of};

#[derive(Default)]
struct Uniforms {
  mv_matrix:   i32,
  proj_matrix: i32,
}

#[derive(Default)]
struct App {
  tex_object:  [u32; 2],
  tex_index:   usize,
  render_prog: u32,
  uniforms:    Uniforms,
  object:      sb7::object::Object,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - Texture Coordinates".into(),
                ..Default::default() }
  }

  fn startup(&mut self) {
    macro_rules! tex_data {
      (@a W) => ([ 0xFF, 0xFF, 0xFF, 0xFFu8 ]);
      (@a B) => ([ 0x00, 0x00, 0x00, 0x00u8 ]);
      ($($x: ident),+ $(,)?) => ([$(tex_data!(@a $x),)*].concat());
    }

    let tex_data = tex_data! {
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                     B, W, B, W, B, W, B, W, B, W, B, W, B, W, B, W,
                     W, B, W, B, W, B, W, B, W, B, W, B, W, B, W, B,
                   };

    unsafe {
      GenTextures(1, &mut self.tex_object[0]);
      BindTexture(TEXTURE_2D, self.tex_object[0]);
      TexStorage2D(TEXTURE_2D, 1, RGB8, 16, 16);
      TexSubImage2D(TEXTURE_2D, 0, 0, 0, 16, 16, RGBA, UNSIGNED_BYTE, tex_data[..].as_ptr() as _);
      TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as _);
      TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as _);
    }

    self.tex_object[1] = sb7::ktx::file::load("media/textures/pattern1.ktx").unwrap()
                                                                            .0;

    self.object.load("media/objects/torus_nrms_tc.sbm");

    self.load_shaders();

    unsafe {
      Enable(DEPTH_TEST);
      DepthFunc(LEQUAL);
    }

    let AppConfig { width, height, .. } = AppConfig::default();
    self.on_resize(width as _, height as _);
  }

  fn render(&self, current_time: f64) {
    let grey = [0.2, 0.2, 0.2, 1.0f32].as_ptr();
    let ones = [1.0f32].as_ptr();

    unsafe {
      ClearBufferfv(COLOR, 0, grey);
      ClearBufferfv(DEPTH, 0, ones);

      BindTexture(TEXTURE_2D, self.tex_object[self.tex_index]);

      let mv_proj =
        sb7::vmath::translate(0.0, 0.0, -3.0)
        * sb7::vmath::rotate_with_axis(current_time as f32 * 19.3, 0.0, 1.0, 0.0)
        * sb7::vmath::rotate_with_axis(current_time as f32 * 21.1, 0.0, 0.0, 1.0);

      UniformMatrix4fv(self.uniforms.mv_matrix, 1, FALSE, addr_of!(mv_proj) as _);

      self.object.render();
    }
  }

  fn on_resize(&mut self, w: i32, h: i32) {
    let proj_matrix = sb7::vmath::perspective(60.0, w as f32 / h as f32, 0.1, 1000.0);
    unsafe {
      UniformMatrix4fv(self.uniforms.proj_matrix, 1, FALSE, addr_of!(proj_matrix) as _);
    }
  }

  fn shutdown(&mut self) {
    unsafe {
      DeleteTextures(2, self.tex_object.as_ptr());
      DeleteProgram(self.render_prog);
      self.object.free();
    }
  }

  fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
    if let glfw::Action::Press = press {
      match key {
        glfw::Key::R => self.load_shaders(),
        glfw::Key::T => {
          self.tex_index += 1;
          if self.tex_index > 1 {
            self.tex_index = 0;
          }
        }
        _ => {}
      }
    }
  }
}

fn main() {
  App::default().run();
}
impl App {
  fn load_shaders(&mut self) {
    if self.render_prog != 0 {
      unsafe { DeleteProgram(self.render_prog) };
    }

    self.render_prog = sb7::program::link_from_shaders(&[
      sb7::shader::load("media/shaders/simpletexcoords/render.vs.glsl", VERTEX_SHADER, true),
      sb7::shader::load("media/shaders/simpletexcoords/render.fs.glsl", FRAGMENT_SHADER, true)
    ], true);

    let location = |name: &str| unsafe {
      let name = CString::new(name).unwrap();
      GetUniformLocation(self.render_prog, name.as_ptr())
    };

    self.uniforms.mv_matrix = location("mv_matrix");
    self.uniforms.proj_matrix = location("proj_matrix");

    unsafe {
      UseProgram(self.render_prog);
    }
  }
}
