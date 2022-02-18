pub mod application;
pub mod vmath;

mod examples;

pub mod prelude {
  pub use crate::application::*;
  pub use crate::vmath::*;
  pub use crate::{vec3, vec4, mat, mat2, mat3, mat4};

  pub use web_sys::WebGl2RenderingContext as gl;
  pub use web_sys::WebGlBuffer;
  pub use web_sys::WebGlProgram;
  pub use web_sys::WebGlVertexArrayObject;
  pub use web_sys::WebGlTexture;

  pub use std::ptr::addr_of;
  pub use std::mem::size_of;
  pub use std::mem::size_of_val;
}
