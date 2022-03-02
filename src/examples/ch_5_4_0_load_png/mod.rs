use image::EncodableLayout;
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::prelude::*;

#[derive(Default)]
#[derive(Clone)]
pub struct App {
  vao: Option<WebGlVertexArrayObject>,
  vbo: Option<WebGlBuffer>,
  prog: Option<WebGlProgram>,
  tex: Option<WebGlTexture>,
  uniform_trans: Option<WebGlUniformLocation>,
}

impl App {
  #[inline(always)]
  fn rotate(&self, gl: &gl, x: f32, y: f32, z: f32) {
    let AppConfig { width, height, .. } = self.info();
    let r = perspective(35.0, width as f32 / height as f32, 0.1, 1000.0)
      * lookat(vec3!(2.0, 1.0, 2.5), vec3!(0.0, 0.0, 0.0), vec3!(0.0, 1.0, 0.0))
      * rotate(x, y, z);
    gl.use_program(self.prog.as_ref());
    gl.uniform_matrix4fv_with_f32_sequence(self.uniform_trans.as_ref(), false, & unsafe { js_sys::Float32Array::view_mut_raw(addr_of!(r) as _, 16).into() })
  }
}

impl Application for App {
  fn startup(&mut self, gl: &web_sys::WebGl2RenderingContext) {
    let vs_src = "#version 300 es
    precision mediump float;

    layout (location = 0) in vec3 position;
    layout (location = 1) in vec2 tc;

    out vec2 tex_tc;

    uniform mat4 trans;

    void main() {
      gl_Position = trans * vec4(position, 1.0);
      tex_tc = vec2(1.0 - tc.x, tc.y);
    }
    ";

    let fs_src = "#version 300 es
    precision mediump float;
    
    in vec2 tex_tc;
    out vec4 color;

    uniform sampler2D s;

    void main() {
      vec4 tex_color = texture(s, tex_tc);
      color = mix(vec4(1.0), tex_color, tex_color.a);
      // color = vec4(1.0);
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
    self.rotate(gl, 0.0, 0.0, 0.0);

    self.vao = gl.create_vertex_array();
    gl.bind_vertex_array(self.vao.as_ref());

    self.vbo = gl.create_buffer();
    gl.bind_buffer(gl::ARRAY_BUFFER, self.vbo.as_ref());
    gl.buffer_data_with_array_buffer_view(gl::ARRAY_BUFFER, &unsafe { js_sys::Float32Array::view(vertex_position).into() }, gl::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(0, 3, gl::FLOAT, false, 5 * 4, 0);
    gl.vertex_attrib_pointer_with_i32(1, 2, gl::FLOAT, false, 5 * 4, 3 * 4);
    gl.enable_vertex_attrib_array(0);
    gl.enable_vertex_attrib_array(1);

    let (w, h, data) = match image::load_from_memory(include_bytes!("assert/Opengl-logo.png")) {
      Ok(image::DynamicImage::ImageRgba8(img)) => (img.width(), img.height(), Vec::from(img.as_bytes())),
      unsupport @ _ => {log!("{:?}", unsupport); unreachable!();}
    };

    self.tex = gl.create_texture();
    gl.bind_texture(gl::TEXTURE_2D, self.tex.as_ref());
    gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, w as _, h as _);
    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(gl::TEXTURE_2D, 0, 0, 0, w as _, h as _, gl::RGBA, gl::UNSIGNED_BYTE, &data[..], 0).unwrap();
  
    gl.use_program(self.prog.as_ref());
    gl.enable(gl::DEPTH_TEST);
  }

  fn render(&self, gl: &web_sys::WebGl2RenderingContext, _current_time: f64) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.clear_depth(1.0);
    gl.clear(gl::DEPTH_BUFFER_BIT);

    gl.draw_arrays(gl::TRIANGLES, 0, 36);
  }

  fn ui(&mut self, gl: std::rc::Rc<web_sys::WebGl2RenderingContext>, ui: &web_sys::Element) {
    ui.set_inner_html(r#"
    <label>Rotate_X: <input type="range" min="0" max="360" value="0" class="rotate_input"/></label><br>
    <label>Rotate_Y: <input type="range" min="0" max="360" value="0" class="rotate_input"/></label><br>
    <label>Rotate_Z: <input type="range" min="0" max="360" value="0" class="rotate_input"/></label>
    "#);

    let inputs = ui.query_selector_all("input.rotate_input").unwrap();

    let _inputs = inputs.clone();
    let _gl = gl.clone();
    let _s = self.clone();
    let closure = Closure::wrap(Box::new(move |_: web_sys::EventTarget| {
      let mut val = [0.0; 3];
      for i in 0.._inputs.length() {
        let v = _inputs.get(i).unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().value_as_number();
        val[i as usize] = v;
      }

      _s.rotate(&_gl, val[0] as f32, val[1] as f32, val[2] as f32)
    }) as Box<dyn FnMut(_)>);

    let inputs = ui.query_selector_all("input.rotate_input").unwrap();
    for i in 0..inputs.length() {
      inputs.get(i).unwrap().add_event_listener_with_callback("input", closure.as_ref().unchecked_ref()).unwrap();
    }

    closure.forget();
  }

  fn shutdown(&mut self, gl: &web_sys::WebGl2RenderingContext) {
    gl.delete_vertex_array(self.vao.as_ref());
    gl.delete_buffer(self.vbo.as_ref());
    gl.delete_program(self.prog.as_ref());
    gl.delete_texture(self.tex.as_ref());
  }
}
