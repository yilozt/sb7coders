use std::{mem::size_of, ffi::c_void};

#[test]
fn run() {
  assert_eq!(size_of::<c_void>(), 1);
  assert_eq!(size_of::<u8>(), 1);
}