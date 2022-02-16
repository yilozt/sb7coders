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

use std::mem::size_of;
use rand::prelude::random;
use sb7::prelude::*;

#[derive(Default)]
struct App {
  render_prog:     u32,
  render_vao:      u32,
  rain_buffer:     u32,
  tex_alien_array: u32,

  droplet_x_offset:   Vec<f32>,
  droplet_rot_speed:  Vec<f32>,
  droplet_fall_speed: Vec<f32>,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - Alien Rain".into(),
                ..Default::default() }
  }

  fn startup(&mut self) {
    let vs_src = "
      #version 460 core

      layout (location = 0) in int alien_index;

      out VS_OUT {
        flat int alien;
        vec2 tc;
      } vs_out;

      struct droplet_t {
        float x_offset;
        float y_offset;
        float orientation;
        float unused;
      };

      layout (std140) uniform droplets {
        droplet_t droplet[256];
      };

      void main(void) {
        const vec2[4] position = vec2[4](vec2(-0.5, -0.5),
                                         vec2( 0.5, -0.5),
                                         vec2(-0.5,  0.5),
                                         vec2( 0.5,  0.5));
        vs_out.tc = position[gl_VertexID].xy + vec2(0.5);
        float co = cos(droplet[alien_index].orientation);
        float so = sin(droplet[alien_index].orientation);
        mat2 rot = mat2(vec2(co, so),
                        vec2(-so, co));
        vec2 pos = 0.25 * rot * position[gl_VertexID];
        gl_Position = vec4(pos.x + droplet[alien_index].x_offset,
                           pos.y + droplet[alien_index].y_offset,
                           0.5, 1.0);
        vs_out.alien = alien_index % 64;
      }
    ";

    let vs_src = std::ffi::CString::new(vs_src).unwrap();

    let fs_src = "
      #version 460 core

      layout (location = 0) out vec4 color;

      in VS_OUT {
        flat int alien;
        vec2 tc;
      } fs_in;

      uniform sampler2DArray tex_aliens;

      void main(void) {
        color = texture(tex_aliens, vec3(fs_in.tc, float(fs_in.alien)));
      }
    ";

    let fs_src = std::ffi::CString::new(fs_src).unwrap();

    gl! {
      let buf: [u8; 1024] = [0; 1024];

      let vs = CreateShader(VERTEX_SHADER);
      ShaderSource(vs, 1, &vs_src.as_ptr(), std::ptr::null());
      CompileShader(vs);

      GetShaderInfoLog(vs, 1024, std::ptr::null_mut(), buf.as_ptr() as _);
      println!("{}", std::str::from_utf8(&buf).unwrap_or("invaild str"));

      let fs = CreateShader(FRAGMENT_SHADER);
      ShaderSource(fs, 1, &fs_src.as_ptr(), std::ptr::null());
      CompileShader(fs);

      GetShaderInfoLog(fs, 1024, std::ptr::null_mut(), buf.as_ptr() as _);
      println!("{}", std::str::from_utf8(&buf).unwrap_or("invaild str"));

      self.render_prog = CreateProgram();
      AttachShader(self.render_prog, vs);
      AttachShader(self.render_prog, fs);
      LinkProgram(self.render_prog);

      DeleteShader(vs);
      DeleteShader(fs);
    }

    gl! {
      GenVertexArrays(1, &mut self.render_vao);
      BindVertexArray(self.render_vao);
    }

    gl! {
      self.tex_alien_array = ktx::file::load("media/textures/aliens.ktx").unwrap().0;
      BindTexture(TEXTURE_2D_ARRAY, self.tex_alien_array);
      TexParameteri(TEXTURE_2D_ARRAY, TEXTURE_MIN_FILTER, LINEAR_MIPMAP_LINEAR as _);
    }

    gl! {
      GenBuffers(1, &mut self.rain_buffer);
      BindBuffer(UNIFORM_BUFFER, self.rain_buffer);
      BufferData(UNIFORM_BUFFER, (256 * size_of::<Vec4>()) as _, std::ptr::null(), DYNAMIC_DRAW);
    }

    self.droplet_fall_speed.reserve(256);
    self.droplet_rot_speed.reserve(256);
    self.droplet_x_offset.reserve(256);

    for i in 0..256 {
      self.droplet_x_offset.push(random::<f32>() * 2.0 - 1.0);
      self.droplet_rot_speed.push(random::<f32>() + 0.5 * (match i & 1 { 0 => 3.0, _ => -3.0 }));
      self.droplet_fall_speed.push(random::<f32>() + 0.2);
    }

    gl! {
      BindVertexArray(self.render_vao);
      Enable(BLEND);
      BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
    }
  }

  fn render(&self, current_time: f64) {
    let t = current_time as f32;

    let droplet: *mut Vec4 = gl! {
      ClearBufferfv(COLOR, 0, color::Black.as_ptr());
      UseProgram(self.render_prog);

      BindBufferBase(UNIFORM_BUFFER, 0, self.rain_buffer);
      MapBufferRange(UNIFORM_BUFFER, 0, (256 * size_of::<Vec4>()) as _, MAP_WRITE_BIT | MAP_INVALIDATE_BUFFER_BIT) as _
    };
    assert_ne!(droplet as usize, 0, "buf map to NULL");

    unsafe {
      for i in 0..256 {
        (&mut *(droplet.add(i)))[0] = self.droplet_x_offset[i];
        (&mut *(droplet.add(i)))[1] =
          2.0 - ((t + i as f32) * self.droplet_fall_speed[i]) % 4.31;
        (&mut *(droplet.add(i)))[2] = t * self.droplet_rot_speed[i];
        (&mut *(droplet.add(i)))[3] = 0.0;
      }
    }

    gl! { UnmapBuffer(UNIFORM_BUFFER); }

    for i in 0..256 {
      gl! {
        VertexAttribI1i(0, i as _);
        DrawArrays(TRIANGLE_STRIP, 0, 4);
      }
    }
  }

  fn shutdown(&mut self) {
    gl! {
      DeleteTextures(1, &self.tex_alien_array);
      DeleteBuffers(1, &self.rain_buffer);
      DeleteVertexArrays(1, &self.render_vao);
      DeleteProgram(self.render_prog);
    }
  }
}

fn main() {
  App::default().run();
}
