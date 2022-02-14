use sb7::{vmath::{rotate, rotate_with_axis}, mat4};

#[test]
fn test_rotate() {
  let r = 256.26f32;
  assert_eq!(rotate_with_axis(r, 0.0, 1.0, 0.0), rotate(0.0, r, 0.0));
  let t = r.to_radians();
  assert_eq!(
      rotate(0.0, r, 0.0),mat4!(
        t.cos(), 0.0, -t.sin(), 0.0,
            0.0, 1.0,      0.0, 0.0,
        t.sin(), 0.0,  t.cos(), 0.0,
            0.0, 0.0,      0.0, 1.0,
  ));
}
