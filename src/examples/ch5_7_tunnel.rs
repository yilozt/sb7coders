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

use std::ffi::CString;
use sb7::application::*;
use sb7::gl;

#[derive(Default)]
struct Uniforms {
  mvp:    i32,
  offset: i32,
}


#[derive(Default)]
struct App {
  render_prog: u32,
  render_vao:  u32,
  uniforms:    Uniforms,
  tex_wall:    u32,
  tex_ceiling: u32,
  tex_floor:   u32,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig {
      title: String::from("OpenGL SuperBible - Tunnel"),
      ..AppConfig::default()
    }
  }

  fn startup(&mut self) {
    let vs_src = "
      #version 460 core

      out VS_OUT {
        vec2 tc;
      } vs_out;

      uniform mat4 mvp;
      uniform float offset;

      void main(void) {
        const vec2[4] position = vec2[4](vec2(-0.5, -0.5),
                                         vec2( 0.5, -0.5),
                                         vec2(-0.5,  0.5),
                                         vec2( 0.5,  0.5));
        vs_out.tc = (position[gl_VertexID].xy + vec2(offset, 0.5)) * vec2(30.0, 1.0);
        gl_Position = mvp * vec4(position[gl_VertexID], 0.0, 1.0);
      }
    ";

    let fs_src = "
      #version 460 core

      layout (location = 0) out vec4 color;

      in VS_OUT {
        vec2 tc;
      } fs_in;

      layout (binding = 0) uniform sampler2D tex;

      void main(void) {
        color = texture(tex, fs_in.tc);
      }
    ";

    self.render_prog = sb7::program::link_from_shaders(&[
      sb7::shader::from_str(vs_src, gl::VERTEX_SHADER, true),
      sb7::shader::from_str(fs_src, gl::FRAGMENT_SHADER, true)
    ], true);

    let get_location = |name| gl!{
      let name = CString::new(name).unwrap();
      gl::GetUniformLocation(self.render_prog, name.as_ptr() as _)
    };

    self.uniforms.mvp    = get_location("mvp");
    self.uniforms.offset = get_location("offset");

    self.tex_wall = sb7::ktx::file::load("media/textures/brick.ktx").unwrap().0;
    self.tex_ceiling = sb7::ktx::file::load("media/textures/ceiling.ktx").unwrap().0;
    self.tex_floor = sb7::ktx::file::load("media/textures/floor.ktx").unwrap().0;
  
    for tex in [self.tex_wall, self.tex_floor, self.tex_ceiling] {
      gl! {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
      }
    }

    gl!{
      gl::GenVertexArrays(1, &mut self.render_vao);
      gl::BindVertexArray(self.render_vao);
    }
  }

  fn render(&self, current_time: f64) {
    let t = current_time as f32;
    let black = [0.0, 0.0, 0.0, 0.0f32].as_ptr();

    gl!{
      gl::ClearBufferfv(gl::COLOR, 0, black);
      gl::UseProgram(self.render_prog);
    }

    let proj_matrix = {
      let AppConfig { width, height, .. } = self.info();
      let aspect = width as f32 / height as f32;
      sb7::vmath::perspective(60.0, aspect, 0.1, 100.0)
    };

    gl!(gl::Uniform1f(self.uniforms.offset, t * 0.003));

    let textures = [ self.tex_wall, self.tex_floor, self.tex_wall, self.tex_ceiling ];
    for (i, tex) in textures.iter().enumerate() {
      let mv_matrix = sb7::vmath::rotate_with_axis(90.0 * i as f32, 0.0, 0.0, 1.0)
                    * sb7::vmath::translate(-0.5, 0.0, -10.0)
                    * sb7::vmath::rotate_with_axis(90.0, 0.0, 1.0, 0.0)
                    * sb7::vmath::scale(30.0, 1.0, 1.0);
      let mvp = proj_matrix * mv_matrix;

      gl! {
        gl::UniformMatrix4fv(self.uniforms.mvp, 1, gl::FALSE, std::ptr::addr_of!(mvp) as _);

        gl::BindTexture(gl::TEXTURE_2D, *tex);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
      }
    }
  }

  fn shutdown(&mut self) {
    gl!{
      gl::DeleteProgram(self.render_prog);
      gl::DeleteVertexArrays(1, &self.render_vao);
      for t in [self.tex_wall, self.tex_floor, self.tex_ceiling] {
        gl::DeleteTextures(1, &t);
      }
    }
  }
}


fn main() {
  App::default().run();
}
