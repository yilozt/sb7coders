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

enum DisplayMode {
  ClampToBorder,
  MirrorClampToEdge,
}

impl Default for DisplayMode {
  #[inline(always)]
  fn default() -> Self {
    Self::MirrorClampToEdge
  }
}

#[derive(Default)]
struct App {
  render_prog:  u32,
  tex:          u32,
  vao:          u32,
  display_mode: DisplayMode,
}

impl App {
  fn load_shaders(&mut self) {
    gl!(DeleteShader(self.render_prog));

    self.render_prog = program::link_from_shaders(&[
      shader::load("media/shaders/mirrorclampedge/drawquad.vs.glsl", VERTEX_SHADER, true),
      shader::load("media/shaders/mirrorclampedge/drawquad.fs.glsl", FRAGMENT_SHADER, true)
    ], true);
  }
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - GL_MIRROR_CLAMP_TO_EDGE".into(),
                ..Default::default() }
  }

  fn startup(&mut self) {
    // "media/textures/brick.ktx" has broken:
    // - https://github.com/openglsuperbible/sb7code/issues/44
    self.tex = ktx::file::load("media/textures/brick.ktx").unwrap().0;

    self.load_shaders();

    gl!(GenVertexArrays(1, &mut self.vao));
  }

  fn render(&mut self, _current_time: f64) {
    gl! {
      ClearBufferfv(COLOR, 0, color::Black.as_ptr());

      BindTexture(TEXTURE_2D, self.tex);

      BindVertexArray(self.vao);
      UseProgram(self.render_prog);

      match self.display_mode {
        DisplayMode::MirrorClampToEdge => {
          TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, MIRROR_CLAMP_TO_EDGE as _);
          TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, MIRROR_CLAMP_TO_EDGE as _);
        },
        DisplayMode::ClampToBorder => {
          TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_BORDER as _);
          TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_BORDER as _);
        }
      }

      DrawArrays(TRIANGLE_STRIP, 0, 4);
    }
  }

  fn shutdown(&mut self) {
    gl! {
      DeleteVertexArrays(1, &self.vao);
      DeleteProgram(self.render_prog);
      DeleteTextures(1, &self.tex);
    }
  }

  fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
    if let glfw::Action::Press = press {
      match key {
        glfw::Key::M => {
          self.display_mode = match self.display_mode {
            DisplayMode::ClampToBorder => DisplayMode::MirrorClampToEdge,
            DisplayMode::MirrorClampToEdge => DisplayMode::ClampToBorder,
          }
        }
        glfw::Key::R => self.load_shaders(),
        _ => {}
      }
    }
  }

  fn ui(&mut self, ui: &imgui::Ui) {
    let win = imgui::Window::new("Press M to toggle wrap mode")
      .position([10.0, 10.0], imgui::Condition::Appearing);
    
      if let Some(w) = win.begin(ui) {
      ui.text(format!("current: {}", match self.display_mode {
        DisplayMode::ClampToBorder => "GL_CLAMP_TO_BORDER",
        DisplayMode::MirrorClampToEdge => "GL_MIRROR_CLAMP_TO_EDGE",
      }));

      w.end();
    }
  }
}

fn main() {
  App::default().run();
}
