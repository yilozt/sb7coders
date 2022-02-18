use crate::prelude::*;

#[derive(Default)]
pub struct App {
  texture: Option<WebGlTexture>,
  prog:    Option<WebGlProgram>,
  vao:     Option<WebGlVertexArrayObject>,
}

impl App {
  fn generate_texture(&self, data: &mut [u8], width: usize, height: usize) {
    assert_eq!(data.len(), width * height * 4);
    for y in 0..height {
      for x in 0..width {
        data[(y * width + x) * 4 + 0] = ((x & y) & 0xFF) as u8;
        data[(y * width + x) * 4 + 1] = ((x | y) & 0xFF) as u8;
        data[(y * width + x) * 4 + 2] = ((x ^ y) & 0xFF) as u8;
        data[(y * width + x) * 4 + 3] = 0xFF;
      }
    }
  }
}

impl Application for App {
  fn startup(&mut self, gl: &gl) {
    self.texture = gl.create_texture();
    gl.bind_texture(gl::TEXTURE_2D, self.texture.as_ref());

    gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, 256, 256);

    // 在堆上分配空间，这段内存会在离开作用域时自动释放
    let mut data = Box::new([0u8; 256 * 256 * 4]);

    // generate_texture 函数用来向 data 填充数据
    self.generate_texture(&mut data[..], 256, 256);

    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(gl::TEXTURE_2D, 0, 0, 0, 256, 256, gl::RGBA, gl::UNSIGNED_BYTE, data.as_slice(), 0).unwrap();

    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
    gl.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

    let vs_src = "#version 300 es
      precision mediump float;
      void main(void) {
        const vec4 vertices[] = vec4[](vec4( 0.75, -0.75, 0.5, 1.0),
                                       vec4(-0.75, -0.75, 0.5, 1.0),
                                       vec4( 0.75,  0.75, 0.5, 1.0));
        gl_Position = vertices[gl_VertexID];
      }
    ";

    let fs_src = "#version 300 es
      precision mediump float;
      uniform sampler2D s;
      out vec4 color;
      void main(void) {
        color = texture(s, gl_FragCoord.xy / vec2(textureSize(s, 0)));
      }
    ";

    let vs = gl.create_shader(gl::VERTEX_SHADER).unwrap();
    gl.shader_source(&vs, vs_src);
    gl.compile_shader(&vs);

    let fs = gl.create_shader(gl::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&fs, fs_src);
    gl.compile_shader(&fs);

    self.prog = gl.create_program();
    gl.attach_shader(self.prog.as_ref().unwrap(), &vs);
    gl.attach_shader(self.prog.as_ref().unwrap(), &fs);
    gl.link_program(self.prog.as_ref().unwrap());

    gl.delete_shader(Some(&vs));
    gl.delete_shader(Some(&fs));

    gl.use_program(self.prog.as_ref());

    self.vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.vao.as_ref());
  }

  fn render(&self, gl: &gl, _current_time: f64) {
    gl.clear_color(0.0, 0.25, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.draw_arrays(gl::TRIANGLES, 0, 3);
  }

  fn shutdown(&mut self, gl: &gl) {
    gl.delete_program(self.prog.as_ref());
    gl.delete_texture(self.texture.as_ref());
    gl.delete_vertex_array(self.vao.as_ref());
  }
}