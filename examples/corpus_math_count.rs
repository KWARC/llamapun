//! Count the total number of <math> elements,
//! and their Content MathML annotations
//! in a directory of HTML documents
//!
//! example use for arXMLiv:
//!    `cargo run --release --example corpus_math_count /data/datasets/dataset-arXMLiv-2022`
//!
//! This script extracts the raw data from a "blind" descent over each `<math>` element, and may
//! require additional cutoffs and post-processing over uncurated corpora.
//! You can find an example of post-processing done for the data of arXMLiv here:
//! https://gist.github.com/dginev/e50a632d31be05bb87d64cc1800f6fd4#file-apply_cutoffs-pl
#![allow(clippy::unused_io_amount)]

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Error};
use std::time::Instant;

use libxml::xpath::Context;
use llamapun::parallel_data::Corpus;

static BUFFER_CAPACITY: usize = 10_485_760;

pub fn main() -> Result<(), Error> {
  let start = Instant::now();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string(),
  };
  let node_statistics_filepath = match input_args.next() {
    Some(path) => path,
    None => "corpus_math_count.csv".to_string(),
  };
  let content_statistics_filepath = match input_args.next() {
    Some(path) => path,
    None => "corpus_content_count.csv".to_string(),
  };

  let extension_filter = input_args.next();

  let node_statistics_file = File::create(node_statistics_filepath)?;
  let content_statistics_file = File::create(content_statistics_filepath)?;

  let mut corpus = Corpus::new(corpus_path);
  corpus.extension = extension_filter;

  let mut total = 0;
  let (math_catalog, content_math_catalog) = corpus.catalogs_with_parallel_walk(|document| {
    let mut math_count_hash = HashMap::new();
    let mut content_count_hash = HashMap::new();
    // just return the number of math elements
    let mut xpath_context = Context::new(&document.dom).unwrap();
    let math_count = xpath_context
      .findvalue("count(//*[local-name()='math'])", None)
      .unwrap();
    math_count_hash.insert(math_count, 1);

    let content_count = xpath_context
      .findvalue(
        "count(//*[local-name()='annotation-xml' and @encoding='MathML-Content'])",
        None,
      ).unwrap();
    content_count_hash.insert(content_count, 1);

    (math_count_hash, content_count_hash)
  });

  let duration_sec = start.elapsed().as_millis();
  eprintln!("---");
  eprintln!("Math counting finished in {:?}ms", duration_sec);

  // Report on Math.
  let mut catalog_vec: Vec<(&String, &u64)> = math_catalog.iter().collect();
  catalog_vec.sort_by(|a, b| b.1.cmp(a.1));

  let buffered_writer = BufWriter::with_capacity(BUFFER_CAPACITY, node_statistics_file);
  let mut csv_writer = csv::Writer::from_writer(buffered_writer);
  csv_writer.write_record(["math elements", "documents in corpus"])?;

  for (key, val) in catalog_vec {
    total += key.parse::<u64>().unwrap() * val;
    csv_writer.write_record([key, &val.to_string()])?;
  }
  eprintln!(" Grand total of <math> in dataset: ");
  eprintln!(" --- ");
  eprintln!(" {} ", total);
  eprintln!(" --- ");
  // Close the writer
  csv_writer.flush()?;

  // Report on Content Math.
  total = 0;
  let mut catalog_vec: Vec<(&String, &u64)> = content_math_catalog.iter().collect();
  catalog_vec.sort_by(|a, b| b.1.cmp(a.1));

  let buffered_writer = BufWriter::with_capacity(BUFFER_CAPACITY, content_statistics_file);
  let mut csv_writer = csv::Writer::from_writer(buffered_writer);
  csv_writer.write_record(["annotation-xml elements", "documents in corpus"])?;

  for (key, val) in catalog_vec {
    total += key.parse::<u64>().unwrap() * val;
    csv_writer.write_record([key, &val.to_string()])?;
  }
  eprintln!(" Grand total of Content MathML <annotation-xml> in dataset: ");
  eprintln!(" --- ");
  eprintln!(" {} ", total);
  eprintln!(" --- ");
  // Close the writer
  csv_writer.flush()

}

// Example output from arXMLiv 2022:
// Math counting finished in 14030571ms
//   Grand total of <math> in dataset:
// ---
// 970414519
// ---
// Grand total of Content MathML <annotation-xml> in dataset:
// ---
// 953308908
// ---

// Example output from ar5iv 2024:
//Math counting finished in 22121404ms
// Grand total of <math> in dataset:
// ---
// 1059794660
// ---
// Grand total of Content MathML <annotation-xml> in dataset:
// ---
// 1038882200
// ---
