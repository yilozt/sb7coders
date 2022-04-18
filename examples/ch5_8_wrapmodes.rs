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
use sb7::{application::Application, gl};

#[derive(Default)]
struct App {
  vao:            u32,
  tex:            u32,
  uniform_offset: i32,
  prog:           u32,
}

impl Application for App {
  fn init(&self) -> sb7::application::AppConfig {
    sb7::application::AppConfig {
      title: "OpenGL SuperBible - Texture Wrap Modes".into(),
      ..Default::default()
    }
  }

  fn startup(&mut self) {
    let vs_src = "
      #version 460 core

      uniform vec2 offset;

      out vec2 tex_coord;

      void main(void) {
        const vec4 vertices[] = vec4[](vec4(-0.45, -0.45, 0.5, 1.0),
                                       vec4( 0.45, -0.45, 0.5, 1.0),
                                       vec4(-0.45,  0.45, 0.5, 1.0),
                                       vec4( 0.45,  0.45, 0.5, 1.0));
        gl_Position = vertices[gl_VertexID] + vec4(offset, 0.0, 0.0); 
        tex_coord = vertices[gl_VertexID].xy * 3.0 + vec2(0.45 * 3);
      }
    ";
    
    let fs_src = "
      #version 410 core
      uniform sampler2D s;
      out vec4 color;

      in vec2 tex_coord;

      void main(void) {
        color = texture(s, tex_coord);
      }
    ";

    let vs = sb7::shader::from_str(vs_src, VERTEX_SHADER, true);
    let fs = sb7::shader::from_str(fs_src, FRAGMENT_SHADER, true);
    self.prog = sb7::program::link_from_shaders(&[vs, fs], true);

    let name = std::ffi::CString::new("offset").unwrap();
    self.tex = sb7::ktx::file::load("media/textures/rightarrows.ktx").unwrap().0;
  
    gl! {
      BindTexture(TEXTURE_2D, self.tex);

      self.uniform_offset = GetUniformLocation(self.prog, name.as_ptr());

      GenVertexArrays(1, &mut self.vao);
      BindVertexArray(self.vao);
    }
  }

  fn render(&mut self, _: f64) {
    let green = [0.0, 0.1, 0.0, 1.0f32].as_ptr();
    let yellow = [0.4, 0.4, 0.0, 1.0f32].as_ptr();

    gl! { ClearBufferfv(COLOR, 0, green); }

    let wrapmodes = [ CLAMP_TO_EDGE, REPEAT, CLAMP_TO_BORDER, MIRRORED_REPEAT ];
    let offset = [ -0.5, -0.5,
                    0.5, -0.5,
                   -0.5,  0.5,
                    0.5,  0.5f32 ];

    gl! {
      UseProgram(self.prog);

      TexParameterfv(TEXTURE_2D, TEXTURE_BORDER_COLOR, yellow);
    }

    for i in 0..4 {
      gl! {
        Uniform2fv(self.uniform_offset, 1, &offset[2 * i] as *const f32 as _);
        TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, wrapmodes[i] as _);
        TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, wrapmodes[i] as _);

        DrawArrays(TRIANGLE_STRIP, 0, 4);
      }
    }
  }

  fn shutdown(&mut self) {
    gl! {
      DeleteShader(self.prog);
      DeleteVertexArrays(1, &self.vao);
      DeleteTextures(1, &self.tex);
    }      
  }
}

fn main() {
  App::default().run()
}
