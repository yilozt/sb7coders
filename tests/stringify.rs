#[test]
fn test_stringify() {
  assert_eq!(stringify!(1 + 1), "1 + 1");

  assert_eq!(string("aaaaaaaaaaaaaaaaaa"), "a");
  assert_eq!(string("123456"), "a");
}

fn string<T>(_a: T) -> &'static str {
    stringify!(_a)
}
