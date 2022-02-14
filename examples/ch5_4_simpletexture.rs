use std::{ffi::CString, ptr::{null, null_mut}};

use sb7::application::*;
use gl::*;

#[derive(Default)]
struct App {
  texture: u32,
  prog: u32,
  vao: u32,
}

impl App {
  fn generate_texture(&self, data: &mut [f32], width: usize, height: usize) {
    assert_eq!(data.len(), width * height * 4);
    for y in 0..height {
      for x in 0..width {
        data[(y * width + x) * 4 + 0] = ((x & y) & 0xFF) as f32 / 255.0;
        data[(y * width + x) * 4 + 1] = ((x | y) & 0xFF) as f32 / 255.0;
        data[(y * width + x) * 4 + 2] = ((x ^ y) & 0xFF) as f32 / 255.0;
        data[(y * width + x) * 4 + 3] = 1.0;
      }
    }
  }

  fn log_info(&self, obj: u32, log_type: u32) {
    let mut buf = [0u8; 2048];

    unsafe {
      match log_type {
        COMPILE_STATUS => GetShaderInfoLog(obj, 2048, null_mut(), buf.as_mut_ptr() as _),
        LINK_STATUS => GetProgramInfoLog(obj, 2048, null_mut(), buf.as_mut_ptr() as _),
        _ => (),
      };  
    }

    
    let str = std::str::from_utf8(&buf).unwrap_or("invaild utf-8 str");
    println!("{}", str);
  }
}

impl Application for App {
  fn startup(&mut self) {
    unsafe {
      let mut texture = 0;

      // 创建纹理
      CreateTextures(TEXTURE_2D, 1, &mut texture);

      // 分配空间
      TextureStorage2D(texture,   // 要分配空间的纹理对象
                       1,         // 分级细化等级
                       RGBA32F,   // 数据格式
                       256, 256); // 纹理宽、高

      // 绑定纹理目标
      BindTexture(TEXTURE_2D, texture);

      // 在堆上分配空间，这段内存会在离开作用域时自动释放
      let mut data = Box::new([0f32; 256 * 256 * 4]);

      // generate_texture 函数用来向 data 填充数据
      self.generate_texture(&mut data[..], 256, 256);

      // 将生成的数据写入到纹理对象
      TextureSubImage2D(texture,
                        0,        // 细节等级，等级0代表基本图形级别
                        0, 0,     // 偏移量 0, 0
                        256, 256, // 宽 x 高
                        RGBA,     // 四通道数据
                        FLOAT,    // 数据类型为浮点数
                        data.as_ptr() as _);

      self.texture = texture;
    }

    let vs_src = "
      #version 460 core
      void main(void) {
        const vec4 vertices[] = vec4[](vec4( 0.75, -0.75, 0.5, 1.0),
                                       vec4(-0.75, -0.75, 0.5, 1.0),
                                       vec4( 0.75,  0.75, 0.5, 1.0));
        gl_Position = vertices[gl_VertexID];
      }
    ";
    let vs_src = CString::new(vs_src).unwrap();

    let fs_src = "
      #version 460 core
      uniform sampler2D s;
      out vec4 color;
      void main(void) {
        color = texture(s, gl_FragCoord.xy / textureSize(s, 0));
      }
    ";
    let fs_src = CString::new(fs_src).unwrap();
    
    unsafe {
      let vs = CreateShader(VERTEX_SHADER);
      ShaderSource(vs, 1, &vs_src.as_ptr(), null());
      CompileShader(vs);
      self.log_info(vs, COMPILE_STATUS);

      let fs = CreateShader(FRAGMENT_SHADER);
      ShaderSource(fs, 1, &fs_src.as_ptr(), null());
      CompileShader(fs);
      self.log_info(fs, COMPILE_STATUS);

      let prog = CreateProgram();
      AttachShader(prog, vs);
      AttachShader(prog, fs);
      LinkProgram(prog);
      self.log_info(prog, LINK_STATUS);

      DeleteShader(vs);
      DeleteShader(fs);

      UseProgram(prog);
      self.prog = prog;
    }

    unsafe {
      let mut vao = 0;
      CreateVertexArrays(1, &mut vao);
      BindVertexArray(vao);
      self.vao = vao;
    }
  }

  fn render(&self, _current_time: f64) {
    unsafe {
      ClearBufferfv(COLOR, 0, [0.0f32, 0.25, 0.0, 1.0].as_ptr());
      DrawArrays(TRIANGLES, 0, 3);
    }
  }

  fn shutdown(&mut self) {
    unsafe {
      DeleteProgram(self.prog);
      DeleteTextures(1, &self.texture);  
      DeleteVertexArrays(1, &self.vao);
    }
  }
}

fn main() {
  App::default().run();
}
