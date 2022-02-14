use std::{ffi::CString, fmt::Display};

enum Obj {
  Shader(u32),
  Program(u32),
}

impl Obj {
  fn check_errors(self) {
    super::gl! {
      use std::ptr::null_mut;
      use Obj::{Program, Shader};
      let mut success = gl::FALSE as _;
      match self {
        Shader(s) => gl::GetShaderiv(s, gl::COMPILE_STATUS, &mut success),
        Program(p) => gl::GetProgramiv(p, gl::LINK_STATUS, &mut success),
      }

      if success == gl::TRUE as _ {
        return;
      }

      let mut info = [0u8; 1024];
      let infoptr = info.as_mut_ptr() as _;
      match self {
        Shader(shader) => gl::GetShaderInfoLog(shader, 1024, null_mut(), infoptr),
        Program(program) => gl::GetProgramInfoLog(program, 1024, null_mut(), infoptr),
      }

      println!("======== {} ========", self);
      use std::str::from_utf8 as from;
      let lines = from(&info).unwrap_or("Invaild utf-8 log")
                             .lines()
                             .map(|s| s.trim_matches('\0'))
                             .filter(|s| s.len() != 0);
      for i in lines {
        println!("== {}", i);
      }
    }
  }
}

impl Display for Obj {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      Obj::Shader(s) => {
        let mut shader_type = 0;
        super::gl! {
          gl::GetShaderiv(*s, gl::SHADER_TYPE, &mut shader_type);
        }

        let t = match shader_type as u32 {
          gl::VERTEX_SHADER => "VERTEX_SHADER",
          gl::FRAGMENT_SHADER => "FRAGMENT_SHADER",
          gl::TESS_CONTROL_SHADER => "TESS_CONTROL_SHADER",
          gl::TESS_EVALUATION_SHADER => "TESS_EVALUATION_SHADER",
          gl::GEOMETRY_SHADER => "GEOMETRY_SHADER",
          gl::COMPUTE_SHADER => "COMPUTE_SHADER",
          _ => "ERR|Unknown Shader Type",
        };
        write!(f, "{} [{}]", t, s)
      }
      Obj::Program(p) => write!(f, "Shader Program [{}]", p),
    }
  }
}

pub mod shader {
  use super::*;

  pub fn load(filename: &str, shader_type: u32, check_errors: bool) -> u32 {
    let str = std::fs::read_to_string(filename).unwrap_or("".into());
    from_str(&str, shader_type, check_errors)
  }

  pub fn from_str(source: &str, shader_type: u32, check_errors: bool) -> u32 {
    crate::gl! {
      let shader = gl::CreateShader(shader_type);
      let source = CString::new(source).unwrap();
      gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
      gl::CompileShader(shader);

      if check_errors {
        Obj::Shader(shader).check_errors();
      }

      return shader;
    }
  }
}

pub mod program {
  use super::*;

  pub fn link_from_shaders(shaders: &[u32], delete_shaders: bool) -> u32 {
    crate::gl! {
      let program = gl::CreateProgram();
      for &shader in shaders {
        gl::AttachShader(program, shader);
        if delete_shaders {
          gl::DeleteShader(shader);
        }
      }
      gl::LinkProgram(program);
      Obj::Program(program).check_errors();
      program
    }
  }
}
