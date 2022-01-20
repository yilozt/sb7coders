use sb7::{vec3, vec4, vmath::*};

#[test]
fn test_ops() {
  // Add and sub
  assert_eq!(vec3!(0.0, 1.0, 2.0) + vec3!(2.0, 1.0, 0.0), vec3!(2.0, 2.0, 2.0));
  assert_eq!(vec4!(0.0, 1.0, 2.0, 3.0) + vec4!(3.0, 2.0, 1.0, 0.0), vec4!(3.0, 3.0, 3.0, 3.0));
  assert_eq!(vec3!(0.0, 1.0, 2.0) - vec3!(2.0, 1.0, 0.0), vec3!(-2.0, 0.0, 2.0));

  // dot
  assert_eq!(
    vec4!(0.0, 1.0, 2.0, 3.0).dot(vec4!(3.0, 2.0, 1.0, 0.0)),
    0.0 * 3.0 + 1.0 * 2.0 + 2.0 * 1.0 + 3.0 * 0.0
  );

  // length
  assert_eq!(
    vec4!(1.0, 2.0, 3.0, 4.0).length(),
    ((1.0 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0 + 4.0 * 4.0) as f32).sqrt()
  );

  // angle
  assert_eq!(
    vec3!(1.0, 1.0, 1.0).angle(vec3!(2.0, 3.0, 4.0)),
    ((2.0 + 3.0 + 4.0) as f32
      / (((1.0 + 1.0 + 1.0) as f32).sqrt() * ((2.0 * 2.0 + 3.0 * 3.0 + 4.0 * 4.0) as f32).sqrt()))
    .acos()
  );
  assert_eq!(vec3!(1.0, 0.0, 0.0).angle(vec3!(0.0, 1.0, 0.0)), std::f32::consts::PI * 0.5);

  // cross
  assert_eq!(vec3!(1.0, 0.0, 0.0).cross(vec3!(0.0, 1.0, 0.0)), vec3!(0.0, 0.0, 1.0));

  // reflect
  assert_eq!(reflect(vec3!(-1.0, -1.0, -1.0), vec3!(0.0, 1.0, 0.0)), vec3!(-1.0, 1.0, -1.0));

  assert_eq!(vec3!(1.0, 2.0, 3.0) * 2.0, vec3!(2.0, 4.0, 6.0));

  assert_eq!(vec3!(1, 1, 1).length(), 3.0f32.sqrt());
}
