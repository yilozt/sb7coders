use crate::application::Application;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlVertexArrayObject};

#[derive(Default)]
pub struct App {
  rendering_program:   Option<WebGlProgram>,
  vertex_array_object: Option<WebGlVertexArrayObject>,
}

impl App {
  fn compile_shaders(&self,
                     gl: &WebGl2RenderingContext)
                     -> Option<web_sys::WebGlProgram> {
    const VERTEX_SHADER_SOUECE: &str = "#version 300 es

      precision mediump float;

      void main() {
        const vec4 vertices[3] = vec4[3](
          vec4( 0.25, -0.25, 0.5, 1.0),
          vec4(-0.25, -0.25, 0.5, 1.0),
          vec4( 0.25,  0.25, 0.5, 1.0)
        );

        gl_Position = vertices[gl_VertexID];
      }
    ";

    const FRAGMENT_SHADER_SOUECE: &str = "#version 300 es
      precision mediump float;
      
      out vec4 color;

      void main() {
        color = vec4(0.0, 0.0, 0.0, 1.0);
      }
    ";

    let vertex_shader = gl.create_shader(web_sys::WebGl2RenderingContext::VERTEX_SHADER)
                          .unwrap();
    gl.shader_source(&vertex_shader, VERTEX_SHADER_SOUECE);
    gl.compile_shader(&vertex_shader);

    let fragment_shader =
      gl.create_shader(web_sys::WebGl2RenderingContext::FRAGMENT_SHADER)
        .unwrap();
    gl.shader_source(&fragment_shader, FRAGMENT_SHADER_SOUECE);
    gl.compile_shader(&fragment_shader);

    // 创建着色器程序
    let program = gl.create_program().unwrap();
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    // 着色器已经被编译，就不再需要了
    gl.delete_shader(Some(&vertex_shader));
    gl.delete_shader(Some(&fragment_shader));

    Some(program)
  }
}

impl Application for App {
  fn startup(&mut self, gl: &WebGl2RenderingContext) {
    self.rendering_program = self.compile_shaders(gl);
    self.vertex_array_object = gl.create_vertex_array();
    gl.bind_vertex_array(self.vertex_array_object.as_ref());
  }

  fn render(&self, gl: &WebGl2RenderingContext, current_time: f64) {
    let g = (current_time as f32).sin() * 0.5 + 0.5;
    gl.clear_color(g, g, g, 1.0);
    gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

    gl.use_program(self.rendering_program.as_ref());
    gl.draw_arrays(web_sys::WebGl2RenderingContext::TRIANGLES, 0, 3);
  }

  fn shutdown(&mut self, gl: &WebGl2RenderingContext) {
    web_sys::console::log_1(&"shutdown.".into());
    gl.delete_vertex_array(self.vertex_array_object.as_ref());
    gl.delete_program(self.rendering_program.as_ref());
  }
}
