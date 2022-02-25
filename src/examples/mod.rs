use crate::application::*;
use once_cell::sync::Lazy;
use std::ptr::addr_of;
use wasm_bindgen::prelude::wasm_bindgen;

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

macro_rules! exams {
  ($($mod: ident, $export: ident),+ $(,)?) => {
    $(def_exam!($mod, $export);)*
  };
}

exams!(
  default,                _default,
  ch3_1_vertexattr,       _ch3_1_vertexattr,
  ch3_2_transdata,        _ch3_2_transdata,
  ch5_1_vao,              _ch5_1_vao,
  ch5_2_spinningcube,     _ch5_2_spinningcube,
  ch5_3_spinningcubes,    _ch5_3_spinningcubes,
  ch5_4_simpletexture,    _ch5_4_simpletexture,
  ch5_5_simpletexcoords,  _ch5_5_simpletexcoords,
  ch5_6_texturefilter,    _ch5_6_texturefilter,
  ch5_7_tunnel,           _ch5_7_tunnel,
  ch5_8_wrapmodes,        _ch5_8_wrapmodes,
);