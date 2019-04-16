use llamapun::parallel_data::Corpus;
use std::collections::HashMap;

#[test]
fn can_iterate_corpus() {
  let corpus = Corpus::new("tests".to_string());
  let catalog = corpus.catalog_with_parallel_walk(|document| {
    let mut thread_count = HashMap::new();
    thread_count.insert(String::from("doc_count"), 1);
    let mut word_count = 0;
    for mut paragraph in document.paragraph_iter() {
      for mut sentence in paragraph.iter() {
        for word in sentence.simple_iter() {
          word_count += 1;
          assert!(!word.range.is_empty());
        }
      }
    }
    thread_count.insert(String::from("word_count"), word_count);
    thread_count
  });

  let word_count = catalog.get("word_count").unwrap_or(&0);
  let doc_count = catalog.get("doc_count").unwrap_or(&0);
  println!("Words iterated on: {:?}", word_count);
  assert_eq!(*doc_count, 2, "expected 2 documents, found {:?}", doc_count);
  assert!(
    *word_count > 8400,
    "expected more than 8400 words, found {:?}",
    word_count
  );
}
