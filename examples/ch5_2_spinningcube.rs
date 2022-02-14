use gl::types::*;
use sb7::application::{Application, AppConfig};
use sb7::mat4;
use sb7::vmath::{Mat4, translate, rotate_with_axis};
use std::ffi::CString;
use std::mem::size_of_val;
use std::ptr::{null, addr_of};

#[derive(Default)]
struct App {
  vao: GLuint,
  buf: GLuint,
  program: GLuint,
  proj_matrix: Mat4,
}

impl Application for App {
  fn startup(&mut self) {
    #[rustfmt::skip]
    let vertex_position : &[f32]= &[
      -0.25,  0.25, -0.25,
      -0.25, -0.25, -0.25,
       0.25, -0.25, -0.25,

       0.25, -0.25, -0.25,
       0.25,  0.25, -0.25,
      -0.25,  0.25, -0.25,

       0.25, -0.25, -0.25,
       0.25, -0.25,  0.25,
       0.25,  0.25, -0.25,

       0.25, -0.25,  0.25,
       0.25,  0.25,  0.25,
       0.25,  0.25, -0.25,

       0.25, -0.25,  0.25,
      -0.25, -0.25,  0.25,
       0.25,  0.25,  0.25,

      -0.25, -0.25,  0.25,
      -0.25,  0.25,  0.25,
       0.25,  0.25,  0.25,

      -0.25, -0.25,  0.25,
      -0.25, -0.25, -0.25,
      -0.25,  0.25,  0.25,

      -0.25, -0.25, -0.25,
      -0.25,  0.25, -0.25,
      -0.25,  0.25,  0.25,

      -0.25, -0.25,  0.25,
       0.25, -0.25,  0.25,
       0.25, -0.25, -0.25,

       0.25, -0.25, -0.25,
      -0.25, -0.25, -0.25,
      -0.25, -0.25,  0.25,

      -0.25,  0.25, -0.25,
       0.25,  0.25, -0.25,
       0.25,  0.25,  0.25,

       0.25,  0.25,  0.25,
      -0.25,  0.25,  0.25,
      -0.25,  0.25, -0.25
    ];

    unsafe {
      let mut vao = 0;
      gl::CreateVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);

      let mut buf = 0;
      gl::CreateBuffers(1, &mut buf);
      gl::BindBuffer(gl::ARRAY_BUFFER, buf);
      gl::NamedBufferData(buf,
                          size_of_val(vertex_position) as _,
                          vertex_position.as_ptr() as _,
                          gl::STATIC_DRAW);
      gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
      gl::EnableVertexArrayAttrib(vao, 0);

      let vs_source = CString::new("
        #version 460 core

        in vec4 position;
        
        out VS_OUT {
          vec4 color;
        } vs_out;

        layout (location = 0) uniform mat4 mv_matrix = mat4(1.0);
        layout (location = 1) uniform mat4 proj_matrix = mat4(1.0);

        void main() {
          gl_Position =  proj_matrix * mv_matrix * position;
          vs_out.color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
        }
      ").unwrap();
      let vs = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
      gl::CompileShader(vs);
        
      let fs_source = CString::new("
        #version 460 core

        out vec4 color;
        
        in VS_OUT {
          vec4 color;
        } fs_in;

        void main() {
          color = fs_in.color;
        }
      ").unwrap();
      let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
      gl::CompileShader(fs);

      let program = gl::CreateProgram();
      gl::AttachShader(program, vs);
      gl::AttachShader(program, fs);
      gl::LinkProgram(program);
      gl::DeleteShader(vs);
      gl::DeleteShader(fs);

      gl::UseProgram(program);

      gl::Enable(gl::DEPTH_TEST);
      *self = Self { vao, program, buf, proj_matrix: mat4!() };
    }

    let AppConfig { width, height, .. } = AppConfig::default();
    self.on_resize(width as _, height as _);
  }

  fn render(&self, current_time: f64) {
    unsafe {
      let current_time = current_time as f32;
      let f = current_time * 0.3;
      let mv_matrix = translate(0.0, 0.0, -4.0) *
                      translate((2.1 * f).sin() * 0.5,
                                (1.7 * f).cos() * 0.5,
                                (1.3 * f).sin() * (1.5 * f).cos() * 2.0) *
                      rotate_with_axis(current_time * 45.0, 0.0, 1.0, 0.0) *
                      rotate_with_axis(current_time * 81.0, 1.0, 0.0, 0.0);
      gl::UniformMatrix4fv(0, 1, gl::FALSE, addr_of!(mv_matrix) as _);
      
      gl::ClearBufferfv(gl::COLOR,0, [0.0, 0.0, 0.0].as_ptr());
      gl::ClearBufferfv(gl::DEPTH, 0, &1.0);
      gl::DrawArrays(gl::TRIANGLES, 0, 36);
    }
  }

  fn on_resize(&mut self, w: i32, h: i32) {
    let aspect = w as GLfloat / h as GLfloat;
    self.proj_matrix = sb7::vmath::perspective(50.0, aspect, 0.1, 1000.0);
    unsafe {
      gl::UniformMatrix4fv(1, 1, gl::FALSE, addr_of!(self.proj_matrix) as _);
    }
  }

  fn shutdown(&mut self) {
    unsafe {
      gl::DeleteBuffers(2, &self.buf);
      gl::DeleteProgram(self.program);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

fn main() {
  App::default().run()
}
