// Copyright � 2012-2015 Graham Sellers
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use sb7::prelude::*;

#[derive(Default)]
struct Textures {
  color:   u32,
  normals: u32,
}

#[derive(Default)]
struct UniformsBlock {
  mv_matrix:   Mat4,
  view_matrix: Mat4,
  proj_matrix: Mat4,
}

#[derive(Default)]
struct Uniforms {
  mvp: i32,
}

#[derive(Default)]
struct App {
  clear_program:   u32,
  append_program:  u32,
  resolve_program: u32,

  textures: Textures,

  uniforms_block:  UniformsBlock,
  uniforms_buffer: u32,

  uniforms: Uniforms,

  object: Object,

  fragment_buffer:       u32,
  head_pointer_image:    u32,
  atomic_counter_buffer: u32,
  dummy_vao:             u32,
}

impl App {
  fn load_shaders(&mut self) {
    gl! {
      for prog in [self.clear_program, self.append_program, self.resolve_program] {
        DeleteProgram(prog);
      }
    }

    self.clear_program = program::link_from_shaders(&[
      shader::load("media/shaders/fragmentlist/clear.vs.glsl", VERTEX_SHADER, true),
      shader::load("media/shaders/fragmentlist/clear.fs.glsl", FRAGMENT_SHADER, true),
    ], true);

    self.append_program = program::link_from_shaders(&[
      shader::load("media/shaders/fragmentlist/append.vs.glsl", VERTEX_SHADER, true),
      shader::load("media/shaders/fragmentlist/append.fs.glsl", FRAGMENT_SHADER, true),
    ], true);

    let mvp = std::ffi::CString::new("mvp").unwrap();
    self.uniforms.mvp = gl! { GetUniformLocation(self.append_program, mvp.as_ptr()) };

    self.resolve_program = program::link_from_shaders(&[
      shader::load("media/shaders/fragmentlist/resolve.vs.glsl", VERTEX_SHADER, true),
      shader::load("media/shaders/fragmentlist/resolve.fs.glsl", FRAGMENT_SHADER, true),
    ], true);
  }
}

impl Application for App {
  fn init(&self) -> AppConfig {
    AppConfig { title: "OpenGL SuperBible - Fragment List".into(),
                ..Default::default() }
  }

  fn startup(&mut self) {
    self.load_shaders();

    gl! {
      GenBuffers(1, &mut self.uniforms_buffer);
      BindBuffer(UNIFORM_BUFFER, self.uniforms_buffer);
      BufferData(UNIFORM_BUFFER, size_of::<UniformsBlock>() as _, null(), DYNAMIC_DRAW);

      self.object.load("media/objects/dragon.sbm");

      GenBuffers(1, &mut self.fragment_buffer);
      BindBuffer(SHADER_STORAGE_BUFFER, self.fragment_buffer);
      BufferData(SHADER_STORAGE_BUFFER, 1024 * 1024 * 16, null(), DYNAMIC_COPY);

      GenBuffers(1, &mut self.atomic_counter_buffer);
      BindBuffer(ATOMIC_COUNTER_BUFFER, self.atomic_counter_buffer);
      BufferData(ATOMIC_COUNTER_BUFFER, 4, null(), DYNAMIC_COPY);

      GenTextures(1, &mut self.head_pointer_image);
      BindTexture(TEXTURE_2D, self.head_pointer_image);
      TexStorage2D(TEXTURE_2D, 1, R32UI, 1024, 1024);

      GenVertexArrays(1, &mut self.dummy_vao);
      BindVertexArray(self.dummy_vao);
    }
  }

  fn render(&self, current_time: f64) {
    let zeros = [0.0, 0.0, 0.0, 0.0f32].as_ptr();
    let gray = [0.1, 0.1, 0.1, 0.0f32].as_ptr();
    let ones = [1.0f32].as_ptr();
    let f = current_time as f32;

    gl! {
      MemoryBarrier(SHADER_IMAGE_ACCESS_BARRIER_BIT | ATOMIC_COUNTER_BARRIER_BIT | SHADER_STORAGE_BARRIER_BIT);

      UseProgram(self.clear_program);
      BindVertexArray(self.dummy_vao);
      DrawArrays(TRIANGLE_STRIP, 0, 4);

      UseProgram(self.append_program);

      let model_matrix = scale(7.0, 7.0, 7.0);
      let view_position = vec3!((f * 0.35).cos() * 120.0, (f * 0.4).cos() * 30.0, (f * 0.35).sin() * 120.0);
      let view_matrix = lookat(view_position,
                               vec3!(0.0, 30.0, 0.0),
                               vec3!(0.0, 1.0, 0.0));

      let mv_matrix = view_matrix * model_matrix;
      let AppConfig { width, height, .. } = self.info();
      let proj_matrix = perspective(50.0, width as f32 / height as f32,
                                    0.1,
                                    1000.0);

      let mat = proj_matrix * mv_matrix;
      UniformMatrix4fv(self.uniforms.mvp, 1, FALSE, addr_of!(mat) as _);

      let zero:u32 = 0;
      BindBufferBase(ATOMIC_COUNTER_BUFFER, 0, self.atomic_counter_buffer);
      BufferSubData(ATOMIC_COUNTER_BUFFER, 0, size_of::<u32>() as _, &zero as *const u32 as _);

      BindBufferBase(SHADER_STORAGE_BUFFER, 0, self.fragment_buffer);

      BindImageTexture(0, self.head_pointer_image, 0, FALSE, 0, READ_WRITE, R32UI);

      MemoryBarrier(SHADER_IMAGE_ACCESS_BARRIER_BIT | ATOMIC_COUNTER_BARRIER_BIT | SHADER_STORAGE_BARRIER_BIT);

      self.object.render();

      MemoryBarrier(SHADER_IMAGE_ACCESS_BARRIER_BIT | ATOMIC_COUNTER_BARRIER_BIT | SHADER_STORAGE_BARRIER_BIT);

      UseProgram(self.resolve_program);

      BindVertexArray(self.dummy_vao);

      MemoryBarrier(SHADER_IMAGE_ACCESS_BARRIER_BIT | ATOMIC_COUNTER_BARRIER_BIT | SHADER_STORAGE_BARRIER_BIT);

      DrawArrays(TRIANGLE_STRIP, 0, 4);
    }
  }

  fn on_key(&mut self, key: glfw::Key, press: glfw::Action) {
    if let glfw::Action::Press = press {
      if let glfw::Key::R = key {
        self.load_shaders();
      }
    }
  }

  fn shutdown(&mut self) {
    
  }
}

fn main() {
  App::default().run();
}
