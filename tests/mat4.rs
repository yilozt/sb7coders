use sb7;
use sb7::{mat, mat4};
use sb7::vec4;
use sb7::vmath::TMat4;

#[test]
fn test_new() {
  let a = mat!([3, 3] [
    1, 2, 3,
    4, 5, 6,
    7, 8, 9,
  ]);

  assert_eq!(a[0][0], 1);
  assert_eq!(a[0][1], 4);
  assert_eq!(a[0][2], 7);
  assert_eq!(a[1][0], 2);
  assert_eq!(a[1][1], 5);
  assert_eq!(a[1][2], 8);
  assert_eq!(a[2][0], 3);
  assert_eq!(a[2][1], 6);
  assert_eq!(a[2][2], 9);

  let a = mat!([4, 2][
    1, 2, 3, 4,
    5, 6, 7, 8,
  ]);
  assert_eq!(a[0][0], 1);
  assert_eq!(a[1][0], 2);
  assert_eq!(a[2][0], 3);
  assert_eq!(a[3][0], 4);
  assert_eq!(a[0][1], 5);
  assert_eq!(a[1][1], 6);
  assert_eq!(a[2][1], 7);
  assert_eq!(a[3][1], 8);
}

#[test]
fn test_mul() {
  let a = mat!([3, 2][
    1, 2, 3,
    4, 5, 6,
  ]);
  let b = mat!([4, 3][
    12, 11, 10, 9,
     8,  7,  6, 5,
     4,  3,  2, 1,
  ]);
  let res = a * b;
  assert_eq!(res[0][0], 1 * 12 + 2 * 8 + 3 * 4);
  assert_eq!(res[1][0], 1 * 11 + 2 * 7 + 3 * 3);
  assert_eq!(res[2][0], 1 * 10 + 2 * 6 + 3 * 2);
  assert_eq!(res[3][0], 1 * 09 + 2 * 5 + 3 * 1);
  assert_eq!(res[0][1], 4 * 12 + 5 * 8 + 6 * 4);
  assert_eq!(res[1][1], 4 * 11 + 5 * 7 + 6 * 3);
  assert_eq!(res[2][1], 4 * 10 + 5 * 6 + 6 * 2);
  assert_eq!(res[3][1], 4 * 09 + 5 * 5 + 6 * 1);

  let m1: TMat4<i32> =TMat4::identity();
  let m2: TMat4<i32> =TMat4::identity();
  assert_eq!(m1 * m2, m1);

  #[rustfmt::skip]
  assert_eq!(sb7::vmath::Mat4::identity(), mat4![
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
  ]);
  assert_eq!(mat4!(), sb7::vmath::Mat4::identity());

  let m = mat!([4, 2][
    1, 2, 3, 4,
    5, 6, 7, 8,
  ]);
  let v = sb7::vmath::VecN::new([4, 3, 2, 1]);
  assert_eq!(
    m * v,
    sb7::vmath::VecN::new([
      1 * 4 + 2 * 3 + 3 * 2 + 4 * 1, //
      5 * 4 + 6 * 3 + 7 * 2 + 8 * 1, //
    ])
  );
}

#[test]
fn translate() {
  let x = vec4!(1, 2, 3, 1);
  let t = sb7::vmath::translate(3, 2, 1);
  assert_eq!(t, mat4!(
    1, 0, 0, 3,
    0, 1, 0, 2,
    0, 0, 1, 1,
    0, 0, 0, 1
  ));
  assert_eq!(vec4!(1 + 3, 2 + 2, 3 + 1, 1), t * x);
}
