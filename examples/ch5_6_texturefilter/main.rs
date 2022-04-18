use gl::*;
use sb7::gl;
use sb7::{application::*, vmath::*};
use std::{ffi::CString, mem::size_of};

mod data;
use data::generate_tex;

#[derive(Clone, Copy)]
enum TexFilter {
  Nearst = NEAREST as _,
  Linear = LINEAR as _,
}

impl Default for TexFilter {
  #[inline(always)]
  fn default() -> Self { Self::Linear }
}

impl TexFilter {
  #[inline(always)]
  fn apply_filter(&self, tex: u32) {
    let filter = match self {
      a@TexFilter::Nearst => *a as isize,
      a@TexFilter::Linear => *a as isize,
    };

    gl! {
      BindTexture(TEXTURE_2D, tex);
      TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, filter as _);
      TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, filter as _);
    }
  }
}

#[derive(Default)]
struct App {
  tex_obj: u32,
  prog:    u32,
  vao:     u32,
  vbo:     u32,
  mat:     i32,
}

impl Application for App {
  fn startup(&mut self) {
    let vs_src = "
      #version 460 core

      in vec2 pos;
      out vec2 tc;

      uniform mat4 mat = mat4(1.0);

      void main() {
        gl_Position = mat * vec4(pos, 0.0, 1.0);
        tc = pos + vec2(0.5);
        tc.y = -tc.y;
      }
    ";
    let fs_src = "
      #version 460 core

      uniform sampler2D tex;

      in vec2 tc;
      out vec4 color;

      void main() {
        color = texture(tex, tc);
      }
    ";
    
    self.prog = sb7::program::link_from_shaders(&[
      sb7::shader::from_str(vs_src, VERTEX_SHADER, true),
      sb7::shader::from_str(fs_src, FRAGMENT_SHADER, true),
    ], true);

    let str = CString::new("mat").unwrap();
    gl! {
      self.mat = GetUniformLocation(self.prog, str.as_ptr());
      UseProgram(self.prog);
    }

    let vertices = [
       0.5f32,  0.5f32,  // 右上角
       0.5f32, -0.5f32,  // 右下角
      -0.5f32,  0.5f32,  // 左上角

      -0.5f32,  0.5f32,  // 左上角
       0.5f32, -0.5f32,  // 右下角
      -0.5f32, -0.5f32,  // 左下角
    ];

    gl! {
      CreateVertexArrays(1, &mut self.vao);
      BindVertexArray(self.vao);

      CreateBuffers(1, &mut self.vbo);
      BindBuffer(ARRAY_BUFFER, self.vbo);
      NamedBufferData(self.vbo,
                      std::mem::size_of_val(&vertices) as _,
                      vertices.as_ptr() as _,
                      STATIC_DRAW);

      VertexAttribPointer(0, 2, FLOAT, FALSE, (2 * size_of::<f32>()) as _, 0 as _);
      EnableVertexArrayAttrib(self.vao, 0);
    }

    let (width, height, _, data) = generate_tex();
    gl! {
      CreateTextures(TEXTURE_2D, 1, &mut self.tex_obj);
      BindTexture(TEXTURE_2D, self.tex_obj);
      TexStorage2D(TEXTURE_2D, 1, RGBA8, width as _, height as _);
      TexSubImage2D(TEXTURE_2D, 0, 0, 0, width as _, height as _, RGBA, UNSIGNED_BYTE, data.as_ptr() as _);
    }
  }

  fn render(&mut self, current_time: f64) {
    gl! { ClearBufferfv(COLOR, 0, [0.0, 0.0, 0.0].as_ptr()); }

    let aspect = {
      let AppConfig {width, height, ..} = self.info();
      width as f32 / height as f32
    };

    let current_time = (current_time * 30.0) as f32;

    for (pos, filter) in [(-0.6f32, TexFilter::Linear), (0.6f32, TexFilter::Nearst)] {
      let mat: Mat4 = sb7::vmath::perspective(45.0, aspect, 0.1, 1000.0)
      * sb7::vmath::translate(pos, 0.0, -3.0)
      * sb7::vmath::rotate_with_axis(current_time, 0.0, 1.0, 0.0);

      gl! {
        BindVertexArray(self.vao);
        UniformMatrix4fv(self.mat, 1, FALSE, std::ptr::addr_of!(mat) as _);

        filter.apply_filter(self.tex_obj);
        
        DrawArrays(TRIANGLES, 0, 6);
      }
    }
  }
}

fn main() {
  App::default().run();
}