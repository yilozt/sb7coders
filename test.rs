use gl::*;
use sb7::gl;
use sb7::{application::*, vmath::*};
use std::{ffi::CString, mem::size_of, ptr::addr_of};

#[derive(Default)]
struct App {
  tex_obj: u32,
  prog:    u32,
  vao:     u32,
  vbo:     u32,
  mv_mat:  Mat4,
}

impl Application for App {
  fn startup(&mut self) {
    let vs_src = "
      #version 460 core

      in vec2 pos;
      out vec2 tc;

      uniform mat4 mv_mat;

      void main() {
        gl_Position = mv_mat * vec4(pos, 0.0, 0.0);
        tc = pos;
      }
    ";
    let fs_src = "
      #version 460 core

      in vec2 tc;
      out vec4 color;

      void main() {
        color = vec4(1.0);
      }
    ";
    
    self.prog = sb7::program::link_from_shaders(&[
      sb7::shader::from_str(vs_src, VERTEX_SHADER, true),
      sb7::shader::from_str(fs_src, FRAGMENT_SHADER, true),
    ], true);

    let str = CString::new("mv_mat").unwrap();
    gl! { GetUniformLocation(self.prog, str.as_ptr()); }

    let vertices = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0f32];

    gl! {
      GenVertexArrays(1, &mut self.vao);

      GenBuffers(1, &mut self.vbo);
      BindBuffer(VERTEX_SHADER, self.vbo);
      NamedBufferData(self.vao,
                      std::mem::size_of_val(&vertices) as _,
                      vertices.as_ptr() as _,
                      STATIC_DRAW);

      BindVertexArray(self.vao);
      VertexAttribPointer(0, 2, FLOAT, FALSE, (4 * size_of::<f32>()) as _, 0 as _);
      EnableVertexArrayAttrib(self.vao, 0);
    }
  }

  fn render(&self, current_time: f64) {
    gl! {
      ClearBufferfv(COLOR, 0, [0.0, 0.0, 0.0, 1.0].as_ptr());

    }

    sb7::gl!{
      GetUniformLocation(self.prog, std::ptr::null());
    }
  }
}

fn main() {
  App::default().run();
}
