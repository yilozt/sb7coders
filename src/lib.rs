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
  
  pub use imgui_glfw_rs::glfw;
  pub use imgui_glfw_rs::imgui;
  
  pub use gl::*;
}
