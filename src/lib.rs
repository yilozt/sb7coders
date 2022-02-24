pub mod application;
pub mod vmath;
pub mod ktx;
pub mod object;
pub mod prog;

mod examples;

pub mod prelude {
  pub use crate::application::*;
  pub use crate::vmath::*;
  pub use crate::{vec3, vec4, mat, mat2, mat3, mat4};
  pub use crate::ktx;
  pub use crate::object::Object;
  pub use crate::prog::shader;
  pub use crate::prog::program;

  pub use web_sys::WebGl2RenderingContext as gl;
  pub use web_sys::WebGlBuffer;
  pub use web_sys::WebGlProgram;
  pub use web_sys::WebGlUniformLocation;
  pub use web_sys::WebGlShader;
  pub use web_sys::WebGlVertexArrayObject;
  pub use web_sys::WebGlTexture;

  pub use web_sys::console::log_1;

  pub use std::ptr::addr_of;
  pub use std::mem::size_of;
  pub use std::mem::size_of_val;

  #[macro_export]
  macro_rules! log {
    ($($t: tt)*) => {
      web_sys::console::log_1(&format!($($t)*).into())
    };
  }
  pub use crate::log;
}
