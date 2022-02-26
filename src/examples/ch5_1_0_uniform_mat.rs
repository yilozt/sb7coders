use crate::prelude::*;

#[derive(Default)]
pub struct App {
  vao: Option<WebGlVertexArrayObject>,
  buf: Option<WebGlBuffer>,
  program: Option<WebGlProgram>,
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig {
      title: "Uniform Mat4", ..Default::default()
    }
  }

  fn startup(&mut self, gl: &gl) {
    #[allow(dead_code)]
    #[repr(C)]
    struct Vertex {
      x: f32, y: f32, z: f32, // position
      r: f32, g: f32, b: f32, // color
    }

    let vertices = [
      Vertex { x: -0.5, y: -0.5, z: 0.0, r: 1.0, g: 0.0, b: 0.0 },
      Vertex { x:  0.5, y: -0.5, z: 0.0, r: 0.0, g: 1.0, b: 0.0 },
      Vertex { x:  0.0, y:  0.5, z: 0.0, r: 0.0, g: 0.0, b: 1.0 },
    ];
  
    self.vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.vao.as_ref());
      
    self.buf = gl.create_buffer();
    gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.as_ref());

    unsafe {
      let vertices_arr = js_sys::Float32Array::view_mut_raw(
        vertices.as_ptr() as _,
        3 * 6
      );
      gl.buffer_data_with_array_buffer_view(gl::ARRAY_BUFFER ,
        &vertices_arr,
        gl::STATIC_DRAW);
    }

    use  std::mem::size_of;

    gl.vertex_attrib_pointer_with_i32(0, 3, gl::FLOAT, false, 6 * size_of::<f32>() as i32, 0);
    gl.vertex_attrib_pointer_with_i32(1, 3, gl::FLOAT, false, 6 * size_of::<f32>() as i32, 3 * size_of::<f32>() as i32);
    gl.enable_vertex_attrib_array(0);
    gl.enable_vertex_attrib_array(1);

    let vs_source = "#version 300 es
      precision mediump float;
      layout (location = 0) in vec3 position;
      layout (location = 1) in vec3 color;

      uniform mat4 rotate;

      out vec4 vs_color;

      void main() {
        gl_Position = rotate * vec4(position, 1.0);
        vs_color = vec4(color, 1.0);
      }
    ";
    let vs = gl.create_shader(gl::VERTEX_SHADER).unwrap();
    gl.shader_source(&vs, vs_source);
    gl.compile_shader(&vs);
      
    let fs_source = "#version 300 es
      precision mediump float;
      in vec4 vs_color;
      out vec4 fs_color;

      void main() {
        fs_color = vs_color;
      }
    ";
    let fs = gl.create_shader(gl::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&fs, fs_source);
    gl.compile_shader(&fs);

    let program = gl.create_program();
    gl.attach_shader(program.as_ref().unwrap(), &vs);
    gl.attach_shader(program.as_ref().unwrap(), &fs);
    gl.link_program(program.as_ref().unwrap());
    gl.delete_shader(Some(&vs));
    gl.delete_shader(Some(&fs));
    gl.use_program(program.as_ref());

    self.program = program;

    gl.bind_vertex_array(self.vao.as_ref());
  }

  fn render(&self, gl: &gl, current_time: f64) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.draw_arrays(gl::TRIANGLES, 0, 3);

    let rotate = rotate_with_axis(current_time as f32 * 20.0, 0., 1., 0.);
    gl.use_program(self.program.as_ref());
    let locate = gl.get_uniform_location(self.program.as_ref().unwrap(), "rotate");
    gl.uniform_matrix4fv_with_f32_sequence(locate.as_ref(), false, &unsafe {js_sys::Float32Array::view_mut_raw(addr_of!(rotate) as _, 16).into()})
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_buffer(self.buf.as_ref());
    gl.delete_program(self.program.as_ref());
    gl.delete_vertex_array(self.vao.as_ref());
  }
}