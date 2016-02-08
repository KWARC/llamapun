//! Our glove implementation takes a directory of HTML files, views them as a word token stream,
//! and executes the glove algorithm on top

extern crate llamapun;
extern crate time;

use std::env;
use time::PreciseTime;
use llamapun::data::Corpus;
use llamapun::glove::Glove;

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

  // Train the GloVe model, on an HTML corpus
  let corpus = Corpus::new(directory);
  let model : Glove = Glove::train(corpus);

  let end_parse = PreciseTime::now();
  println!("Trained a GloVe model in {:?}ms", start_example.to(end_parse).num_milliseconds());
}