use std::ffi::CString;

use gl::types::*;
use sb7::application;

#[derive(Default)]
struct MyApp {
  vao: GLuint,
  program: GLuint,
}

impl application::Application for MyApp {
  fn init(&self) -> application::AppConfig {
    application::AppConfig {
      title: "OpenGL SuperBible - Passing Data from Stage to Stage".to_string(),
      ..Default::default()
    }
  }

  fn startup(&mut self) {
    let vs_source = CString::new(
      "
        #version 460 core

        layout (location = 0) in vec4 offset;
        layout (location = 1) in vec4 color;

        // 输出到下一阶段着色器的数据
        out vec4 vs_color;

        void main(void) {
          const vec4 vertices[3] = vec4[3](vec4(0.25, -0.25, 0.5, 1.0),
                                           vec4(-0.25, -0.25, 0.5, 1.0),
                                           vec4(0.25, 0.25, 0.5, 1.0));
          gl_Position = vertices[gl_VertexID] + offset;
          
          // 输出数据
          vs_color = color;
        }
      ",
    )
    .unwrap();

    let fs_source = CString::new(
      "
        #version 460 core

        // 从上一阶段传过来的属性，名字必须相同
        in vec4 vs_color;

        out vec4 color;
        void main(void) {
          color = vs_color;
        }
      ",
    )
    .unwrap();

    sb7::gl! {
      let program = gl::CreateProgram();

      let vs = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(vs, 1, &vs_source.as_ptr(), std::ptr::null());
      gl::CompileShader(vs);

      let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(fs, 1, &fs_source.as_ptr(), std::ptr::null());
      gl::CompileShader(fs);

      gl::AttachShader(program, vs);
      gl::AttachShader(program, fs);
      gl::LinkProgram(program);

      gl::DeleteShader(vs);
      gl::DeleteShader(fs);

      let mut vao = 0;
      gl::CreateVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);

      self.vao = vao;
      self.program = program;
    }
  }

  fn render(&self, current_time: f64) {
    let current_time = current_time as f32;
    let green = [0.0, 0.0, 0.0, 0.0f32];
    let attrib = [current_time.sin() * 0.5, current_time.cos() * 0.6, 0.0, 0.0];
    let color = [
      current_time.sin() * 0.5 + 0.5,
      current_time.cos() * 0.5 + 0.5,
      0.5,
      0.0,
    ];
    sb7::gl! {
      gl::ClearBufferfv(gl::COLOR, 0, green.as_ptr());
      gl::UseProgram(self.program);

      gl::VertexAttrib4fv(0, attrib.as_ptr());
      gl::VertexAttrib4fv(1, color.as_ptr());
      gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
  }

  fn shutdown(&mut self) {
    sb7::gl! {
      gl::DeleteProgram(self.program);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

fn main() {
  let mut app = MyApp::default();
  application::Application::run(&mut app);
}
