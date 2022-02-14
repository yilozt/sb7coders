use std::mem::size_of;

#[test]
fn run() {
  #[derive(Default)]
  #[repr(C)]
  struct A {
    a: u32,
    b: u32,
  }

  // alloc in stack
  let a = A::default();
  assert!((&a as *const _ as usize) < (&a.b as *const _ as usize));
  assert_eq!((&a.b as *const _ as usize) - (&a as *const _ as usize), size_of::<u32>());

  // alloc in heap
  let a: Box<A> = Default::default();
  assert!((a.as_ref() as *const _ as usize) < (&a.b as *const _ as usize));
  assert_eq!((&a.b as *const _ as usize) - (&a.a as *const _ as usize), size_of::<u32>());
}
