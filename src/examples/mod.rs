use crate::application::*;
use wasm_bindgen::prelude::wasm_bindgen;

macro_rules! def_exam {
  ($mod: ident, $export: ident) => {
    mod $mod;

    #[wasm_bindgen]
    #[allow(non_camel_case_types)]
    pub struct $export;

    #[wasm_bindgen]
    impl $export {
      pub fn run(width: Option<u32>, height: Option<u32>, id: Option<String>) {
        $mod::App::run(Box::new($mod::App::default()), width, height, id);
      }
      pub fn stop(id: Option<String>) {
        $mod::App::close_app(id);
      }
    }
  };
}


def_exam!(default,                      _default);

// CH 3
def_exam!(ch3_1_vertexattr,             _ch3_1_vertexattr);
def_exam!(ch3_2_transdata,              _ch3_2_transdata);

// CH5
def_exam!(ch5_1_vao,                    _ch5_1_vao);
def_exam!(ch5_1_0_uniform_mat,          _ch5_1_0_uniform_mat);
def_exam!(ch5_1_1_atom_counter,         _ch5_1_1_atom_counter);
def_exam!(ch5_2_spinningcube,           _ch5_2_spinningcube);
def_exam!(ch5_3_spinningcubes,          _ch5_3_spinningcubes);
def_exam!(ch5_4_simpletexture,          _ch5_4_simpletexture);
def_exam!(ch5_5_simpletexcoords,        _ch5_5_simpletexcoords);
def_exam!(ch5_6_texturefilter,          _ch5_6_texturefilter);
def_exam!(ch5_7_tunnel,                 _ch5_7_tunnel);
def_exam!(ch5_7_0_tunnel_scintillation, _ch5_7_0_tunnel_scintillation);
def_exam!(ch5_8_wrapmodes,              _ch5_8_wrapmodes);
def_exam!(ch5_9_mirrorclampedge,        _ch5_9_mirrorclampedge);
def_exam!(ch5_10_alienrain,             _ch5_10_alienrain);