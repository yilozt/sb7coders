use std::ptr::addr_of;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::application::*;
use once_cell::sync::Lazy;


macro_rules! def_exam {
  ($mod: ident, $export: ident) => {
    mod $mod;

    #[allow(non_upper_case_globals)]
    static mut $mod: Lazy<$mod::App> = Lazy::new(|| $mod::App::default());

    #[wasm_bindgen]
    #[allow(non_camel_case_types)]
    pub struct $export;

    #[wasm_bindgen]
    impl $export {
      pub fn run() {
        unsafe { $mod.run(addr_of!($mod) as _) };
      }
      pub fn stop() {
        unsafe { $mod.close_app(addr_of!($mod) as _) };
      }
    }
  };
}

def_exam!(default, _default);
def_exam!(ch2_main, _ch2_main);
def_exam!(ch3_1_vertexattr, _ch3_1_vertexattr);
def_exam!(ch3_2_transdata, _ch3_2_transdata);