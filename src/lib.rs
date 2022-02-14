pub mod application;
pub mod ktx;
pub mod object;
pub mod vmath;

mod prog;
pub use prog::program;
pub use prog::shader;

#[macro_export]
macro_rules! gl {
  ($($x: tt)*) => {
    unsafe { $($x)* }
  };
}
