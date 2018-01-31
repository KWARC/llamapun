extern crate llamapun;
use llamapun::stopwords;

#[test]
fn can_load_stopwords() {
  let stopwords = stopwords::load();
  assert!(stopwords.len() > 100);
}

#[test]
fn can_use_stopwords() {
  let stopwords = stopwords::load();
  assert!(stopwords.contains("the"));
  assert!(stopwords.contains("about"));
  assert!(!stopwords.contains("equation"));
}
