// Copyright 2015-2019 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
/// Extracts a corpus heading model from an unpacked corpus of HTML files
/// With math lexemes (default):
/// $ cargo run --release --example corpus_heading_stats /path/to/corpus heading_data.tar
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Error};

use libxml::xpath::Context;
use llamapun::parallel_data::*;
use llamapun::util::data_helpers;
use serde::Serialize;

static BUFFER_CAPACITY: usize = 10_485_760;

#[derive(Debug, Serialize)]
struct HeadingRecord<'a> {
  heading: &'a str,
  frequency: &'a u64,
}

pub fn main() -> Result<(), Error> {
  let start = time::get_time();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string(),
  };
  let headings_report_filename = match input_args.next() {
    Some(path) => path,
    None => "headings_report_filename.csv".to_string(),
  };

  let corpus = Corpus::new(corpus_path);

  let mut catalog = corpus.catalog_with_parallel_walk(|document| {
    let mut heading_count: u64 = 0;
    let mut overflow_count = 0;
    let mut thread_counts = HashMap::new();
    thread_counts.insert(String::from("total_document_count"), 1);

    let mut context = Context::new(&document.dom).unwrap();

    'headings: for mut heading in document.heading_iter() {
      // Before we go into tokenization, ensure this is an English sentence on the math-normalized plain text.
      if data_helpers::invalid_for_english_latin(&heading.dnm) {
        continue 'headings;
      }
      let mut heading_buffer = String::new();
      let mut invalid_heading = false;
      'sentences: for mut sentence in heading.iter() {
        let mut sentence_buffer = String::new();
        for word in sentence.word_iter() {
          if word.range.is_empty() {
            continue;
          }
          let word_string =
            match data_helpers::ams_normalize_word_range(&word.range, &mut context, false) {
              Ok(w) => w,
              Err(_) => {
                overflow_count += 1;
                invalid_heading = true;
                break 'sentences;
              }
            };
          if !word_string.is_empty() {
            sentence_buffer.push_str(&word_string);
            sentence_buffer.push(' ');
          }
        }
        if !sentence_buffer.is_empty() {
          heading_buffer.push_str(&sentence_buffer);
          heading_buffer.push(' ');
        }
      }
      // If heading was valid and contains text, record it
      if !invalid_heading && !heading_buffer.is_empty() {
        // simplify/normalize to standard names
        while heading_buffer.ends_with('\n') || heading_buffer.ends_with(' ') {
          heading_buffer.pop();
        }
        heading_count += 1;
        let this_heading_counter = thread_counts.entry(heading_buffer).or_insert(0);
        *this_heading_counter += 1;
      }
    }
    thread_counts.insert(String::from("heading_count"), heading_count);
    thread_counts.insert(String::from("overflow_count"), overflow_count);
    thread_counts
  });

  println!(
    "{:?} Total traversed documents;",
    catalog.remove("total_document_count").unwrap_or(0)
  );
  println!(
    "{:?} headings;",
    catalog.remove("heading_count").unwrap_or(0)
  );
  println!(
    "{:?} discarded headings (long words)",
    catalog.remove("overflow_count").unwrap_or(0)
  );

  let mut catalog_vec: Vec<(&String, &u64)> = catalog.iter().collect();
  catalog_vec.sort_by(|a, b| b.1.cmp(a.1));

  let heading_statistics_file = File::create(headings_report_filename)?;
  let buffered_writer = BufWriter::with_capacity(BUFFER_CAPACITY, heading_statistics_file);
  let mut csv_writer = csv::Writer::from_writer(buffered_writer);
  for (heading, frequency) in catalog_vec {
    csv_writer.serialize(HeadingRecord { heading, frequency })?;
  }
  csv_writer.flush()?;

  let end = time::get_time();
  let duration_sec = (end - start).num_milliseconds() / 1000;
  println!("---");
  println!("Headings statistics finished in {:?}s", duration_sec);

  Ok(())
}
