extern crate llamapun;
use llamapun::data::{Corpus};

#[test]
fn can_iterate_corpus() {
  let mut corpus = Corpus::new(".".to_string());
  let mut word_count = 0;
  for mut document in corpus.iter() {
    for mut paragraph in document.iter() {
      for mut sentence in paragraph.iter() {
        for word in sentence.iter() {
          word_count+=1;
          assert!(! word.text.is_empty());
        }
      }
    }
  }
  println!("Words iterated on: {:?}", word_count);
  assert!(word_count > 1500);
}