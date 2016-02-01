extern crate llamapun;
use llamapun::data::{Corpus};

#[test]
fn can_iterate_corpus() {
  let mut corpus = Corpus::new(".".to_string());
  for mut document in corpus.iter() {
    for paragraph in document.iter() {
      // for sentence in paragraph.iter() {
      //   for word in sentence.iter() {
      //     assert!(!word.get_plaintext().is_empty());
      //   }
      // }
    }
  }
}