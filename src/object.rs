// Copyright � 2012-2015 Graham Sellers
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::{io::Read, mem::size_of, ptr::null};

pub mod sb6m {
  macro_rules! fourcc {
    ($a: expr, $b: expr, $c: expr, $d: expr) => {
      ($a as u32) << 0 | ($b as u32) << 8 | ($c as u32) << 16 | ($d as u32) << 24
    };
  }

  #[inline(always)]
  pub fn magic() -> u32 {
    fourcc!('S', 'M', '6', 'M')
  }

  #[allow(non_snake_case)]
  pub mod ChunkType {
    pub const INDEX_DATA: u32      = fourcc!('I', 'N', 'D', 'X');
    pub const VERTEX_DATA: u32     = fourcc!('V', 'R', 'T', 'X');
    pub const VERTEX_ATTRIBS: u32  = fourcc!('A', 'T', 'R', 'B');
    pub const SUB_OBJECT_LIST: u32 = fourcc!('O', 'L', 'S', 'T');
    pub const COMMENT: u32         = fourcc!('C', 'M', 'N', 'T');
    pub const DATA: u32            = fourcc!('D', 'A', 'T', 'A');
  }

  #[repr(C)]
  #[derive(Clone)]
  pub struct Header {
    pub magic:      u32,
    pub size:       u32,
    pub num_chunks: u32,
    pub flags:      u32,
  }

  #[repr(C)]
  #[derive(Clone)]
  pub struct ChunkHeader {
    pub chunk_type: u32,
    pub size:       u32,
  }

  #[repr(C)]
  #[derive(Clone)]
  pub struct ChunkIndexData {
    pub header:            ChunkHeader,
    pub index_type:        u32,
    pub index_count:       u32,
    pub index_data_offset: u32,
  }

  #[repr(C)]
  #[derive(Clone)]
  pub struct ChunkVertexData {
    pub header:         ChunkHeader,
    pub data_size:      u32,
    pub data_offset:    u32,
    pub total_vertices: u32,
  }

  #[repr(C)]
  #[derive(Clone)]
  pub struct VertexAttribDecl {
    pub name:        [u8; 64],
    pub size:        u32,
    pub data_type:   u32,
    pub stride:      u32,
    pub flags:       u32,
    pub data_offset: u32,
  }

  pub const VERTEX_ATTRIB_FLAG_NORMALIZED: u32 = 0x00000001;
  pub const VERTEX_ATTRIB_FLAG_INTEGER: u32 = 0x00000002;

  #[repr(C)]
  #[derive(Clone)]
  pub struct VertexAttribChunk {
    pub header:       ChunkHeader,
    pub attrib_count: u32,
    pub attrib_data:  [VertexAttribDecl; 1],
  }

  pub enum DataEncoding {
    DataEncodingRaw = 0,
  }

  #[repr(C)]
  #[derive(Clone)]
  pub struct DataChunk {
    pub header:      ChunkHeader,
    pub encoding:    u32,
    pub data_offset: u32,
    pub data_length: u32,
  }

  #[repr(C)]
  #[derive(Clone, Copy)]
  pub struct SubObjectDecl {
    pub first: u32,
    pub count: u32,
  }

  #[repr(C)]
  #[derive(Clone)]
  pub struct ChunkSubObjectList {
    pub header:     ChunkHeader,
    pub count:      u32,
    pub sub_object: [SubObjectDecl; 1],
  }

  #[repr(C)]
  pub struct ChunkComment {
    header:  ChunkHeader,
    comment: u8,
  }
}

#[derive(Default)]
pub struct Object {
  data_buf:        u32,
  vao:             u32,
  index_type:      u32,
  index_offset:    u32,
  num_sub_objects: u32,
  sub_object:      Vec<sb6m::SubObjectDecl>,
}

impl Object {
  #[inline(always)]
  pub fn render(&self) {
    self.render_objects(0, 1, 0);
  }

  #[inline(always)]
  pub fn render_sub_object(&self, object_index: usize) {
    self.render_sub_objects(object_index, 1, 0);
  }

  #[inline(always)]
  pub fn render_objects(&self,
                        object_index: usize,
                        instance_count: u32,
                        base_instance: u32) {
    self.render_sub_objects(object_index, instance_count, base_instance);
  }

  pub fn render_sub_objects(&self,
                            object_index: usize,
                            instance_count: u32,
                            base_instance: u32) {
    crate::gl! {
      gl::BindVertexArray(self.vao);

      if self.index_type != gl::NONE {
        gl::DrawElementsInstancedBaseInstance(gl::TRIANGLES,
                                              self.sub_object[object_index].count as _,
                                              self.index_type,
                                              self.sub_object[object_index].first as _,
                                              instance_count as _,
                                              base_instance);
      } else {
        gl::DrawArraysInstancedBaseInstance(gl::TRIANGLES,
                                            self.sub_object[object_index].first as _,
                                            self.sub_object[object_index].count as _,
                                            instance_count as _,
                                            base_instance);
      }
    }
  }

  pub fn get_sub_object_info(&self, index: usize) -> (u32, u32) {
    if index > self.num_sub_objects as usize {
      (0, 0)
    } else {
      (self.sub_object[index].first, self.sub_object[index].count)
    }
  }

  #[inline(always)]
  pub fn get_sub_object_count(&self) -> u32 {
    self.num_sub_objects
  }

  #[inline(always)]
  pub fn get_vao(&self) -> u32 {
    self.vao
  }

  pub fn load(&mut self, filename: &str) {
    self.free();

    let mut data = Vec::new();
    std::fs::File::open(filename).unwrap()
                                 .read_to_end(&mut data)
                                 .unwrap();

    let mut offset = 0;

    let header: &sb6m::Header = data.load(offset).unwrap();
    offset += header.size as usize;

    let mut vertex_attrib_chunk: Option<&sb6m::VertexAttribChunk>  = None;
    let mut vertex_data_chunk:   Option<&sb6m::ChunkVertexData>    = None;
    let mut index_data_chunk:    Option<&sb6m::ChunkIndexData>     = None;
    let mut sub_object_chunk:    Option<&sb6m::ChunkSubObjectList> = None;
    let mut data_chunk:          Option<&sb6m::DataChunk>          = None;

    use sb6m::ChunkType::*;
    for _ in 0..header.num_chunks {
      let chunk: &sb6m::ChunkHeader = data.load(offset).unwrap();
      let chunk_offset = offset;
      offset += chunk.size as usize;

      match chunk.chunk_type {
        VERTEX_ATTRIBS  => vertex_attrib_chunk = data.load(chunk_offset),
        VERTEX_DATA     => vertex_data_chunk   = data.load(chunk_offset),
        INDEX_DATA      => index_data_chunk    = data.load(chunk_offset),
        SUB_OBJECT_LIST => sub_object_chunk    = data.load(chunk_offset),
        DATA            => data_chunk          = data.load(chunk_offset),
        _ => {}
      }
    }

    crate::gl! {
      gl::GenVertexArrays(1, &mut self.vao);
      gl::BindVertexArray(self.vao);
    }

    if let Some(chunk) = data_chunk {
      crate::gl!{
        gl::GenBuffers(1, &mut self.data_buf);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.data_buf);
        gl::BufferData(gl::ARRAY_BUFFER, chunk.data_length as _, data.chunk_at(chunk, chunk.data_offset, chunk.data_length), gl::STATIC_DRAW);  
      }
    } else {
      let mut data_size = 0;
      let mut size_used = 0;

      if let Some(chunk) = vertex_data_chunk {
        data_size += chunk.data_size;
      }

      if let Some(chunk) = index_data_chunk {
        data_size += chunk.index_count * match chunk.index_type { gl::UNSIGNED_SHORT => size_of::<u16>() , _ => size_of::<u8>() } as u32;
      }

      crate::gl! {
        gl::GenBuffers(1, &mut self.data_buf);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.data_buf);
        gl::BufferData(gl::ARRAY_BUFFER, data_size as _, null(), gl::STATIC_DRAW);
      }

      if let Some(chunk) = vertex_data_chunk {
        crate::gl!(gl::BufferSubData(gl::ARRAY_BUFFER, 0, chunk.data_size as _, data.at(chunk.data_offset as _) as _));
        size_used += chunk.data_offset;
      }

      if let Some(chunk) = index_data_chunk {
        crate::gl!(gl::BufferSubData(gl::ARRAY_BUFFER,
                          size_used as _,
                          (chunk.index_count as usize
                            * match chunk.index_type { gl::UNSIGNED_SHORT => size_of::<u16>() , _ => size_of::<u8>()}) as _,
                          data.at(chunk.index_data_offset)));
      }
    }

    if let Some(chunk) = vertex_attrib_chunk {
      for i in 0..chunk.attrib_count as usize {
        let decl = data.load_decl(&chunk.attrib_data, i);
        crate::gl!{
          gl::VertexAttribPointer(i as _,
                                  decl.size as _,
                                  decl.data_type,
                                  match decl.flags & sb6m::VERTEX_ATTRIB_FLAG_NORMALIZED { 0 => gl::FALSE, _ => gl::TRUE },
                                  decl.stride as _,
                                  decl.data_offset as _);
          gl::EnableVertexAttribArray(i as _);
        }
      }  
    }

    if let Some(chunk) = index_data_chunk.as_ref() {
      crate::gl!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.data_buf));
      self.index_type = chunk.index_type;
      self.index_offset = chunk.index_data_offset;
    } else {
      self.index_type = gl::NONE;
    }

    if let Some(chunk) = sub_object_chunk {
      for i in 0..chunk.count as usize {
        self.sub_object.push(data.load_decl(&chunk.sub_object, i));
      }

      self.num_sub_objects = chunk.count;
    } else {
      let decl = sb6m::SubObjectDecl {
        first: 0,
        count: match self.index_type { gl::NONE => vertex_data_chunk.unwrap().total_vertices, _ => index_data_chunk.unwrap().index_count }
      };
      self.sub_object.push(decl);
      self.num_sub_objects = 1;
    }

    crate::gl! {
      gl::BindVertexArray(0);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);  
    }
  }

  pub fn free(&mut self) {
    crate::gl! {
      gl::DeleteVertexArrays(1, &self.vao);
      gl::DeleteBuffers(1, &self.data_buf);
    }

    self.vao = 0;
    self.data_buf = 0;
    self.sub_object.clear();
  }
}

trait LoadChunk<T: Clone>: Check + std::ops::Deref<Target = [u8]> {
  #[inline(always)]
  fn load(&self, offset: usize) -> Option<&T> {
    let end = offset + size_of::<T>();
    assert!(end < self.len());

    unsafe {
      Some(&*(&self[offset] as *const _ as *const T))
    }
  }

  #[inline(always)]
  fn load_decl(&self, decl: &[T; 1], index: usize) -> T {
    unsafe {
      let head = (decl as *const T).add(index);
      let tail = head.add(1);
      self.assert_contain((head as _, tail as _));
      (*head).clone()
    }
  }

  #[inline(always)]
  fn chunk_at(&self, chunk: &T, offset: u32, data_len: u32) -> *const std::ffi::c_void {
    unsafe {
      let head = (chunk as *const T as *const u8).add(offset as _);
      let tail = head.add(data_len as _);
      self.assert_contain((head, tail));
      head as _
    }
  }
}

trait Check: std::ops::Deref<Target = [u8]> {
  #[inline(always)]
  fn assert_contain(&self, range: (*const u8, *const u8)) {
    let bounds = (self.first().unwrap() as *const u8, self.last().unwrap() as *const u8);

    assert!(range.0.le(&range.1));
    assert!((range.0 as *const u8).ge(&bounds.0));
    assert!((range.1 as *const u8).le(&bounds.1));
  }
}

trait Ptr: std::ops::Deref<Target = [u8]> {
  #[inline(always)]
  fn at(&self, offset: u32) -> *const std::ffi::c_void {
    (&self[offset as usize]) as *const _ as _
  }
}

impl<T: Clone> LoadChunk<T> for Vec<u8> {}
impl Ptr for Vec<u8> {}
impl Check for Vec<u8> {}