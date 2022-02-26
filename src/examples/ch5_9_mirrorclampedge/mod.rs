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

// gl.MIRROR_CLAMP_TO_EDGE and gl.CLAMP_TO_BORDER are NOT supported in WebGL2
// So this demo will use special texture to simulate it.

use image::EncodableLayout;
use wasm_bindgen::{JsCast, prelude::Closure};

use crate::prelude::*;

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
pub struct App {
  render_prog:  Option<WebGlProgram>,
  tex_border:   Option<WebGlTexture>,
  tex_mirror:   Option<WebGlTexture>,
  vao:          Option<WebGlVertexArrayObject>,
  is_mirror:    Option<WebGlUniformLocation>,
  display_mode: DisplayMode,
}

impl App {
  fn load_shaders(&mut self, gl: &gl) {
    gl.delete_program(self.render_prog.as_ref());

    self.render_prog = program::link_from_shaders(gl, &[
      shader::load(gl, include_str!("../../../media/shaders/mirrorclampedge/drawquad.vs.glsl"), gl::VERTEX_SHADER).as_ref(),
      shader::load(gl, include_str!("../../../media/shaders/mirrorclampedge/drawquad.fs.glsl"), gl::FRAGMENT_SHADER).as_ref()
    ], true);

    self.is_mirror = gl.get_uniform_location(self.render_prog.as_ref().unwrap(), "is_mirror");
  }
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - GL_MIRROR_CLAMP_TO_EDGE".into(),
                ..Default::default() }
  }

  fn startup(&mut self, gl: &gl) {
    // "media/textures/brick.ktx" has broken:
    // - https://github.com/openglsuperbible/sb7code/issues/44
    if let Ok(image::DynamicImage::ImageRgba8(img)) = image::load_from_memory(include_bytes!("assert/brickmirror.png")) {
      self.tex_mirror = gl.create_texture();
      gl.bind_texture(gl::TEXTURE_2D, self.tex_mirror.as_ref());
      gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, img.width() as _, img.height() as _);
      gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(gl::TEXTURE_2D, 0, 0, 0, img.width() as _, img.height() as _, gl::RGBA, gl::UNSIGNED_BYTE, Some(img.as_bytes())).unwrap();
    }
    if let Ok(image::DynamicImage::ImageRgba8(img)) = image::load_from_memory(include_bytes!("assert/brickborder.png")) {
      self.tex_border = gl.create_texture();
      gl.bind_texture(gl::TEXTURE_2D, self.tex_border.as_ref());
      gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, img.width() as _, img.height() as _);
      gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(gl::TEXTURE_2D, 0, 0, 0, img.width() as _, img.height() as _, gl::RGBA, gl::UNSIGNED_BYTE, Some(img.as_bytes())).unwrap();
    }

    self.load_shaders(gl);

    self.vao = gl.create_vertex_array();
  }

  fn render(&self, gl: &gl, _current_time: f64) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);

    gl.bind_vertex_array(self.vao.as_ref());
    gl.use_program(self.render_prog.as_ref());

    match self.display_mode {
      DisplayMode::MirrorClampToEdge => {
        gl.bind_texture(gl::TEXTURE_2D, self.tex_mirror.as_ref());
        gl.uniform1i(self.is_mirror.as_ref(), 1);
      },
      DisplayMode::ClampToBorder => {
        gl.bind_texture(gl::TEXTURE_2D, self.tex_border.as_ref());
        gl.uniform1i(self.is_mirror.as_ref(), 0);
      },
    }

    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);

    gl.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
  }

  fn ui(&mut self, _gl: &web_sys::WebGl2RenderingContext, ui: &web_sys::Element) {
    ui.set_inner_html(&format!(r#"
      <label><input name="mir_settings" type="radio" value="0"/>GL_MIRROR_CLAMP_TO_EDGE</label>
      <label><input name="mir_settings" type="radio" value="1"/>GL_CLAMP_TO_BORDER</label>
    "#));

    let radios = ui.query_selector_all("input[type=radio]").unwrap();
    match self.display_mode {
      DisplayMode::ClampToBorder => {
        let r:web_sys::HtmlInputElement = radios.get(1).unwrap().dyn_into().unwrap();
        r.set_checked(true);
      },
      DisplayMode::MirrorClampToEdge => {
        let r:web_sys::HtmlInputElement = radios.get(0).unwrap().dyn_into().unwrap();
        r.set_checked(true);
      },
    }

    let closure = Closure::wrap(Box::new(|e: web_sys::Event| {
      let r: web_sys::HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
      match r.value().as_str() {
        "0" => unsafe { super::ch5_9_mirrorclampedge.display_mode = DisplayMode::MirrorClampToEdge},
        "1" => unsafe { super::ch5_9_mirrorclampedge.display_mode = DisplayMode::ClampToBorder},
        _ => unreachable!()
      }
    }) as Box<dyn FnMut(_)>);
    for i in 0..radios.length() {
      radios.get(i).unwrap().add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).unwrap();
    }
    closure.forget();
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_vertex_array(self.vao.as_ref());
    gl.delete_program(self.render_prog.as_ref());
    gl.delete_texture(self.tex_border.as_ref());
    gl.delete_texture(self.tex_mirror.as_ref());
  }

  // fn ui(&mut self, ui: &imgui::Ui) {
  //   todo!()
  // }
}
