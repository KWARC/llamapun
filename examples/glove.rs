//! Our glove implementation takes a directory of HTML files, views them as a word token stream,
//! and executes the glove algorithm on top

extern crate llamapun;
extern crate time;

use std::env;
use time::PreciseTime;
use llamapun::data::Corpus;
use llamapun::glove::*;

fn main() {
  // Prepare input directory
  let args: Vec<_> = env::args().collect();
  let directory = if args.len() > 1 {
    args[1].clone()
  } else {
    ".".to_string()
  };
  // Initializing
  let start_example = PreciseTime::now();
  let mut doc_count = 0;
  let mut word_count = 0;

  // Train the GloVe model, as we traverse the word stream of an HTML corpus
  let mut corpus = Corpus::new(directory);
  for mut document in corpus.iter() {
    println!("Document: {:?}", document.path);

    for mut paragraph in document.iter() {
      for mut sentence in paragraph.iter() {    
        for _word in sentence.iter() {
          word_count += 1;
        }
      }
    }

    // TODO: Remove me after dev is done.
    doc_count+=1;
    if doc_count > 20 { // limit, fast prototyping here
      break;
    }
  }
  
  let end_parse = PreciseTime::now();
  println!("Trained a GloVe model on {:?} tokens in {:?}ms", word_count, start_example.to(end_parse).num_milliseconds());
}