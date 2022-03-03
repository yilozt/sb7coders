use image::EncodableLayout;

use crate::prelude::*;

#[derive(Default)]
#[derive(Clone)]
pub struct App {
  vao: Option<WebGlVertexArrayObject>,
  vbo: Option<WebGlBuffer>,
  prog: Option<WebGlProgram>,
  texs: [Option<WebGlTexture>; 6],
  samplers: [Option<WebGlSampler>; 2],
  uniform_trans: Option<WebGlUniformLocation>,
  aspect: f32,
}

impl Application for App {
  fn startup(&mut self, gl: &web_sys::WebGl2RenderingContext) {
    let vs_src = "#version 300 es
    precision mediump float;

    uniform mat4 trans;

    layout (location = 0) in vec3 position;
    layout (location = 1) in vec2 tc;

    out vec2 tex_tc;
    out float face_index;

    void main() {
      gl_Position = trans * vec4(position, 1.0);
      tex_tc = vec2(tc.x, 1.0 - tc.y);
      face_index = float(gl_VertexID / 6);
    }
    ";

    let fs_src = "#version 300 es
    precision mediump float;
    
    in vec2 tex_tc;
    in float face_index;
    out vec4 color;

    uniform sampler2D s[6];

    void main() {
      switch(int(face_index)) {
        case 0: color = texture(s[0], tex_tc); break;
        case 1: color = texture(s[1], tex_tc); break;
        case 2: color = texture(s[2], tex_tc); break;
        case 3: color = texture(s[3], tex_tc); break;
        case 4: color = texture(s[4], tex_tc); break;
        case 5: color = texture(s[5], tex_tc); break;
        default: color = vec4(1.0); break;
      }
    }
    ";

    let vertex_position : &[f32]= &[
      // position        // tc
      -0.5, -0.5, -0.5,  0.0, 0.0,
       0.5, -0.5, -0.5,  1.0, 0.0,
       0.5,  0.5, -0.5,  1.0, 1.0,
       0.5,  0.5, -0.5,  1.0, 1.0,
      -0.5,  0.5, -0.5,  0.0, 1.0,
      -0.5, -0.5, -0.5,  0.0, 0.0,

      -0.5, -0.5,  0.5,  0.0, 0.0,
       0.5, -0.5,  0.5,  1.0, 0.0,
       0.5,  0.5,  0.5,  1.0, 1.0,
       0.5,  0.5,  0.5,  1.0, 1.0,
      -0.5,  0.5,  0.5,  0.0, 1.0,
      -0.5, -0.5,  0.5,  0.0, 0.0,

      -0.5,  0.5,  0.5,  1.0, 0.0,
      -0.5,  0.5, -0.5,  1.0, 1.0,
      -0.5, -0.5, -0.5,  0.0, 1.0,
      -0.5, -0.5, -0.5,  0.0, 1.0,
      -0.5, -0.5,  0.5,  0.0, 0.0,
      -0.5,  0.5,  0.5,  1.0, 0.0,

       0.5,  0.5,  0.5,  1.0, 0.0,
       0.5,  0.5, -0.5,  1.0, 1.0,
       0.5, -0.5, -0.5,  0.0, 1.0,
       0.5, -0.5, -0.5,  0.0, 1.0,
       0.5, -0.5,  0.5,  0.0, 0.0,
       0.5,  0.5,  0.5,  1.0, 0.0,

      -0.5, -0.5, -0.5,  0.0, 1.0,
       0.5, -0.5, -0.5,  1.0, 1.0,
       0.5, -0.5,  0.5,  1.0, 0.0,
       0.5, -0.5,  0.5,  1.0, 0.0,
      -0.5, -0.5,  0.5,  0.0, 0.0,
      -0.5, -0.5, -0.5,  0.0, 1.0,

      -0.5,  0.5, -0.5,  0.0, 1.0,
       0.5,  0.5, -0.5,  1.0, 1.0,
       0.5,  0.5,  0.5,  1.0, 0.0,
       0.5,  0.5,  0.5,  1.0, 0.0,
      -0.5,  0.5,  0.5,  0.0, 0.0,
      -0.5,  0.5, -0.5,  0.0, 1.0
    ];

    self.prog = program::link_from_shaders(gl, &[
      shader::load(gl, vs_src, gl::VERTEX_SHADER).as_ref(),
      shader::load(gl, fs_src, gl::FRAGMENT_SHADER).as_ref(),
    ], true);

    self.uniform_trans = gl.get_uniform_location(self.prog.as_ref().unwrap(), "trans");

    self.vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.vao.as_ref());

    self.vbo = gl.create_buffer();
    gl.bind_buffer(gl::ARRAY_BUFFER, self.vbo.as_ref());
    gl.buffer_data_with_array_buffer_view(gl::ARRAY_BUFFER, &unsafe { js_sys::Float32Array::view(vertex_position).into() }, gl::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(0, 3, gl::FLOAT, false, 5 * 4, 0);
    gl.vertex_attrib_pointer_with_i32(1, 2, gl::FLOAT, false, 5 * 4, 3 * 4);
    gl.enable_vertex_attrib_array(0);
    gl.enable_vertex_attrib_array(1);

    let asserts: [&[u8]; 6] = [
      include_bytes!("assert/assert_1.png"),
      include_bytes!("assert/assert_2.png"),
      include_bytes!("assert/assert_3.png"),
      include_bytes!("assert/assert_4.png"),
      include_bytes!("assert/assert_5.png"),
      include_bytes!("assert/assert_6.png"),
    ];

    for i in 0..2 {
      self.samplers[i] = gl.create_sampler();
    }
    gl.sampler_parameteri(self.samplers[0].as_ref().unwrap(), gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
    gl.sampler_parameteri(self.samplers[1].as_ref().unwrap(), gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

    gl.use_program(self.prog.as_ref());
    for i in 0..6 {
      let (w, h, data) = match image::load_from_memory(asserts[i]) {
        Ok(image::DynamicImage::ImageRgba8(img)) => (img.width(), img.height(), Vec::from(img.as_bytes())),
        unsupport @ _ => {log!("{:?}", unsupport); unreachable!();}
      };
  
      let location = gl.get_uniform_location(self.prog.as_ref().unwrap(), &format!("s[{}]", i));

      // bind to texture unit i
      gl.active_texture(gl::TEXTURE0 + i as u32);
      gl.uniform1i(location.as_ref(), i as _);

      self.texs[i] = gl.create_texture();
      gl.bind_texture(gl::TEXTURE_2D, self.texs[i].as_ref());
      gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, w as _, h as _);
      gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(gl::TEXTURE_2D, 0, 0, 0, w as _, h as _, gl::RGBA, gl::UNSIGNED_BYTE, &data[..], 0).unwrap();
      gl.bind_sampler(i as _, self.samplers[i % 2].as_ref());
    }

    gl.use_program(self.prog.as_ref());
    gl.enable(gl::DEPTH_TEST);

    let AppConfig { width, height, .. } = self.info();
    self.aspect = width as f32 / height as f32;
  }

  fn render(&self, gl: &web_sys::WebGl2RenderingContext, current_time: f64) {
    let t = current_time as f32 * 40.0;
    let AppConfig { width, height, .. } = self.info();
    let trans = perspective(45.0, width as f32 / height as f32, 0.1, 1000.0)
      * translate(0.0, 0.0, -2.5)
      * rotate(t, t, t);

    gl.uniform_matrix4fv_with_f32_sequence(self.uniform_trans.as_ref(), false, & unsafe { js_sys::Float32Array::view_mut_raw(addr_of!(trans) as _, 16).into()});
    
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.clear_depth(1.0);
    gl.clear(gl::DEPTH_BUFFER_BIT);

    gl.draw_arrays(gl::TRIANGLES, 0, 36);
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_vertex_array(self.vao.as_ref());
    gl.delete_buffer(self.vbo.as_ref());
    gl.delete_program(self.prog.as_ref());
    for i in &self.samplers {
      gl.delete_sampler(i.as_ref());
    }
    for i in &self.texs {
      gl.delete_texture(i.as_ref());
    }
  }
}
