use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct VecN<T, const LEN: usize> {
  a: [T; LEN],
}
impl<T: Default + Copy, const LEN: usize> Default for VecN<T, LEN> {
  fn default() -> Self {
    Self { a: [T::default(); LEN] }
  }
}
impl<T, const LEN: usize> VecN<T, LEN> {
  pub fn new(a: [T; LEN]) -> Self {
    Self { a }
  }
}

impl<T, const LEN: usize> Index<usize> for VecN<T, LEN> {
  type Output = T;
  #[inline(always)]
  fn index(&self, index: usize) -> &Self::Output {
    &self.a[index]
  }
}
impl<T, const LEN: usize> IndexMut<usize> for VecN<T, LEN> {
  #[inline(always)]
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.a[index]
  }
}

impl<T, const LEN: usize> VecN<T, LEN>
where
  T: Copy + Add<Output = T> + Mul<Output = T> + Default,
{
  #[inline(always)]
  pub fn dot(self, other: Self) -> T {
    self
      .a
      .iter()
      .zip(other.a.iter())
      .fold(T::default(), |sum, (a, b)| sum + *a * *b)
  }

  #[inline(always)]
  pub fn length(&self) -> f32
  where
    f64: From<T>,
  {
    let squared = self
      .a
      .iter()
      .fold(T::default(), |sum, item| sum + *item * *item);
    f64::from(squared).sqrt() as f32
  }

  #[inline(always)]
  pub fn angle(self, v: Self) -> f32
  where
    f64: From<T>,
  {
    (f64::from(self.dot(v)) as f32 / (self.length() * v.length())).acos()
  }

  #[inline(always)]
  pub fn normalize(&self) -> Self
  where
    T: From<f32> + Div<Output = T>,
    f64: From<T>,
  {
    *self / T::from(self.length())
  }
}

impl<T, const LEN: usize> Add<Self> for VecN<T, LEN>
where
  T: Add<Output = T> + Default + Copy,
{
  type Output = Self;
  #[inline(always)]
  fn add(self, rhs: Self) -> Self::Output {
    let mut a: [T; LEN] = [T::default(); LEN];
    for i in 0..LEN {
      a[i] = self.a[i] + rhs.a[i]
    }
    Self { a }
  }
}

impl<T, const LEN: usize> Sub<Self> for VecN<T, LEN>
where
  T: Sub<Output = T> + Default + Copy,
{
  type Output = Self;
  #[inline(always)]
  fn sub(self, rhs: Self) -> Self::Output {
    let mut a: [T; LEN] = [T::default(); LEN];
    for i in 0..LEN {
      a[i] = self.a[i] - rhs.a[i];
    }
    Self { a }
  }
}

impl<T, const LEN: usize> Mul<T> for VecN<T, LEN>
where
  T: Mul<Output = T> + Copy + Default,
{
  type Output = Self;
  #[inline(always)]
  fn mul(self, rhs: T) -> Self::Output {
    let mut a = [T::default(); LEN];
    for i in 0..LEN {
      a[i] = self.a[i] * rhs;
    }
    Self { a }
  }
}
impl<T, const LEN: usize> Div<T> for VecN<T, LEN>
where
  T: Div<Output = T> + Copy + Default + From<f32>,
{
  type Output = Self;
  #[inline(always)]
  fn div(self, rhs: T) -> Self::Output {
    let mut a = [T::default(); LEN];
    for i in 0..LEN {
      a[i] = self.a[i] / rhs;
    }
    Self { a }
  }
}

pub type TVec3<T> = VecN<T, 3>;
pub type TVec4<T> = VecN<T, 4>;
impl<T> TVec3<T> {
  #[inline(always)]
  pub fn cross(self, other: Self) -> Self
  where
    T: Mul<Output = T> + Sub<Output = T> + Copy,
  {
    Self {
      a: [
        self.a[1] * other.a[2] - self.a[2] * other.a[1],
        self.a[2] * other.a[0] - self.a[1] * other.a[2],
        self.a[0] * other.a[1] - self.a[1] * other.a[0],
      ],
    }
  }
}

pub type Vec3 = TVec3<f32>;
pub type Vec4 = TVec4<f32>;

#[macro_export]
macro_rules! vec3 {
  ($($x:expr),+ $(,)?) => ({
    let a = $crate::vmath::TVec3::new([$($x),+]);
    a
  });
}
#[macro_export]
macro_rules! vec4 {
  ($($x:expr),+ $(,)?) => ({
    let a = $crate::vmath::TVec4::new([$($x),+]);
    a
  });
}

#[inline(always)]
pub fn reflect(rin: Vec3, n: Vec3) -> Vec3 {
  rin - n * ((n * 2.0).dot(rin))
}
#[inline(always)]
pub fn refract(rin: Vec3, n: Vec3, ir: f32) -> Vec3 {
  let k = 1.0 - ir * ir * (1.0 - (rin.dot(n)).powf(2.0));
  if k > 0. {
    rin * ir - n * (ir * n.dot(rin) + k.sqrt())
  } else {
    vec3!(0.0, 0.0, 0.0)
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MatNM<T, const W: usize, const H: usize> {
  a: [VecN<T, H>; W],
}
impl<T, const W: usize, const H: usize> Default for MatNM<T, W, H>
where
  T: Default + Copy,
{
  #[inline(always)]
  fn default() -> Self {
    Self {
      a: [VecN { a: [T::default(); H] }; W],
    }
  }
}
impl<T, const W: usize, const H: usize> Index<usize> for MatNM<T, W, H> {
  type Output = VecN<T, H>;
  #[inline(always)]
  fn index(&self, index: usize) -> &Self::Output {
    &self.a[index]
  }
}
impl<T, const W: usize, const H: usize> IndexMut<usize> for MatNM<T, W, H> {
  #[inline(always)]
  fn index_mut(&mut self, index: usize) -> &mut VecN<T, H> {
    &mut self.a[index]
  }
}

impl<T, const W: usize, const H: usize> MatNM<T, W, H>
where
  T: Default + Copy,
{
  #[inline(always)]
  pub fn new(arr: &[T]) -> Self {
    assert_eq!(arr.len(), W * H, "The length of array mast be: {} * {} = {}", W, H, W * H);
    let mut mat = MatNM::default();
    for i in 0..W {
      for j in 0..H {
        mat[i][j] = arr[W * j + i];
      }
    }
    mat
  }
}

impl<T, const W: usize, const H: usize> From<&[T]> for MatNM<T, W, H>
where
  T: Default + Copy,
{
  #[inline(always)]
  fn from(arr: &[T]) -> Self {
    Self::new(arr)
  }
}

impl<T, const X: usize, const Y: usize, const Z: usize> Mul<MatNM<T, Z, X>> for MatNM<T, X, Y>
where
  T: Default + Copy + Mul<Output = T> + AddAssign<T>,
{
  type Output = MatNM<T, Z, Y>;
  #[inline(always)]
  fn mul(self, rhs: MatNM<T, Z, X>) -> Self::Output {
    let mut res = MatNM::default();
    for i in 0..Z {
      for j in 0..Y {
        for k in 0..X {
          res[i][j] += self[k][j] * rhs[i][k];
        }
      }
    }
    res
  }
}

impl<T: Default, const X: usize, const Y: usize> Mul<VecN<T, X>> for MatNM<T, X, Y>
where
  T: Mul<Output = T> + AddAssign<T> + Copy,
{
  type Output = VecN<T, Y>;
  #[inline(always)]
  fn mul(self, rhs: VecN<T, X>) -> Self::Output {
    let mut res = VecN::default();
    for i in 0..X {
      for j in 0..Y {
        res[j] += self[i][j] * rhs[i]
      }
    }
    res
  }
}

impl<T, const X: usize> MatNM<T, X, X>
where
  T: Default + Copy + From<u8>,
{
  #[inline(always)]
  pub fn identity() -> Self {
    let mut mat = Self::default();
    for i in 0..X {
      mat[i][i] = 1.into();
    }
    mat
  }
}

pub type TMat2<T> = MatNM<T, 2, 2>;
pub type TMat3<T> = MatNM<T, 3, 3>;
pub type TMat4<T> = MatNM<T, 4, 4>;
pub type Mat2 = TMat2<f32>;
pub type Mat3 = TMat3<f32>;
pub type Mat4 = TMat4<f32>;

#[macro_export]
macro_rules! mat2 {
  () => ({
    let m = $crate::vmath::TMat2::identity();
    m
  });
  ($($x:expr),+ $(,)?) => (
      [$($x),+][..].into()
  );
}
#[macro_export]
macro_rules! mat3 {
  () => ({
      let m = $crate::vmath::TMat3::identity();
      m
  });
  ($($x:expr),+ $(,)?) => (
      [$($x),+][..].into()
  );
}

#[macro_export]
macro_rules! mat4 {
  () => ({
    let m = $crate::vmath::TMat4::identity();
    m
  });
  ($($x:expr),+ $(,)?) => (
    [$($x),+][..].into()
  );
}
#[macro_export]
macro_rules! mat {
  ([$w:expr, $h:expr] [$($x:expr),+ $(,)?]) => ({
    let a: $crate::vmath::MatNM<_, $w, $h> = [$($x),+][..].into();
    a
  })
}

#[inline(always)]
pub fn translate<T>(x: T, y: T, z: T) -> TMat4<T>
where
  T: Default + Copy + From<u8>,
{
  let mut m = mat4!();
  m[3][0] = x;
  m[3][1] = y;
  m[3][2] = z;
  m
}

#[inline(always)]
pub fn rotate_with_axis<T>(angle: T, x: T, y: T, z: T) -> TMat4<T>
where
  T: Mul<T, Output = T> + Add<T, Output = T> + Sub<T, Output = T> + Default + Copy + From<f32>,
  f64: From<T>,
{
  let x2 = x * x;
  let y2 = y * y;
  let z2 = z * z;
  let rads = (f64::from(angle) * 0.0174532925) as f32;
  let c = rads.cos();
  let s = rads.sin();
  let omc = 1.0 - c;

  TMat4 { a: [
    TVec4 {a: [ x2 * omc.into() + c.into(), y * x * omc.into() + z * s.into(), x * z * omc.into() - y * s.into(), 0.0.into() ]},
    TVec4 {a: [ x * y * omc.into() - z * s.into(), y2 * omc.into() + c.into(), y * z * omc.into() + x * s.into(), 0.0.into() ]},
    TVec4 {a: [ x * z * omc.into() + y * s.into(), y * z * omc.into() - x * s.into(), z2 * omc.into() + c.into(), 0.0.into() ]},
    TVec4 {a: [ 0.0.into(), 0.0.into(), 0.0.into(), 1.0.into(), ]},
  ] }
}

#[rustfmt::skip]
#[inline(always)]
pub fn rotate<T>(angle_x: T, angle_y: T, angle_z: T) -> TMat4<T>
where
  T: Mul<T, Output = T>
    + Add<T, Output = T>
    + Sub<T, Output = T>
    + Default
    + Copy
    + From<f32>
    + From<u8>,
  TMat4<T>: Mul<TMat4<T>, Output = TMat4<T>>,
  f64: From<T>,
{
    rotate_with_axis::<T>(angle_z, T::from(0.0), T::from(0.0), T::from(1.0))
  * rotate_with_axis::<T>(angle_y, T::from(0.0), T::from(1.0), T::from(0.0))
  * rotate_with_axis::<T>(angle_x, T::from(1.0), T::from(0.0), T::from(0.0))
}

#[inline(always)]
pub fn scale<T>(x: T, y: T, z: T) -> TMat4<T>
where
  T: Default + Copy + From<u8>,
{
  let mut m = mat4!();
  m[0][0] = x;
  m[1][1] = y;
  m[2][2] = z;
  m
}

#[inline(always)]
#[rustfmt::skip]
pub fn lookat<T>(eye: TVec3<T>, center: TVec3<T>, up: TVec3<T>) -> TMat4<T>
where
  TVec3<T>: Sub<TVec3<T>, Output = TVec3<T>>,
  f64: From<T>,
  T: Div<Output = T>
    + Add<Output = T>
    + Mul<Output = T>
    + Sub<Output = T>
    + Neg<Output = T>
    + Default
    + Copy
    + From<f32> + From<u8>,
  TMat4<T>: Mul<TMat4<T>, Output = TMat4<T>>,
  f64: From<T>,
{
  // forward
  let f = (center - eye).normalize();
  let u = up.normalize();
  // sideway
  let s = f.cross(u);
  // up
  let u = s.cross(f);

  let m: TMat4<T> = mat4![
          s[0],        s[1],         s[2],         T::from(0.0),
          u[0],        u[1],         u[2],         T::from(0.0),
         -f[0],        -f[1],        -f[2],        T::from(0.0),
         T::from(0.0), T::from(0.0), T::from(0.0), T::from(1.0)];

  m * translate(-eye[0], -eye[1], -eye[2])
}

#[inline(always)]
#[rustfmt::skip]
pub fn frustum(left: f32, right: f32, bottom: f32, top: f32, n: f32, f: f32) -> Mat4 {
  if right == left || top == bottom || n == f || n < 0.0 || f < 0.0 {
    return mat4!();
  }
  
  let mut m = Mat4::default();
  m[0][0] = (2.0 * n)      / (right - left);
  m[1][1] = (2.0 * n)      / (top - bottom);

  m[2][0] = (right + left) / (right - left);
  m[2][1] = (top + bottom) / (top - bottom);
  m[2][2] = (n + f)        / (n - f);
  m[2][3] = -1.0;

  m[3][2] = (2.0 * n * f)  / (n - f);
  m
}

/// aspect: in degress
#[inline(always)]
#[rustfmt::skip]
pub fn perspective<T>(fovy: T, aspect: T, n: T, f: T) -> TMat4<T> 
where   f64: From<T>,
T: Div<Output = T>
  + Add<Output = T>
  + Mul<Output = T>
  + Sub<Output = T>
  + Neg<Output = T>
  + Default
  + Copy
  + From<f32> + From<u8>,
f64: From<T>
{
  let q = 1.0 / (0.5 * f64::from(fovy).to_radians()).tan();
  let q: T = (q as f32).into();
  let a = q / aspect;
  let b = (n + f) / (n - f);
  let c = (T::from(2.0) * n * f) / (n - f);

  let mut result = mat4!();
  result[0].a =  [a  , 0.0.into(), 0.0.into(), 0.0.into()];
  result[1].a =  [ 0.0.into(), q  , 0.0.into(), 0.0.into()];
  result[2].a =  [ 0.0.into(), 0.0.into(), b  , (-1.0).into()];
  result[3].a =  [ 0.0.into(), 0.0.into(), c  , 0.0.into()];
  result
}

#[inline(always)]
pub fn mix<T>(a: T, b: T, t: T) -> T
where
  T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy,
{
  b + t * (b - a)
}
