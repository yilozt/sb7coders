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
  render_prog:     Option<WebGlProgram>,
  render_vao:      Option<WebGlVertexArrayObject>,
  rain_buffer:     Option<WebGlBuffer>, 
  tex_alien_array: Option<WebGlTexture>,

  droplet_x_offset:   Vec<f32>,
  droplet_rot_speed:  Vec<f32>,
  droplet_fall_speed: Vec<f32>,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - Alien Rain".into(),
                ..Default::default() }
  }

  fn startup(&mut self, gl: &gl) {
    let vs_src = "#version 300 es
      precision mediump float;

      layout(std140) uniform;

      struct droplet_t {
        float x_offset;
        float y_offset;
        float orientation;
        float unused;
      };

      layout (location = 0) in float alien_index;

      out float alien;
      out vec2 tc;

      uniform droplets {
        droplet_t droplet[256];
      };


      void main(void) {
        const vec2[4] position = vec2[4](vec2(-0.5, -0.5),
                                         vec2( 0.5, -0.5),
                                         vec2(-0.5,  0.5),
                                         vec2( 0.5,  0.5));
        tc = position[gl_VertexID].xy + vec2(0.5);
        float co = cos(droplet[int(alien_index)].orientation);
        float so = sin(droplet[int(alien_index)].orientation);
        mat2 rot = mat2(vec2(co, so),
                        vec2(-so, co));
        vec2 pos = 0.25 * rot * position[gl_VertexID];
        gl_Position = vec4(pos.x + droplet[int(alien_index)].x_offset,
                           pos.y + droplet[int(alien_index)].y_offset,
                           0.5, 1.0);
        alien = float(int(alien_index) % 9);
      }
    ";

    let fs_src = "#version 300 es
      precision mediump float;
      precision mediump sampler2DArray;

      in float alien;
      in vec2 tc;

      layout (location = 0) out vec4 color;

      uniform sampler2DArray tex_aliens;

      void main(void) {
        color = texture(tex_aliens, vec3(tc, float(int(alien))));
      }
    ";

    let vs = gl.create_shader(gl::VERTEX_SHADER);
    gl.shader_source(vs.as_ref().unwrap(), vs_src);
    gl.compile_shader(vs.as_ref().unwrap());

    let fs = gl.create_shader(gl::FRAGMENT_SHADER);
    gl.shader_source(fs.as_ref().unwrap(), fs_src);
    gl.compile_shader(fs.as_ref().unwrap());

    self.render_prog = gl.create_program();
    gl.attach_shader(self.render_prog.as_ref().unwrap(), vs.as_ref().unwrap());
    gl.attach_shader(self.render_prog.as_ref().unwrap(), fs.as_ref().unwrap());
    gl.link_program(self.render_prog.as_ref().unwrap());

    gl.delete_shader(vs.as_ref());
    gl.delete_shader(fs.as_ref());

    self.render_vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.render_vao.as_ref());

    self.load_2d_array_tex(gl);

    self.rain_buffer = gl.create_buffer();
    gl.bind_buffer_base(gl::UNIFORM_BUFFER, 0, self.rain_buffer.as_ref());
    gl.buffer_data_with_i32(gl::UNIFORM_BUFFER, 256 * 4, gl::DYNAMIC_DRAW);

    self.droplet_fall_speed.reserve(256);
    self.droplet_rot_speed.reserve(256);
    self.droplet_x_offset.reserve(256);

    quad_rand::srand(123456);
    let random = || {
      quad_rand::gen_range(0.0, 1.0)
    };
    
    for i in 0..256 {
      self.droplet_x_offset.push(random() * 2.0 - 1.0);
      self.droplet_rot_speed.push(random() + 0.5 * (match i & 1 { 0 => 3.0, _ => -3.0 }));
      self.droplet_fall_speed.push(random() + 0.2);
    }

    gl.bind_vertex_array(self.render_vao.as_ref());
    gl.enable(gl::BLEND);
    gl.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
  }

  fn render(&self, gl: &gl, current_time: f64) {
    let t = current_time as f32;

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    
    gl.use_program(self.render_prog.as_ref());

    gl.bind_buffer_base(gl::UNIFORM_BUFFER, 0, self.rain_buffer.as_ref());

    let mut data = Vec::with_capacity(256 * 4);

    for i in 0..256 {
      data.push(self.droplet_x_offset[i]);
      // Reset Y position by mod operation
      data.push(2.0 - ((t + i as f32) * self.droplet_fall_speed[i]) % 4.31);
      data.push(t * self.droplet_rot_speed[i]);
      data.push(0.0);
    }

    gl.buffer_data_with_array_buffer_view(gl::UNIFORM_BUFFER, &unsafe { js_sys::Float32Array::view(data.as_slice()) }.into(), gl::DYNAMIC_COPY);


    for i in 0..256 {
      gl.vertex_attrib1f (0, i as _);
      gl.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
    }
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_texture(self.tex_alien_array.as_ref());
    gl.delete_buffer(self.rain_buffer.as_ref());
    gl.delete_vertex_array(self.render_vao.as_ref());
    gl.delete_program(self.render_prog.as_ref());
  }
}

impl App {
  fn load_2d_array_tex(&mut self, gl: &gl) {
    self.tex_alien_array = gl.create_texture();

    gl.bind_texture(gl::TEXTURE_2D_ARRAY, self.tex_alien_array.as_ref());
    gl.tex_storage_3d(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8, 256, 256, 9);

    for (i, data) in [
      &include_bytes!("assert/1.png")[..],
      &include_bytes!("assert/2.png")[..],
      &include_bytes!("assert/3.png")[..],
      &include_bytes!("assert/4.png")[..],
      &include_bytes!("assert/5.png")[..],
      &include_bytes!("assert/6.png")[..],
      &include_bytes!("assert/7.png")[..],
      &include_bytes!("assert/8.png")[..],
      &include_bytes!("assert/9.png")[..],
    ].iter().enumerate() {
      if let Ok(image::DynamicImage::ImageRgba8(img)) = image::load_from_memory(data) {
        gl.tex_sub_image_3d_with_opt_array_buffer_view(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as _, 256, 256, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some(&unsafe { js_sys::Uint8Array::view(img.as_bytes()).into() })).unwrap();
      } else {
        log!("format of assert/{}.png should be RGBA8", i + 1);
      }
    }
    gl.bind_texture(gl::TEXTURE_2D_ARRAY, self.tex_alien_array.as_ref());
    gl.tex_parameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
  }
}