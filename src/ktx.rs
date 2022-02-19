pub mod file
{

use std::error::Error;
use std::fmt::Display;
use std::mem::size_of;
use std::ffi::c_void;
use wasm_bindgen::JsCast;

use crate::prelude::*;

const IDENTIFIER: [u8; 12] =
  [0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A];

#[inline(always)]
fn swap32(n: u32) -> u32 { n.swap_bytes() }

fn calculate_stride(h: &Header, width: i32, pad: i32) -> i32
{
  web_sys::console::log_1(&format!("glbaseinternalformat: {}", h.glbaseinternalformat).into());
  let channels: i32 = match h.glbaseinternalformat
  {
    gl::RED => 1,
    gl::RG => 2,
    gl::RGB | 32992 => 3,
    gl::RGBA | 32993 => 4,
    _ => 0
  };

  let stride = h.gltypesize as i32 * channels * width;

  (stride + (pad - 1)) & !(pad - 1)
}

fn calculate_face_size(h: &Header) -> i32
{
  let stride = calculate_stride(&h, h.pixelwidth as _, 4);

  stride * h.pixelheight as i32
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct Header
{
  identifier:               [u8; 12],
  endianness:               u32,
  pub gltype:               u32,
  pub gltypesize:           u32,
  pub glformat:             u32,
  pub glinternalformat:     u32,
  pub glbaseinternalformat: u32,
  pub pixelwidth:           u32,
  pub pixelheight:          u32,
  pub pixeldepth:           u32,
  pub arrayelements:        u32,
  pub faces:                u32,
  pub miplevels:            u32,
  keypairbytes:             u32,
}

pub struct KtxTex(pub Option<WebGlTexture>, pub Header);

#[derive(Debug)]
pub enum OpenErr
{
  IoErr(std::io::Error),
  HeaderErr,
  UnSupportedTargetErr,
}

impl Display for OpenErr
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self
    {
      Self::IoErr(err) => write!(f, "IoErr: {}", err),
      Self::HeaderErr => write!(f, "Header of file doesn't match"),
      Self::UnSupportedTargetErr => write!(f, "Unkonwn texture target type")
    }
  }
}

impl Error for OpenErr
{
  fn cause(&self) -> Option<&dyn std::error::Error>
  {
    match self
    {
      Self::IoErr(e) => e.source(),
      _ => Some(self),
    }
  }
}

trait LoadHeader: std::convert::AsRef<[u8]> {
  #[inline(always)]
  fn load_header(&self) -> Result<Header, OpenErr> {
    assert!(size_of::<Header>() < self.as_ref().len(), "header is out of bound in file");
    let header = &self.as_ref()[0..size_of::<Header>()];
    
    unsafe { Ok((*(header.as_ptr() as *const Header)).clone()) }
  }
}

impl LoadHeader  for [u8] {}

struct Void(*const c_void);

impl std::ops::Add<i32> for Void {
  type Output = *const c_void;
  #[inline(always)]
  fn add(self, rhs: i32) -> Self::Output {
    unsafe { (self.0).add(rhs as usize) }
  }
}

/// Example:
/// 
/// ```rust
/// load_with_tex(&gl, include_bytes!("media/textures/aliens.ktx"), tex).unwrap();
/// ```
pub fn load_with_tex(gl: &gl, file: &[u8], tex: Option<WebGlTexture>) -> Result<KtxTex, OpenErr>
{
  use OpenErr::*;

  let mut h = file.load_header()?;

  if h.identifier != IDENTIFIER
  {
    return Err(HeaderErr);
  }

  match h.endianness
  {
    // No swap needed
    0x04030201 => {}

    // Swap needed
    0x01020304 => {
      h.endianness           = swap32(h.endianness);
      h.gltype               = swap32(h.gltype);
      h.gltypesize           = swap32(h.gltypesize);
      h.glformat             = swap32(h.glformat);
      h.glinternalformat     = swap32(h.glinternalformat);
      h.glbaseinternalformat = swap32(h.glbaseinternalformat);
      h.pixelwidth           = swap32(h.pixelwidth);
      h.pixelheight          = swap32(h.pixelheight);
      h.pixeldepth           = swap32(h.pixeldepth);
      h.arrayelements        = swap32(h.arrayelements);
      h.faces                = swap32(h.faces);
      h.miplevels            = swap32(h.miplevels);
      h.keypairbytes         = swap32(h.keypairbytes);
    }
    _ => return Err(OpenErr::HeaderErr),
  };

  // Guess target (texture type)
  let target = if h.pixelheight == 0
  {
    return Err(OpenErr::UnSupportedTargetErr);
  }
  else if h.pixeldepth == 0
  {
    if h.arrayelements == 0
    {
      if h.faces == 0
      {
        gl::TEXTURE_2D
      }
      else
      {
        gl::TEXTURE_CUBE_MAP
      }
    }
    else
    {
      if h.faces == 0
      {
        gl::TEXTURE_2D_ARRAY
      }
      else
      {
        // gl::TEXTURE_CUBE_MAP_ARRAY
        return Err(OpenErr::UnSupportedTargetErr);
      }
    }
  }
  else
  {
    gl::TEXTURE_3D
  };

  // Check for insanity...
  if target == gl::NONE ||                      // Couldn't figure out target
      h.pixelwidth == 0 ||                      // Texture has no width???
      (h.pixelheight == 0 && h.pixeldepth != 0) // Texture has depth but no height???
  {
    return Err(OpenErr::HeaderErr);
  }

  let data_start = size_of::<Header>() + h.keypairbytes as usize;
  let data = &file[data_start..];

  if h.miplevels == 0
  {
    h.miplevels = 1;
  }

  let mut tex = tex;
  if tex.is_none()
  {
    tex = gl::create_texture(gl);
  }

  gl.bind_texture(target, tex.as_ref());

  let array_buf_from = |gltype, data: &[u8]| -> js_sys::Object {
    unsafe {
      match gltype {
        gl::UNSIGNED_BYTE => js_sys::Uint8Array::view_mut_raw(data.as_ptr() as _, data.len()).dyn_into().unwrap(),
        gl::FLOAT => js_sys::Float32Array::view_mut_raw(data.as_ptr() as _, data.len() / size_of::<f32>()).dyn_into().unwrap(),
        _ => {
          web_sys::console::log_1(&format!("Unknown h.gltype: {}", gltype).into());
          unimplemented!();
        }
      }
    }
  };
  
  match target
  {
    gl::TEXTURE_2D =>
    {
      if h.gltype == gl::NONE
      {
        unsafe {
          let view = js_sys::Uint8Array::view(data);
          gl.compressed_tex_image_2d_with_array_buffer_view(target, 0, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _, 0, &view);
        }
      }
      else
      {
        gl.tex_storage_2d(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _);
        {
          let mut offset = 0;
          let mut height = h.pixelheight as i32;
          let mut width = h.pixelwidth as i32;
          gl.pixel_storei(gl::UNPACK_ALIGNMENT, 1);
          for i in 0..h.miplevels
          {
            gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(gl::TEXTURE_2D, i as _, 0, 0, width, height, h.glformat, h.gltype, &array_buf_from(h.gltype, data), offset).unwrap();
            offset = (offset as i32 + height * calculate_stride(&h, width, 1)) as u32;
            height >>= 1;
            width >>= 1;
            if height < 1 { height = 1; }
            if width < 1  { width  = 1; }
          }
          gl.generate_mipmap(target);
        }
      }
    },
    gl::TEXTURE_2D_ARRAY =>
    {
      gl.tex_storage_3d(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _, h.arrayelements as _);
      gl.tex_sub_image_3d_with_opt_array_buffer_view(target, 0, 0, 0, 0, h.pixelwidth as _, h.pixelheight as _, h.arrayelements as _, h.glformat, h.gltype, Some(&array_buf_from(h.gltype, &data))).unwrap();
    },
    gl::TEXTURE_CUBE_MAP =>
    {
      gl.tex_storage_2d(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _);
      // glTexSubImage3D(GL_TEXTURE_CUBE_MAP, 0, 0, 0, 0, h.pixelwidth, h.pixelheight, h.faces, h.glformat, h.gltype, data);
      {
        let face_size = calculate_face_size(&h);
        let mut offset = 0;
        for i in 0..h.faces
        {
          offset = offset + face_size * i as i32;
          gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(gl::TEXTURE_CUBE_MAP_POSITIVE_X + i, 0, 0, 0, h.pixelwidth as _, h.pixelheight as _, h.glformat, h.gltype, &array_buf_from(h.gltype, data), offset as _).unwrap();
        }
      }
    },
    _ => return Err(UnSupportedTargetErr)
  }

  if h.miplevels == 1
  {
    gl.generate_mipmap(target);
  }

  Ok(KtxTex(tex, h))
}

#[inline(always)]
pub fn load(gl: &gl, filename: &[u8]) -> Result<KtxTex, OpenErr> {
  load_with_tex(gl, filename, None)
}

}
