// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
use llamapun::util::token_model;
use std::env;
use std::error::Error;
use std::time::Instant;

/// Given a `CorTeX` corpus of HTML5 documents, extract a token model as a
/// single file
pub fn main() -> Result<(), Box<dyn Error>> {
  let start = Instant::now();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string(),
  };
  let token_model_filepath = match input_args.next() {
    Some(path) => path,
    None => "token_model.txt".to_string(),
  };

  let discard_math = match input_args.next() {
    Some(value) => match value.as_str() {
      "discard_math" => true, // should eventually become --discard_math flag, rushing for now.
      _ => false,
    },
    None => false,
  };

  let corpus_counts = token_model::extract(corpus_path, token_model_filepath, discard_math)?;

  let duration_sec = start.elapsed().as_secs();
  println!("---");
  println!("Token model finished in {:?}s, gathered: ", duration_sec);
  println!(
    "{:?} documents;",
    corpus_counts.get("document_count").unwrap_or(&0)
  );
  println!(
    "{:?} paragraphs;",
    corpus_counts.get("paragraph_count").unwrap_or(&0)
  );
  println!(
    "{:?} discarded paragraphs (long words)",
    corpus_counts.get("overflow_count").unwrap_or(&0)
  );
  println!("{:?} words;", corpus_counts.get("word_count").unwrap_or(&0));
  println!(
    "{:?} numeric literals;",
    corpus_counts.get("num_count").unwrap_or(&0)
  );
  println!(
    "{:?} formulas;",
    corpus_counts.get("formula_count").unwrap_or(&0)
  );
  println!(
    "{:?} inline cites;",
    corpus_counts.get("citation_count").unwrap_or(&0)
  );
  Ok(())
}
