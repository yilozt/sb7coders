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

use crate::prelude::*;

#[derive(Default)]
pub struct App {
  vao:            Option<WebGlVertexArrayObject>,
  tex:            Option<WebGlTexture>,
  texborder:      Option<WebGlTexture>,
  uniform_offset: Option<WebGlUniformLocation>,
  uniform_trans:  Option<WebGlUniformLocation>,
  prog:           Option<WebGlProgram>,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig {
      title: "OpenGL SuperBible - Texture Wrap Modes".into(),
      ..Default::default()
    }
  }

  fn startup(&mut self, gl: &gl) {
    let vs_src = "#version 300 es
      precision mediump float;

      uniform vec2 offset;
      uniform mat4 trans;

      out vec2 tex_coord;

      void main(void) {
        const vec4 vertices[] = vec4[](vec4(-0.45, -0.35, 0.5, 1.0),
                                       vec4( 0.45, -0.35, 0.5, 1.0),
                                       vec4(-0.45,  0.35, 0.5, 1.0),
                                       vec4( 0.45,  0.35, 0.5, 1.0));
        gl_Position = trans *  (vertices[gl_VertexID] + vec4(offset * vec2(1.0, 0.85), 0.0, 0.0)); 
        tex_coord = vertices[gl_VertexID].xy * 3.0 + vec2(0.45 * 3.0);
      }
    ";
    
    let fs_src = "#version 300 es
      precision mediump float;

      uniform sampler2D s;
      out vec4 color;

      in vec2 tex_coord;

      void main(void) {
        color = texture(s, tex_coord);
      }
    ";

    let vs = shader::load(gl, vs_src, gl::VERTEX_SHADER);
    let fs = shader::load(gl, fs_src, gl::FRAGMENT_SHADER);
    self.prog = program::link_from_shaders(gl, &[vs.as_ref(), fs.as_ref()], true);

    // self.tex = ktx::file::load(gl, include_bytes!("../../../media/textures/rightarrows.ktx")).unwrap().0;
    match image::load_from_memory(include_bytes!("assert/rightarrow.jpg")) {
      Ok(image::DynamicImage::ImageRgb8(img)) => {
        self.tex = gl.create_texture();
        gl.bind_texture(gl::TEXTURE_2D, self.tex.as_ref());
        gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGB8, img.width() as _, img.height() as _);
        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(gl::TEXTURE_2D, 0, 0, 0, img.width() as _, img.height() as _, gl::RGB, gl::UNSIGNED_BYTE, img.as_bytes(), 0).unwrap();
        log!("[loaded]");
      },
      _ => log!("unhandled format"),
    }

    match image::load_from_memory(include_bytes!("assert/rightarrowborder.png")) {
      Ok(image::DynamicImage::ImageRgba8(img)) => {
        self.texborder = gl.create_texture();
        gl.bind_texture(gl::TEXTURE_2D, self.texborder.as_ref());
        gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, img.width() as _, img.height() as _);
        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(gl::TEXTURE_2D, 0, 0, 0, img.width() as _, img.height() as _, gl::RGBA, gl::UNSIGNED_BYTE, img.as_bytes(), 0).unwrap();
        log!("[loaded]");
      },
      _ => log!("unhandled format"),
    }
  
    gl.bind_texture(gl::TEXTURE_2D, self.tex.as_ref());

    self.uniform_offset = gl.get_uniform_location(self.prog.as_ref().unwrap(), "offset");
    self.uniform_trans = gl.get_uniform_location(self.prog.as_ref().unwrap(), "trans");

    let trans = perspective(45.0, {
      let AppConfig { width, height, .. } = self.info();
      width as f32 / height as f32
    }, 0.1, 1000.) * translate(0., 0., -2.5);
    gl.use_program(self.prog.as_ref());
    gl.uniform_matrix4fv_with_f32_sequence(self.uniform_trans.as_ref(), false, &unsafe { js_sys::Float32Array::view_mut_raw(addr_of!(trans) as _, 16).into() });

    self.vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.vao.as_ref());
  }

  fn render(&self, gl: &gl, _: f64) {

    gl.clear_color(0.2, 0.4, 0.2, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);

    let wrapmodes = [gl::CLAMP_TO_EDGE, gl::REPEAT, 0xFFFFF, gl::MIRRORED_REPEAT ];
    let offset = [ -0.5, -0.5,
                    0.5, -0.5,
                   -0.5,  0.5,
                    0.5,  0.5f32 ];

    gl.use_program(self.prog.as_ref());

    // gl. (TEXTURE_2D, TEXTURE_BORDER_COLOR, yellow);

    for i in 0..wrapmodes.len() {
      if wrapmodes[i] == 0xFFFFF {
        // wrap_mode in WebGL2 supported:
        // - gl.REPEAT (default value)
        // - gl.CLAMP_TO_EDGE
        // - gl.MIRRORED_REPEAT
        //
        // gl.CLAMP_TO_BORDER is don't supported.
        // So I have to draw border into texture, and use gl.CLAMP_TO_EDGE
        // to simulate this.
        gl.bind_texture(gl::TEXTURE_2D, self.texborder.as_ref());
        gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
        gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);  
      } else {
        gl.bind_texture(gl::TEXTURE_2D, self.tex.as_ref());
        gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrapmodes[i] as _);
        gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrapmodes[i] as _);
      }

      gl.uniform2fv_with_f32_array(self.uniform_offset.as_ref(), &offset[(2 * i)..(2 * i + 2)]);
      gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
      gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

      gl.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
    }
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_program(self.prog.as_ref());
    gl.delete_vertex_array(self.vao.as_ref());
    gl.delete_texture(self.tex.as_ref());
    gl.delete_texture(self.texborder.as_ref());
  }
}
