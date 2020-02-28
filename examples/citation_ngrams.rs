// Example run over arXMLiv 08.2019:
// ```
// cargo run --release --example citation_ngrams \
//    /data/datasets/embeddings-arXMLiv-08-2019/token_model_no_problem.txt
//    /data/datasets/embeddings-arXMLiv-08-2019/token_model_warning_1.txt
//    /data/datasets/embeddings-arXMLiv-08-2019/token_model_warning_2.txt
//    /data/datasets/embeddings-arXMLiv-08-2019/token_model_error.txt
extern crate llamapun;
extern crate time;

use llamapun::ngrams::{Ngrams};
use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufWriter, BufReader};
use time::PreciseTime;
use serde::Serialize;

static BUFFER_CAPACITY: usize = 10_485_760;
#[derive(Debug, Serialize)]
struct HeadingRecord<'a> {
  ngram: &'a str,
  frequency: usize,
}


fn main() -> Result<(), Box<dyn Error>> {
  let start_example = PreciseTime::now();
  let mut ngrams = Ngrams {
    n: 4,
    window_size: 15,
    anchor: Some("citationelement".to_string()),
    counts: HashMap::new()
  };

  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  while let Some(file_path) = input_args.next() {
    eprintln!("-- opening {:?}", file_path);
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut accum : usize = 0;
    for line in reader.lines() {
      let content = line?;
      if content.contains("citationelement") {
        ngrams.add_content(&content);
        accum += 1;
        if accum % 100_000 == 0 {
          println!("-- examined {} lines", accum);
        }
      }
    }
  }
  let end_example = PreciseTime::now();
  let ngrams_file = File::create(format!("{}_grams_{}_window.csv", ngrams.n, ngrams.window_size))?;
  let buffered_writer = BufWriter::with_capacity(BUFFER_CAPACITY, ngrams_file);
  let mut csv_writer = csv::Writer::from_writer(buffered_writer);
  for (ngram, frequency) in ngrams.sorted() {
    csv_writer.serialize(HeadingRecord { ngram, frequency })?;
  }
  csv_writer.flush()?;
  eprintln!(
    "    citation ngram extraction took {:?}ms",
    start_example.to(end_example).num_milliseconds()
  );
  Ok(())
}
