use std::{ffi::CString, ptr};

use gl::types::*;
use sb7::application::{Application};

#[derive(Default)]
struct MyApplication {
  rendering_program: GLuint,
  vertex_array_object: GLuint,
}

impl MyApplication {
  fn check_shader(shader: GLuint) -> Option<String> {
    let mut success = gl::FALSE as GLint;
    let mut log = [0; 1024];
    let mut len: GLsizei = 0;
    unsafe {
      gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    }
    if success != gl::TRUE as GLint {
      unsafe {
        gl::GetShaderInfoLog(shader, 1024, &mut len, log.as_mut_ptr() as *mut GLchar);
        return Some(
          std::str::from_utf8(&log)
            .unwrap_or("invaild utf-8 string")
            .to_string(),
        );
      }
    } else {
      None
    }
  }

  fn compile_shaders(&self) -> GLuint {
    const VERTEX_SHADER_SOUECE: &str = "
      #version 460 core

      void main() {
        const vec4 vertices[3] = vec4[3](
          vec4( 0.25, -0.25, 0.5, 1.0),
          vec4(-0.25, -0.25, 0.5, 1.0),
          vec4( 0.25,  0.25, 0.5, 1.0)
        );

        gl_Position = vertices[gl_VertexID];
      }
    ";

    const FRAGMENT_SHADER_SOUECE: &str = "
      #version 460 core
      out vec4 color;

      void main() {
        color = vec4(0.0, 0.0, 0.0, 1.0);
      }
    ";

    // 创建着色器
    let vertex_shader_source = CString::new(VERTEX_SHADER_SOUECE.as_bytes()).unwrap();
    let fragment_shader_source = CString::new(FRAGMENT_SHADER_SOUECE.as_bytes()).unwrap();

    unsafe {
      let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(vertex_shader, 1, &vertex_shader_source.as_ptr(), ptr::null());
      gl::CompileShader(vertex_shader);

      // 检查编译结果
      if let Some(err) = Self::check_shader(vertex_shader) {
        println!("ERR: vertex shader compile failed.");
        println!("== {}", err);
      }

      let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(fragment_shader, 1, &fragment_shader_source.as_ptr(), ptr::null());
      gl::CompileShader(fragment_shader);

      // 检查编译结果
      if let Some(err) = Self::check_shader(fragment_shader) {
        println!("ERR: fragment shader compile failed.");
        println!("== {}", err);
      }

      // 创建着色器程序
      let program = gl::CreateProgram();
      gl::AttachShader(program, vertex_shader);
      gl::AttachShader(program, fragment_shader);
      gl::LinkProgram(program);

      // 着色器已经被编译，就不再需要了
      gl::DeleteShader(vertex_shader);
      gl::DeleteShader(fragment_shader);

      program
    }
  }
}

impl Application for MyApplication {
  fn startup(&mut self) {
    self.rendering_program = self.compile_shaders();
    unsafe {
      gl::CreateVertexArrays(1, &mut self.vertex_array_object);
      gl::BindVertexArray(self.vertex_array_object);
    }
  }

  fn render(&self, current_time: f64) {
    unsafe {
      let g = (current_time as f32).sin() * 0.5 + 0.5;
      gl::ClearBufferfv(gl::COLOR, 0, &[g, g, g, 1.0f32] as *const f32);

      gl::UseProgram(self.rendering_program);
      gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
  }

  fn shutdown(&mut self) {
    unsafe {
      gl::DeleteVertexArrays(1, &self.vertex_array_object);
      gl::DeleteProgram(self.rendering_program);
    }
  }
}

fn main() {
  let mut app = MyApplication::default();
  app.run();
}
