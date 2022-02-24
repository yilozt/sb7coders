use crate::prelude::*;

mod data;
use data::generate_tex;

#[derive(Clone, Copy)]
enum TexFilter {
  Nearst = gl::NEAREST as _,
  Linear = gl::LINEAR as _,
}

impl Default for TexFilter {
  #[inline(always)]
  fn default() -> Self { Self::Linear }
}

impl TexFilter {
  #[inline(always)]
  fn apply_filter(&self,gl: &gl, tex: Option<&WebGlTexture>) {
    let filter = match self {
      a@TexFilter::Nearst => *a as isize,
      a@TexFilter::Linear => *a as isize,
    };

    gl.bind_texture(gl::TEXTURE_2D, tex);
    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as _);
    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as _);
  }
}

#[derive(Default)]
pub struct App {
  tex_obj: Option<WebGlTexture>,
  prog:    Option<WebGlProgram>,
  vao:     Option<WebGlVertexArrayObject>,
  vbo:     Option<WebGlBuffer>,
  mat:     Option<WebGlUniformLocation>,
}

impl Application for App {
  fn startup(&mut self, gl: &gl) {
    let vs_src = "#version 300 es
      precision mediump float;

      in vec2 pos;
      out vec2 tc;

      uniform mat4 mat = mat4(1.0);

      void main() {
        gl_Position = mat * vec4(pos, 0.0, 1.0);
        tc = pos + vec2(0.5);
        tc.y = -tc.y;
      }
    ";
    let fs_src = "#version 300 es
      precision mediump float;

      uniform sampler2D tex;

      in vec2 tc;
      out vec4 color;

      void main() {
        color = texture(tex, tc);
      }
    ";
    
    self.prog = program::link_from_shaders(gl, &[
      shader::load(gl, vs_src, gl::VERTEX_SHADER).as_ref(),
      shader::load(gl, fs_src, gl::FRAGMENT_SHADER).as_ref(),
    ], true);

    self.mat = gl.get_uniform_location(self.prog.as_ref().unwrap(), "mat");
    gl.use_program(self.prog.as_ref());

    let vertices = [
       0.5f32,  0.5f32,
       0.5f32, -0.5f32,
      -0.5f32,  0.5f32,

      -0.5f32,  0.5f32,
       0.5f32, -0.5f32,
      -0.5f32, -0.5f32,
    ];

    self.vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.vao.as_ref());

    self.vbo = gl.create_buffer();
    gl.bind_buffer(gl::ARRAY_BUFFER, self.vbo.as_ref());
    gl.buffer_data_with_array_buffer_view(gl::ARRAY_BUFFER,
                      &unsafe { js_sys::Object::from(js_sys::Float32Array::view(&vertices)) },
                      gl::STATIC_DRAW);

    gl.vertex_attrib_pointer_with_i32(0, 2, gl::FLOAT, false, (2 * size_of::<f32>()) as _, 0 as _);
    gl.enable_vertex_attrib_array(0);

    let (width, height, _, data) = generate_tex();
    self.tex_obj = gl.create_texture();
    gl.bind_texture(gl::TEXTURE_2D, self.tex_obj.as_ref());
    gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, width as _, height as _);
    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(gl::TEXTURE_2D, 0, 0, 0, width as _, height as _, gl::RGBA, gl::UNSIGNED_BYTE, Some(& unsafe { js_sys::Uint8Array::view(&data).value_of() })).unwrap();
  }

  fn render(&self, gl: &gl, current_time: f64) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);

    let aspect = {
      let AppConfig {width, height, ..} = self.info();
      width as f32 / height as f32
    };

    let current_time = (current_time * 30.0) as f32;

    for (pos, filter) in [(-0.6f32, TexFilter::Linear), (0.6f32, TexFilter::Nearst)] {
      let mat: Mat4 = perspective(45.0, aspect, 0.1, 1000.0)
      * translate(pos, 0.0, -3.0)
      * rotate_with_axis(current_time, 0.0, 1.0, 0.0);

      gl.bind_vertex_array(self.vao.as_ref());
      gl.uniform_matrix4fv_with_f32_sequence(self.mat.as_ref(), false, &unsafe { js_sys::Float32Array::view_mut_raw(std::ptr::addr_of!(mat) as _, 16).into() });

      filter.apply_filter(gl, self.tex_obj.as_ref());
        
      gl.draw_arrays(gl::TRIANGLES, 0, 6);
    }
  }

  fn shutdown(&mut self, gl: &web_sys::WebGl2RenderingContext) {
    gl.delete_texture(self.tex_obj.as_ref());
    gl.delete_vertex_array(self.vao.as_ref());
    gl.delete_buffer(self.vbo.as_ref());
    gl.delete_program(self.prog.as_ref());
  }
}
