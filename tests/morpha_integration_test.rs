//! Test that rust-morpha is integrated properly

extern crate rustmorpha;

#[test]
fn test_that_rust_morpha_is_integrated() {
  assert_eq!(
    rustmorpha::full_stem("The tilings are amazing"),
    "the tile be amaze"
  );
  rustmorpha::close();
}
