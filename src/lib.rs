pub mod application;
pub mod ktx;
pub mod object;
pub mod vmath;
pub mod color;

mod prog;
pub use prog::program;
pub use prog::shader;

#[macro_export]
macro_rules! gl {
  ($($x: tt)*) => {
    unsafe { $($x)* }
  };
}

pub mod prelude {
  pub use crate::application::*;
  pub use crate::gl;
  pub use crate::ktx;
  pub use crate::program;
  pub use crate::shader;
  pub use crate::object::*;
  pub use crate::vmath::*;
  pub use crate::color;
  pub use crate::{vec3, vec4, mat, mat2, mat3, mat4};
  
  pub use imgui_glfw_rs::glfw;
  pub use imgui_glfw_rs::imgui;
  
  pub use gl::*;

  pub use std::mem::size_of;
  pub use std::mem::size_of_val;
  pub use std::ptr::null;
  pub use std::ptr::null_mut;
  pub use std::ptr::addr_of;
  pub use std::ptr::addr_of_mut;
}
