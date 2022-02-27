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

use std::{rc::Rc, cell::Cell};

use image::EncodableLayout;
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::prelude::*;

#[derive(Default)]
#[derive(Clone)]
struct Uniforms {
  mv_matrix:   Option<WebGlUniformLocation>,
  proj_matrix: Option<WebGlUniformLocation>,
}

#[derive(Default)]
#[derive(Clone)]
pub struct App {
  tex_object:  [Option<WebGlTexture>; 2],
  tex_index:   Rc<Cell<usize>>,
  render_prog: Option<WebGlProgram>,
  uniforms:    Uniforms,
  object:      Object,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - Texture Coordinates".into(),
                ..Default::default() }
  }

  fn startup(&mut self, gl: &gl) {
    macro_rules! tex_data {
      (@a W) => ([ 0xFF, 0xFF, 0xFF, 0xFFu8 ]);
      (@a B) => ([ 0x00, 0x00, 0x00, 0xFFu8 ]);
      ($($x: ident),+ $(,)?) => ([$(tex_data!(@a $x),)*].concat());
    }

    let tex_data = tex_data! {
      B, W, B, W, B, W, B, W,
      W, B, W, B, W, B, W, B,
      B, W, B, W, B, W, B, W,
      W, B, W, B, W, B, W, B,
      B, W, B, W, B, W, B, W,
      W, B, W, B, W, B, W, B,
      B, W, B, W, B, W, B, W,
      W, B, W, B, W, B, W, B,
    };

    self.tex_object[0] = gl.create_texture();
    gl.bind_texture(gl::TEXTURE_2D, self.tex_object[0].as_ref());

    gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, 8, 8);


    let view = unsafe {js_sys::Uint8Array::view(tex_data.as_slice()) };
    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(gl::TEXTURE_2D, 0, 0, 0, 8, 8, gl::RGBA, gl::UNSIGNED_BYTE, Some(&view)).unwrap();

    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);


    // - "../../media/textures/pattern1.ktx" can't load correctly with WebGL2:
    //
    // self.tex_object[1] =
    //   ktx::file::load(gl, include_bytes!("../../media/textures/pattern1.ktx")).unwrap()
    //                                                                           .0;

    match image::load_from_memory(include_bytes!("assert/pattern.jpg")) {
      Ok(image::DynamicImage::ImageRgb8(img)) =>  {
        self.tex_object[1] = gl.create_texture();
        gl.bind_texture(gl::TEXTURE_2D, self.tex_object[1].as_ref());
        gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGB8, img.width() as _, img.height() as _);
        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(gl::TEXTURE_2D, 0, 0, 0, img.width() as _, img.height() as _, gl::RGB, gl::UNSIGNED_BYTE, Some(img.as_bytes())).unwrap();
  
        gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
      },
      other @ _ => { log!("{:?}", other) }
    }

    self.object
        .load(gl, include_bytes!("../../../media/objects/torus_nrms_tc.sbm"));

    self.load_shaders(gl);

    gl.enable(gl::DEPTH_TEST);
    gl.depth_func(gl::LEQUAL);

    let AppConfig { width, height, .. } = self.info();
    self.on_resize(gl, width as _, height as _);
  }

  fn render(&self, gl: &gl, current_time: f64) {
    gl.use_program(self.render_prog.as_ref());

    gl.clear_color(0.2, 0.2, 0.2, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.clear_depth(1.0);
    gl.clear(gl::DEPTH_BUFFER_BIT);

    gl.bind_texture(gl::TEXTURE_2D, self.tex_object[self.tex_index.get()].as_ref());

    let mv_proj = translate(0.0, 0.0, -3.0)
                  * rotate_with_axis(current_time as f32 * 19.3, 0.0, 1.0, 0.0)
                  * rotate_with_axis(current_time as f32 * 21.1, 0.0, 0.0, 1.0);

    gl.uniform_matrix4fv_with_f32_sequence(self.uniforms.mv_matrix.as_ref(),
                                           false,
                                           &unsafe {js_sys::Float32Array::view_mut_raw(addr_of!(mv_proj) as _, 16).into()});

    self.object.render(gl);
  }

  fn ui(&mut self, _gl: Rc<web_sys::WebGl2RenderingContext>, ui: &web_sys::Element) {
    ui.set_inner_html(&format!(r#"
      <label><input type="radio" name="tex" value="0"/>Chequer</label>
      <label><input type="radio" name="tex" value="1"/>Pattern</label>
    "#));

    let radios = ui.query_selector_all("input[type=radio]").unwrap();

    {
      let r: web_sys::HtmlInputElement = radios.get(self.tex_index.get() as u32).unwrap().dyn_into().unwrap();
      r.set_checked(true);
    }

    for i in 0..radios.length() {
      let radio: web_sys::HtmlInputElement = radios.get(i).unwrap().dyn_into().unwrap();
      let tex_index = self.tex_index.clone();
      let  closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        let radio: web_sys::HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
        match radio.value().as_str() {
          "0" => tex_index.set(0),
          "1" => tex_index.set(1),
          _ => unreachable!(),
        };
      }) as Box<dyn FnMut(_)>);
      radio.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).unwrap();
      closure.forget();
    }
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_texture(self.tex_object[0].as_ref());
    gl.delete_texture(self.tex_object[1].as_ref());
    gl.delete_program(self.render_prog.as_ref());
    self.object.free(gl);
  }
}

impl App {
  fn load_shaders(&mut self, gl: &gl) {
    if self.render_prog.is_some() {
      gl.delete_program(self.render_prog.as_ref())
    }

    self.render_prog = program::link_from_shaders(gl, &[
      shader::load(gl, include_str!("../../../media/shaders/simpletexcoords/render.vs.glsl"), gl::VERTEX_SHADER).as_ref(),
      shader::load(gl, include_str!("../../../media/shaders/simpletexcoords/render.fs.glsl"), gl::FRAGMENT_SHADER).as_ref()
    ], true);

    let location =
      |name: &str| gl.get_uniform_location(self.render_prog.as_ref().unwrap(), name);

    self.uniforms.mv_matrix = location("mv_matrix");
    self.uniforms.proj_matrix = location("proj_matrix");

    gl.use_program(self.render_prog.as_ref());

    self.on_resize(gl, self.info().width as _, self.info().height as _);
  }

  fn on_resize(&self, gl: &gl, width: i32, height: i32) {
    let proj_matrix = perspective(50.0, width as f32 / height as f32, 0.01, 1000.0);
    gl.uniform_matrix4fv_with_f32_sequence(self.uniforms.proj_matrix.as_ref(), false, &unsafe { js_sys::Float32Array::view_mut_raw(addr_of!(proj_matrix) as _, 16) }.into());
  }
}