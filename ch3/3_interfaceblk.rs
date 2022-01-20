use std::ffi::CString;

use gl::types::GLuint;
use sb7::application::{AppConfig, Application};

#[derive(Default)]
struct MyApp {
  vao: GLuint,
  program: GLuint,
}

impl Application for MyApp {
  fn init(&self) -> sb7::application::AppConfig {
    AppConfig {
      title: "OpenGL SuperBible - Interface Block".to_string(),
      ..Default::default()
    }
  }

  fn startup(&mut self) {
    let vs_source = CString::new(
      "
          #version 460 core

          // 可以将在 shader 之间传递的数据放到一个块结构里
          out VS_OUT {
            vec4 color;
          } vs_out;

          void main() {
            const vec4 vertices[3] = vec4[3](vec4(0.0, 0.30, 0.0, 1.0),
                                             vec4(-0.25, -0.25, 0.0, 1.0),
                                             vec4(0.25, -0.25, 0.0, 1.0));
            const vec4 colors[3] = vec4[3](vec4(1.0, 0.0, 0.0, 1.0),
                                           vec4(0.0, 1.0, 0.0, 1.0),
                                           vec4(0.0, 0.0, 1.0, 1.0));
            gl_Position = vertices[gl_VertexID];
            vs_out.color = colors[gl_VertexID];
          }
        ",
    )
    .unwrap();

    let fs_source = CString::new(
      "
        #version 460 core

        // 从上一着色器传递过来的块结构
        // VS_OUT 这个块名必须一样，变量名可以不同
        in VS_OUT {
          vec4 color;
        } vs_in;

        out vec4 color;

        void main(void) {
          color = vs_in.color;
        }
      ",
    )
    .unwrap();

    unsafe {
      self.program = gl::CreateProgram();

      let vs = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(vs, 1, &vs_source.as_ptr(), std::ptr::null());
      gl::CompileShader(vs);

      let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(fs, 1, &fs_source.as_ptr(), std::ptr::null());
      gl::CompileShader(fs);

      gl::AttachShader(self.program, vs);
      gl::AttachShader(self.program, fs);
      gl::LinkProgram(self.program);

      gl::DeleteShader(vs);
      gl::DeleteShader(fs);

      gl::CreateVertexArrays(1, &mut self.vao);
      gl::BindVertexArray(self.vao);
      gl::UseProgram(self.program);
    }
  }

  fn render(&self, _current_time: f64) {
    let color = [0.0, 0.0, 0.0, 0.0];
    unsafe {
      gl::ClearBufferfv(gl::COLOR, 0, color.as_ptr());
      gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
  }

  fn shutdown(&self) {
    unsafe {
      gl::DeleteProgram(self.program);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

fn main() {
  MyApp::default().run()
}
