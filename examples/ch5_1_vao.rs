use gl::types::GLuint;
use sb7::application::Application;
use std::ffi::{c_void, CString};
use std::mem::{size_of_val, size_of};
use std::ptr::null;

#[derive(Default)]
struct App {
  vao: GLuint,
  buf: GLuint,
  program: GLuint
}

impl Application for App {
  fn startup(&mut self) {
    #[allow(dead_code)]
    struct Vertex {
      x: f32, y: f32, z: f32, // position
      r: f32, g: f32, b: f32, // color
    }

    let vertices = [
      Vertex { x: -0.5, y: -0.5, z: 0.0, r: 1.0, g: 0.0, b: 0.0 },
      Vertex { x:  0.5, y: -0.5, z: 0.0, r: 0.0, g: 1.0, b: 0.0 },
      Vertex { x:  0.0, y:  0.5, z: 0.0, r: 0.0, g: 0.0, b: 1.0 },
    ];
  
    sb7::gl! {
      let mut vao = 0;
      gl::CreateVertexArrays(1, &mut vao);
      
      let mut buf = 0;
      gl::CreateBuffers(1, &mut buf);
      
      gl::NamedBufferStorage(buf, size_of_val(&vertices) as isize,
                            vertices.as_ptr() as *const c_void,
                            gl::DYNAMIC_STORAGE_BIT);
      gl::VertexArrayVertexBuffer(vao, 0, buf, 0, size_of::<Vertex>() as i32);
      gl::VertexArrayAttribFormat(vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
      gl::VertexArrayAttribFormat(vao, 1, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<f32>() as u32);
      gl::VertexArrayAttribBinding(vao, 0, 0);
      gl::VertexArrayAttribBinding(vao, 1, 0);
      gl::EnableVertexArrayAttrib(vao, 0);
      gl::EnableVertexArrayAttrib(vao, 1);

      let vs_source = CString::new("
        #version 460 core
        layout (location = 0) in vec3 position;
        layout (location = 1) in vec3 color;

        out vec4 vs_color;

        void main() {
          gl_Position = vec4(position, 1.0);
          vs_color = vec4(color, 1.0);
        }
      ").unwrap();
      let vs = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
      gl::CompileShader(vs);
        
      let fs_source = CString::new("
        #version 460 core
        in vec4 vs_color;
        out vec4 fs_color;

        void main() {
          fs_color = vs_color;
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

      *self = Self { vao, program, buf };

      gl::BindVertexArray(vao);
    }
  }

  fn render(&mut self, _current_time: f64) {
    sb7::gl! {
      gl::ClearBufferfv(gl::COLOR,0, [0.0, 0.0, 0.0f32].as_ptr());
      gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
  }

  fn shutdown(&mut self) {
    sb7::gl! {
      gl::DeleteBuffers(2, &self.buf);
      gl::DeleteProgram(self.program);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

fn main() {
  App::default().run()
}
