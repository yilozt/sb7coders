use std::ffi::CString;
use sb7::gl;
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
      title: "OpenGL SuperBible - Vertex Attributes".to_string(),
      ..Default::default()
    }
  }

  fn startup(&mut self) {
    let vs_source = CString::new(
      "
        #version 460 core

        // 输入到顶点着色器的属性
        layout (location = 0) in vec4 offset;

        void main(void) {
          const vec4 vertices[3] = vec4[3](vec4(0.25, -0.25, 0.5, 1.0),
                                           vec4(-0.25, -0.25, 0.5, 1.0),
                                           vec4(0.25, 0.25, 0.5, 1.0));
          gl_Position = vertices[gl_VertexID] + offset;
        }
      ",
    )
    .unwrap();

    let fs_source = CString::new(
      "
        #version 460 core

        out vec4 color;
        void main(void) {
          color = vec4(1.0);
        }
      ",
    )
    .unwrap();

    gl! {
      // 创建着色器程序
      let program = gl::CreateProgram();

      // 创建着色器，添加着色器代码，编译着色器
      let vs = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(vs, 1, &vs_source.as_ptr(), std::ptr::null());
      gl::CompileShader(vs);

      let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(fs, 1, &fs_source.as_ptr(), std::ptr::null());
      gl::CompileShader(fs);

      // 链接着色器
      gl::AttachShader(program, vs);
      gl::AttachShader(program, fs);
      gl::LinkProgram(program);

      // shader 已经添加到 program 里，因此不再需要
      gl::DeleteShader(vs);
      gl::DeleteShader(fs);

      // 创建、使用 vao
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
    gl! {
      // 清屏
      gl::ClearBufferfv(gl::COLOR, 0, green.as_ptr());
      
      // 使用着色器程序
      gl::UseProgram(self.program);

      // 将 offset 属性作为顶点着色器的输入
      gl::VertexAttrib4fv(0, attrib.as_ptr());
      
      // 绘制三角形
      gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
  }

  fn shutdown(&mut self) {
    gl! {
      gl::DeleteProgram(self.program);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

fn main() {
  let mut app = MyApp::default();
  application::Application::run(&mut app);
}
