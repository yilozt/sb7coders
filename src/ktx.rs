pub mod file
{

use std::error::Error;
use std::fmt::Display;
use std::io::{ Read, Seek, SeekFrom };
use std::mem::size_of;
use std::ffi::c_void;

const IDENTIFIER: [u8; 12] =
  [0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A];

#[inline(always)]
fn swap32(n: u32) -> u32 { n.swap_bytes() }

fn calculate_stride(h: &Header, width: i32, pad: i32) -> i32
{
  let channels: i32 = match h.glbaseinternalformat
  {
    gl::RED => 1,
    gl::RG => 2,
    gl::BGR | gl::RGB => 3,
    gl::BGRA | gl::RGBA => 4,
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

pub struct KtxTex(pub u32, pub Header);

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

trait LoadHeader: std::io::Read {
  #[inline(always)]
  fn load_header(&mut self) -> Result<Header, OpenErr> {
    use OpenErr::*;
    let mut buf = [0u8; size_of::<Header>()];
    self.read(&mut buf[..]).map_err(IoErr)?;

    unsafe { Ok((*(buf.as_ptr() as *const Header)).clone()) }
  }
}

impl LoadHeader for std::fs::File {}

struct Void(*const c_void);

impl std::ops::Add<i32> for Void {
  type Output = *const c_void;
  #[inline(always)]
  fn add(self, rhs: i32) -> Self::Output {
    unsafe { (self.0).add(rhs as usize) }
  }
}

pub fn load_with_tex(filename: &str, tex: u32) -> Result<KtxTex, OpenErr>
{
  use OpenErr::*;

  let mut file = std::fs::File::open(filename).map_err(IoErr)?;

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
    if h.arrayelements == 0
    {
      gl::TEXTURE_1D
    }
    else
    {
      gl::TEXTURE_1D_ARRAY
    }
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
        gl::TEXTURE_CUBE_MAP_ARRAY
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

  let data_start = file.stream_position().map_err(IoErr)? + h.keypairbytes as u64;
  let data_end = file.seek(SeekFrom::End(0)).map_err(IoErr)?;
  file.seek(SeekFrom::Start(data_start)).map_err(IoErr)?;

  let len = (data_end - data_start) as usize;
  let mut data: Vec<u8> = Vec::with_capacity(len);
  data.resize(len, 0);

  file.read(&mut data).map_err(IoErr)?;

  if h.miplevels == 0
  {
    h.miplevels = 1;
  }

  use crate::gl;
  let mut tex = tex;
  if tex == 0
  {
    gl! { gl::GenTextures(1, &mut tex); }
  }

  gl! { gl::BindTexture(target, tex); }
  
  let data = data[..].as_ptr() as *const c_void;

  match target
  {
    gl::TEXTURE_1D =>
    gl!{
      gl::TexStorage1D(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _);
      gl::TexSubImage1D(target, 0, 0, h.pixelwidth as _, h.glformat, h.glinternalformat, data);
    },
    gl::TEXTURE_2D =>
    {
      if h.gltype == gl::NONE
      {
        gl!(gl::CompressedTexImage2D(target, 0, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _, 0, 420 * 380 / 2, data));
      }
      else
      {
        gl!(gl::TexStorage2D(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _));
        {
          let mut ptr = data;
          let mut height = h.pixelheight as i32;
          let mut width = h.pixelwidth as i32;
          gl!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1));
          for i in 0..h.miplevels
          {
              gl!(gl::TexSubImage2D(gl::TEXTURE_2D, i as _, 0, 0, width, height, h.glformat, h.gltype, ptr));
              ptr = Void(ptr) + height * calculate_stride(&h, width, 1);
              height >>= 1;
              width >>= 1;
              if height < 1 { height = 1; }
              if width < 1  { width  = 1; }
          }
        }
      }
    },
    gl::TEXTURE_3D =>
    gl!{
      gl::TexStorage3D(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _, h.pixeldepth as _);
      gl::TexSubImage3D(target, 0, 0, 0, 0, h.pixelwidth as _, h.pixelheight as _, h.pixeldepth as _, h.glformat, h.glinternalformat, data);
    },
    gl::TEXTURE_1D_ARRAY =>
    gl!{
      gl::TexStorage2D(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.arrayelements as _);
      gl::TexSubImage2D(target, 0, 0, 0, h.pixelwidth as _, h.arrayelements as _, h.glformat, h.gltype, data);
    },
    gl::TEXTURE_2D_ARRAY =>
    gl!{
      gl::TexStorage3D(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _, h.arrayelements as _);
      gl::TexSubImage3D(target, 0, 0, 0, 0, h.pixelwidth as _, h.pixelheight as _, h.arrayelements as _, h.glformat, h.gltype, data);
    },
    gl::TEXTURE_CUBE_MAP =>
    {
      gl!(gl::TexStorage2D(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _));
      // glTexSubImage3D(GL_TEXTURE_CUBE_MAP, 0, 0, 0, 0, h.pixelwidth, h.pixelheight, h.faces, h.glformat, h.gltype, data);
      {
        let face_size = calculate_face_size(&h);
        for i in 0..h.faces
        {
          let data = Void(data) + face_size * i as i32;
          gl!(gl::TexSubImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X + i, 0, 0, 0, h.pixelwidth as _, h.pixelheight as _, h.glformat, h.gltype, data as _));
        }
      }
    },
    gl::TEXTURE_CUBE_MAP_ARRAY =>
    gl!{
      gl::TexStorage3D(target, h.miplevels as _, h.glinternalformat, h.pixelwidth as _, h.pixelheight as _, h.arrayelements as _);
      gl::TexSubImage3D(target, 0, 0, 0, 0, h.pixelwidth as _, h.pixelheight as _, (h.faces * h.arrayelements) as _, h.glformat, h.gltype, data);
    },
    _ => return Err(UnSupportedTargetErr)
  }

  if h.miplevels == 1
  {
    gl!(gl::GenerateMipmap(target));
  }

  Ok(KtxTex(tex, h))
}

#[inline(always)]
pub fn load(filename: &str) -> Result<KtxTex, OpenErr> {
  load_with_tex(filename, 0)
}

}
