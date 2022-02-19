pub mod shader {
  use crate::prelude::*;

  pub fn load(gl: &gl, source: &str, shader_type: u32) -> Option<WebGlShader> {
    let shader = gl.create_shader(shader_type);
    gl.shader_source(shader.as_ref().unwrap(), source);
    gl.compile_shader(shader.as_ref().unwrap());

    return shader;
  }
}

pub mod program {
  use crate::prelude::*;

  pub fn link_from_shaders(gl: &gl,
                           shaders: &[Option<&WebGlShader>],
                           delete_shaders: bool)
                           -> Option<WebGlProgram> {
    let program = gl.create_program();
    for &shader in shaders {
      gl.attach_shader(program.as_ref().unwrap(), shader.unwrap());
      if delete_shaders {
        gl.delete_shader(shader);
      }
    }
    gl.link_program(program.as_ref().unwrap());
    program
  }
}
