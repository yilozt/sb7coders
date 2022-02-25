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


def_exam!(default,                _default);
def_exam!(ch3_1_vertexattr,       _ch3_1_vertexattr);
def_exam!(ch3_2_transdata,        _ch3_2_transdata);
def_exam!(ch5_1_vao,              _ch5_1_vao);
def_exam!(ch5_2_spinningcube,     _ch5_2_spinningcube);
def_exam!(ch5_3_spinningcubes,    _ch5_3_spinningcubes);
def_exam!(ch5_4_simpletexture,    _ch5_4_simpletexture);
def_exam!(ch5_5_simpletexcoords,  _ch5_5_simpletexcoords);
def_exam!(ch5_6_texturefilter,    _ch5_6_texturefilter);
def_exam!(ch5_7_tunnel,           _ch5_7_tunnel);
def_exam!(ch5_8_wrapmodes,        _ch5_8_wrapmodes);
def_exam!(ch5_9_mirrorclampedge,  _ch5_9_mirrorclampedge);
def_exam!(ch5_10_alienrain,       _ch5_10_alienrain);