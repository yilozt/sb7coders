use crate::application::*;
use web_sys::WebGl2RenderingContext as gl;

#[derive(Default)]
pub struct App {
  vao:  Option<web_sys::WebGlVertexArrayObject>,
  prog: Option<web_sys::WebGlProgram>,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - Vertex Attributes",
                ..Default::default() }
  }

  fn startup(&mut self, gl: &gl) {
    let vs_source = "#version 300 es
      precision mediump float;

      layout (location = 0) in vec4 offset;

      void main(void) {
        const vec4 vertices[3] = vec4[3](vec4(0.25, -0.25, 0.5, 1.0),
                                          vec4(-0.25, -0.25, 0.5, 1.0),
                                          vec4(0.25, 0.25, 0.5, 1.0));
        gl_Position = vertices[gl_VertexID] + offset;
      }
    ";

    let fs_source = "#version 300 es
      precision mediump float;

      out vec4 color;
      void main(void) {
        color = vec4(1.0);
      }
    ";

    let program = gl.create_program();

    let vs = gl.create_shader(gl::VERTEX_SHADER).unwrap();
    gl.shader_source(&vs, vs_source);
    gl.compile_shader(&vs);

    let fs = gl.create_shader(gl::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&fs, fs_source);
    gl.compile_shader(&fs);

    gl.attach_shader(program.as_ref().unwrap(), &vs);
    gl.attach_shader(program.as_ref().unwrap(), &fs);
    gl.link_program(program.as_ref().unwrap());

    gl.delete_shader(Some(&vs));
    gl.delete_shader(Some(&fs));

    let vao = gl.create_vertex_array();
    gl.bind_vertex_array(vao.as_ref());

    self.vao = vao;
    self.prog = program;
  }

  fn render(&self, gl: &gl, current_time: f64) {
    let current_time = current_time as f32;
    let attrib = [current_time.sin() * 0.5, current_time.cos() * 0.6, 0.0, 0.0];

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);

    gl.use_program(self.prog.as_ref());

    gl.vertex_attrib4fv_with_f32_array(0, &attrib);

    gl.draw_arrays(gl::TRIANGLES, 0, 3);
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_program(self.prog.as_ref());
    gl.delete_vertex_array(self.vao.as_ref());
  }
}
