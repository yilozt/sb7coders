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

use image::EncodableLayout;
use wasm_bindgen::{JsCast, prelude::Closure};
use crate::prelude::*;

#[derive(Default)]
struct Uniforms {
  mvp:    Option<WebGlUniformLocation>,
  offset: Option<WebGlUniformLocation>,
}


#[derive(Default)]
pub struct App {
  render_prog:    Option<WebGlProgram>,
  render_vao:     Option<WebGlVertexArrayObject>,
  uniforms:       Uniforms,
  tex_wall:       Option<WebGlTexture>,
  tex_ceiling:    Option<WebGlTexture>,
  tex_floor:      Option<WebGlTexture>,
  mipmap_enabled: bool,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig {
      title: "OpenGL SuperBible - Tunnel (Without mipmap)",
      ..AppConfig::default()
    }
  }

  fn startup(&mut self, gl: &gl) {
    let vs_src = "#version 300 es
      precision mediump float;
      out vec2 tc;

      uniform mat4 mvp;
      uniform float offset;


      void main(void) {
        const vec2[6] position = vec2[6](vec2(-0.5, -0.5),
                                         vec2( 0.5, -0.5),
                                         vec2(-0.5,  0.5),
                                         vec2( 0.5, -0.5),
                                         vec2(-0.5,  0.5),
                                         vec2( 0.5,  0.5));
        tc = (position[gl_VertexID].xy + vec2(offset, 0.5)) * vec2(30.0, 1.0);
        gl_Position = mvp * vec4(position[gl_VertexID], 0.0, 1.0);
      }
    ";

    let fs_src = "#version 300 es
      precision mediump float;
      out vec4 color;
      in vec2 tc;

      uniform sampler2D tex;

      void main(void) {
        color = texture(tex, tc * 2.0);
      }
    ";

    self.render_prog = program::link_from_shaders(gl, &[
      shader::load(gl, vs_src, gl::VERTEX_SHADER).as_ref(),
      shader::load(gl, fs_src, gl::FRAGMENT_SHADER).as_ref()
    ], true);

    self.uniforms.mvp    = gl.get_uniform_location(self.render_prog.as_ref().unwrap(), "mvp");
    self.uniforms.offset = gl.get_uniform_location(self.render_prog.as_ref().unwrap(), "offset");

    let load_img = | data | {
      #[inline(always)]
      fn init_tex(gl: &gl, fmt: u32, channel: u32, width: u32, height: u32, img: &[u8]) -> Option<WebGlTexture>{
        let tex = gl.create_texture();
        gl.bind_texture(gl::TEXTURE_2D, tex.as_ref());
        gl.tex_storage_2d(gl::TEXTURE_2D, 8, fmt, width as _, height as _);
        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(gl::TEXTURE_2D, 0, 0, 0, width as _, height as _, channel, gl::UNSIGNED_BYTE, img, 0).unwrap();
        gl.generate_mipmap(gl::TEXTURE_2D);
        tex
      }

      match image::load_from_memory(data).unwrap() {
        image::DynamicImage::ImageRgb8(img) => init_tex(gl, gl::RGB8, gl::RGB, img.width(), img.height(), img.as_bytes()),
        err @ _ => { log!("{:?}", err); unreachable!()}
      }
    };

    self.tex_wall = load_img(include_bytes!("assert/brick.jpg"));
    self.tex_ceiling = load_img(include_bytes!("assert/ceiling.jpg"));
    self.tex_floor = load_img(include_bytes!("assert/floor.jpg"));
  
    for tex in [&self.tex_wall, &self.tex_floor, &self.tex_ceiling] {
      gl.bind_texture(gl::TEXTURE_2D, tex.as_ref());
      gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
      gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
      gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as _);
    }

    self.render_vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.render_vao.as_ref());

    gl.enable(gl::DEPTH_TEST);
  }

  fn render(&self, gl: &gl, current_time: f64) {
    let t = current_time as f32;

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.clear_depth(1.0);
    gl.clear(gl::DEPTH_BUFFER_BIT);

    gl.use_program(self.render_prog.as_ref());

    gl.uniform1f(self.uniforms.offset.as_ref(), t * 0.003);

    let textures = [ &self.tex_wall, &self.tex_floor, &self.tex_wall, &self.tex_ceiling ];
    for (i, tex) in textures.iter().enumerate() {
      let mv_matrix = rotate_with_axis(90.0 * i as f32, 0.0, 0.0, 1.0)
                    * translate(-0.5, 0.0, -10.0)
                    * rotate_with_axis(90.0, 0.0, 1.0, 0.0)
                    * scale(30.0, 1.0, 1.0);

      let proj_mat = {
        let AppConfig { width, height, .. } = self.info();
        let aspect = width as f32 / height as f32;
        perspective(45.0, aspect, 0.1, 100.0)
      };
      let mvp = proj_mat * mv_matrix;

      gl.uniform_matrix4fv_with_f32_sequence(self.uniforms.mvp.as_ref(), false, &unsafe {js_sys::Float32Array::view_mut_raw(addr_of!(mvp) as _, 16)}.into());

      gl.bind_vertex_array(self.render_vao.as_ref());
      gl.bind_texture(gl::TEXTURE_2D, tex.as_ref());
      gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, if self.mipmap_enabled { gl::LINEAR_MIPMAP_LINEAR } else { gl::LINEAR } as _);
      gl.draw_arrays(gl::TRIANGLES, 0, 6);
    }
  }

  fn ui(&mut self, _gl: &web_sys::WebGl2RenderingContext, ui: &web_sys::Element) {
    ui.set_inner_html(r#"
    <label><input type="checkbox"/> Enable mipmap filter </label>
    "#);

    let checkbox: web_sys::HtmlInputElement = ui.query_selector(r#"input[type="checkbox"]"#).unwrap().unwrap().dyn_into().unwrap();
    checkbox.set_checked(self.mipmap_enabled);

    let closure = Closure::wrap(Box::new(|e: web_sys::Event| {
      let checkbox: web_sys::HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
      unsafe { super::ch5_7_0_tunnel_scintillation.mipmap_enabled = checkbox.checked(); }
    }) as Box<dyn FnMut(_)>);

    checkbox.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).unwrap();

    closure.forget();
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_program(self.render_prog.as_ref());
    gl.delete_vertex_array(self.render_vao.as_ref());
    for t in [&self.tex_wall, &self.tex_floor, &self.tex_ceiling] {
      gl.delete_texture(t.as_ref());
    }
  }
}
