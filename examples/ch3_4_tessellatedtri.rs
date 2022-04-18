use std::{ffi::CString, ptr::null};

use gl::types::*;
use sb7::application::{AppConfig, Application};

#[derive(Default)]
struct MyApplication {
  program: GLuint,
  vao: GLuint,
}

impl Application for MyApplication {
  fn init(&self) -> AppConfig {
    AppConfig {
      title: "OpenGL SuperBible - Tessellated Triangle".to_string(),
      ..Default::default()
    }
  }

  fn startup(&mut self) {
    let vs_source = CString::new(
      "
      #version 460 core

      void main(void) {
        const vec4 vertices[] = vec4[](vec4( 0.25, -0.25, 0.5, 1.0),
                                       vec4(-0.25, -0.25, 0.5, 1.0),
                                       vec4( 0.25,  0.25, 0.5, 1.0));

        gl_Position = vertices[gl_VertexID];
      }
      ",
    )
    .unwrap();

    let tcs_source = CString::new(
      "
      #version 460 core

      layout (vertices = 3) out;

      void main(void) {
        if (gl_InvocationID == 0) {
          gl_TessLevelInner[0] = 5.0;
          gl_TessLevelOuter[0] = 5.0;
          gl_TessLevelOuter[1] = 5.0;
          gl_TessLevelOuter[2] = 5.0;
        }

        gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
      }
      ",
    )
    .unwrap();

    let tes_source = CString::new(
      "
      #version 460 core

      layout (triangles, equal_spacing, cw) in;

      void main(void) {
        gl_Position = (gl_TessCoord.x * gl_in[0].gl_Position)
                    + (gl_TessCoord.y * gl_in[1].gl_Position)
                    + (gl_TessCoord.z * gl_in[2].gl_Position);
      }
      ",
    )
    .unwrap();

    let fs_source = CString::new(
      "
      #version 460 core

      out vec4 color;

      void main(void) {
        color = vec4(0.0, 0.8, 1.0, 1.0);
      }
      ",
    )
    .unwrap();

    sb7::gl! {
      let program = gl::CreateProgram();

      let vs = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(vs, 1, &vs_source.as_ptr(), null());
      gl::CompileShader(vs);

      let tcs = gl::CreateShader(gl::TESS_CONTROL_SHADER);
      gl::ShaderSource(tcs, 1, &tcs_source.as_ptr(), null());
      gl::CompileShader(tcs);

      let tes = gl::CreateShader(gl::TESS_EVALUATION_SHADER);
      gl::ShaderSource(tes, 1, &tes_source.as_ptr(), null());
      gl::CompileShader(tes);

      let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(fs, 1, &fs_source.as_ptr(), null());
      gl::CompileShader(fs);

      gl::AttachShader(program, vs);
      gl::AttachShader(program, tcs);
      gl::AttachShader(program, tes);
      gl::AttachShader(program, fs);

      gl::LinkProgram(program);

      let mut vao: GLuint = 0;
      gl::GenVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);

      gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

      self.program = program;
      self.vao = vao;
    }
  }

  fn render(&mut self, _current_time: f64) {
    sb7::gl! {
      gl::ClearBufferfv(gl::COLOR, 0, [0.0, 0.25, 0.0, 1.0f32].as_ptr());

      gl::UseProgram(self.program);
      gl::DrawArrays(gl::PATCHES, 0, 3);
    }
  }

  fn shutdown(&mut self) {
    sb7::gl! {
      gl::DeleteVertexArrays(1, &self.vao);
      gl::DeleteProgram(self.program);
    }
  }
}

fn main() {
  let mut app = MyApplication::default();
  app.run();
}
