// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
extern crate libxml;
extern crate llamapun;
extern crate regex;
extern crate time;

use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::Error;

use libxml::xpath::Context;
use llamapun::ams;
use llamapun::ams::AmsEnv;
use llamapun::data::Corpus;
use llamapun::dnm;

static BUFFER_CAPACITY: usize = 10_485_760;
static MAX_WORD_LENGTH: usize = 25;

/// assume we are dealing with less than 100m items here
pub fn num_file_path(directory: &str, index: u64) -> String {
  let mut file_base = (100_000_000 + index).to_string();
  file_base.remove(0);
  directory.to_string() + "/" + &file_base + ".txt"
}

pub fn save_para_to_file(data: &str, filename: &str) -> Result<(), Error> {
  let para_file = match File::create(filename) {
    Ok(fh) => fh,
    Err(e) => return Err(e),
  };
  let mut para_writer = BufWriter::with_capacity(BUFFER_CAPACITY, para_file);
  if let Err(e) = para_writer.write(data.as_bytes()) {
    println!("Failed to print to BufWriter! Reason: {:?}", e);
  }
  if let Err(e) = para_writer.flush() {
    println!("Failed to print to BufWriter! Reason: {:?}", e);
  }
  Ok(())
}

/// Given a `CorTeX` corpus of HTML5 documents, extract a token model as a
/// single file
pub fn main() -> Result<(), Error> {
  let start = time::get_time();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string(),
  };
  let paragraph_model_directory = match input_args.next() {
    Some(path) => path,
    None => "ams_paragraphs".to_string(),
  };

  let mut total_doc_count: u64 = 0;
  let mut document_count: u64 = 0;
  let mut paragraph_count: u64 = 0;

  let space = ' ';
  let linebreak = '\n';
  // Integers, floats, subfigure numbers
  let is_numeric = Regex::new(r"^-?(?:\d+)(?:[a-k]|(?:\.\d+(?:[eE][+-]?\d+)?))?$").unwrap();
  let mut overflow_count = 0;

  let mut corpus = Corpus::new(corpus_path);
  for mut document in corpus.iter() {
    total_doc_count += 1;
    // Only analyze if document contains AMS markup
    if !ams::has_markup(&document) {
      continue;
    }
    document_count += 1;
    let mut context = Context::new(&document.dom).unwrap();

    for mut paragraph in document.paragraph_iter() {
      let mut paragraph_buffer = String::new();
      let mut sentence_buffer;
      let mut invalid_paragraph = false;
      let para_parent = paragraph.dnm.root_node.get_parent().unwrap();

      'sentences: for mut sentence in paragraph.iter() {
        sentence_buffer = String::new();
        for word in sentence.simple_iter() {
          let lexeme_str: String;
          if !word.range.is_empty() {
            let mut word_string = word
              .range
              .get_plaintext()
              .chars()
              .filter(|c| c.is_alphanumeric()) // drop apostrophes, other noise?
              .collect::<String>()
              .to_lowercase();
            if word_string.len() > MAX_WORD_LENGTH {
              // Using a more aggressive normalization, large words tend to be conversion
              // errors with lost whitespace - drop the entire paragraph when this occurs.
              overflow_count += 1;
              invalid_paragraph = true;
              break 'sentences;
            }
            let mut word_str: &str = &word_string;
            // Note: the formula and citation counts are an approximate lower bound, as
            // sometimes they are not cleanly tokenized, e.g. $k$-dimensional
            // will be the word string "mathformula-dimensional"
            if word_string.contains("mathformula") {
              lexeme_str = dnm::node::lexematize_math(word.range.get_node(), &mut context);
              word_str = &lexeme_str;
            } else if word_string.contains("citationelement") {
              word_str = "citationelement";
            } else if is_numeric.is_match(&word_string) {
              word_str = "NUM";
            }
            sentence_buffer.push_str(word_str);
            sentence_buffer.push(space);
          }
        }

        if !sentence_buffer.is_empty() {
          paragraph_buffer.push_str(&sentence_buffer);
          paragraph_buffer.push(linebreak);
        }
      }
      // If paragraph was valid and contains text, record it
      if !invalid_paragraph && !paragraph_buffer.is_empty() {
        // paragraph was valid, what is its label?
        let parent_class = para_parent.get_attribute("class").unwrap_or_default();
        let ams_class = ams::class_to_env(&parent_class);
        // if Other markup, ignore the paragraph to avoid noise
        if ams_class == Some(AmsEnv::Other) {
          continue;
        }
        // if None, record as "other" in model
        let ams_dir = match ams_class {
          Some(env) => env.to_string(),
          None => String::from("other"),
        };
        let full_dir = paragraph_model_directory.clone() + "/" + &ams_dir;
        fs::create_dir_all(&full_dir)?;
        paragraph_count += 1;
        let full_filename = num_file_path(&full_dir, paragraph_count);
        save_para_to_file(&paragraph_buffer, &full_filename)?;
      }
    }

    if document_count % 1000 == 0 {
      println!("-- processed documents: {:?}", total_doc_count);
      println!("-- AMS documents: {:?}", document_count);
    }
  }

  let end = time::get_time();
  let duration_sec = (end - start).num_milliseconds() / 1000;
  println!("---");
  println!(
    "AMS paragraph model finished in {:?}s, gathered: ",
    duration_sec
  );
  println!("{:?} documents;", document_count);
  println!("{:?} paragraphs;", paragraph_count);
  println!("{:?} discarded paragraphs (long words)", overflow_count);
  Ok(())
}
