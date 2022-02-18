use crate::prelude::*;

#[derive(Default)]
pub struct App {
  vao: Option<WebGlVertexArrayObject>,
  buf: Option<WebGlBuffer>,
  program: Option<WebGlProgram>,
}

impl Application for App {
  fn startup(&mut self, gl: &gl) {
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

    self.vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.vao.as_ref());

    self.buf = gl.create_buffer();
    gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.as_ref());
    unsafe {
      gl.buffer_data_with_array_buffer_view(gl::ARRAY_BUFFER,
        &js_sys::Float32Array::view(vertex_position),
        gl::STATIC_DRAW);
    }
    gl.vertex_attrib_pointer_with_i32(0, 3, gl::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    let vs_source = "#version 300 es
      precision mediump float;

      in vec4 position;
      
      out vec4 fs_in;

      uniform mat4 mv_matrix;
      uniform mat4 proj_matrix;

      void main() {
        gl_Position =  proj_matrix * mv_matrix * position;
        fs_in = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
      }
    ";
    let vs = gl.create_shader(gl::VERTEX_SHADER).unwrap();
    gl.shader_source(vs.as_ref(), vs_source);
    gl.compile_shader(vs.as_ref());
        
    let fs_source = "#version 300 es
      precision mediump float;

      out vec4 color;
      
      in vec4 fs_in;

      void main() {
        color = fs_in;
      }
    ";
    let fs = gl.create_shader(gl::FRAGMENT_SHADER).unwrap();
    gl.shader_source(fs.as_ref(), &fs_source);
    gl.compile_shader(fs.as_ref());

    let program = gl.create_program();
    gl.attach_shader(program.as_ref().unwrap(), &vs);
    gl.attach_shader(program.as_ref().unwrap(), &fs);
    gl.link_program(program.as_ref().unwrap());
    gl.delete_shader(Some(&vs));
    gl.delete_shader(Some(&fs));

    gl.use_program(program.as_ref());
    self.program = program;

    gl.enable(gl::DEPTH_TEST);
  }

  fn render(&self, gl: &gl, current_time: f64) {
    let current_time = current_time as f32;

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    let AppConfig { width, height, .. }= self.info();
    let proj_matrix = perspective(50.0, width as f32 / height as f32, 0.01, 1000.0);
    unsafe {
      let location = gl.get_uniform_location(self.program.as_ref().unwrap(), "proj_matrix");
      gl.uniform_matrix4fv_with_f32_sequence(location.as_ref(), false, &js_sys::Float32Array::view_mut_raw(addr_of!(proj_matrix) as _, 4 * 4));  
    }


    for i in 0..24 {
      let f = i as f32 + current_time * 0.3;

      let mv_matrix = translate(0.0, 0.0, -6.0)
            * rotate_with_axis(current_time * 45.0, 0.0, 1.0, 0.0)
            * rotate_with_axis(current_time * 21.0, 1.0, 0.0, 0.0)
            * translate((2.1 * f).sin() * 2.0,
                        (1.7 * f).cos() * 2.0,
                        (1.3 * f).sin() * (1.5 * f).cos() * 2.0);
      unsafe {
        let location = gl.get_uniform_location(self.program.as_ref().unwrap(), "mv_matrix");
        gl.uniform_matrix4fv_with_f32_sequence(location.as_ref(), false, &js_sys::Float32Array::view_mut_raw(addr_of!(mv_matrix) as _, 4 * 4));
      }
      gl.draw_arrays(gl::TRIANGLES, 0, 36);
    }
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_buffer(self.buf.as_ref());
    gl.delete_vertex_array(self.vao.as_ref());
    gl.delete_program(self.program.as_ref());
  }
}
