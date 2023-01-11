//! A "corpus token model"-generation utilities
use crate::dnm;
use crate::dnm::SpecialTagsOption;
use crate::parallel_data::*;
use libxml::xpath::Context;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

static BUFFER_CAPACITY: usize = 10_485_760;
static MAX_WORD_LENGTH: usize = 25;

/// Parallel traversal of latexml-style HTML5 document corpora, based on jwalk and
/// `DNMParameter::llamapun_normalization` with additional subformula lexemes via
/// `dnm::node::lexematize_math`
pub fn extract(
  corpus_path: String,
  token_model_filepath: String,
  discard_math: bool,
) -> Result<HashMap<String, u64>, Box<dyn Error>> {
  let token_model_file = File::create(token_model_filepath)?;

  let token_writer = Arc::new(Mutex::new(BufWriter::with_capacity(
    BUFFER_CAPACITY,
    token_model_file,
  )));
  let space = ' ';
  let linebreak = '\n';
  // Integers, floats, subfigure numbers
  let is_numeric = Regex::new(r"^-?(?:\d+)(?:[a-k]|(?:\.\d+(?:[eE][+-]?\d+)?))?$").unwrap();

  let mut corpus = Corpus::new(corpus_path);
  if discard_math {
    println!("-- will discard math.");
    corpus
      .dnm_parameters
      .special_tag_name_options
      .insert("math".to_string(), SpecialTagsOption::Skip);
    corpus
      .dnm_parameters
      .special_tag_class_options
      .insert("ltx_equation".to_string(), SpecialTagsOption::Skip);
    corpus
      .dnm_parameters
      .special_tag_class_options
      .insert("ltx_equationgroup".to_string(), SpecialTagsOption::Skip);
  } else {
    println!("-- will lexematize math.")
  }

  let corpus_counts = corpus.catalog_with_parallel_walk(|document| {
    let (
      mut paragraph_count,
      mut word_count,
      mut overflow_count,
      mut formula_count,
      mut citation_count,
      mut num_count,
    ) = (0, 0, 0, 0, 0, 0);
    let mut thread_buffer = String::new();

    let mut context = Context::new(&document.dom).unwrap();
    for mut paragraph in document.extended_paragraph_iter() {
      paragraph_count += 1;
      let mut paragraph_buffer = String::new();
      let mut invalid_paragraph = false;
      'words: for word in paragraph.word_and_punct_iter() {
        if !word.range.is_empty() {
          let word_string = word.range.get_plaintext().to_lowercase();
          if word_string.len() > MAX_WORD_LENGTH {
            // Using a more aggressive normalization, large words tend to be conversion
            // errors with lost whitespace - drop the entire paragraph when this occurs.
            overflow_count += 1;
            invalid_paragraph = true;
            break 'words;
          }
          let mut word_str: &str = &word_string;
          // Note: the formula and citation counts are an approximate lower bound, as
          // sometimes they are not cleanly tokenized, e.g. $k$-dimensional
          // will be the word string "mathformula-dimensional"
          let lexeme_str: String;
          if word_string.contains("mathformula") {
            if !discard_math {
              lexeme_str = dnm::node::lexematize_math(word.range.get_node(), &mut context);
            } else {
              lexeme_str = String::new();
            }
            word_str = &lexeme_str;
            formula_count += 1;
          } else if word_string.contains("citationelement") {
            word_str = "citationelement";
            citation_count += 1;
          } else if is_numeric.is_match(&word_string) {
            num_count += 1;
            word_str = "NUM";
          } else {
            word_count += 1;
          }
          paragraph_buffer.push_str(word_str);
          paragraph_buffer.push(space);
        }
      }
      // if valid paragraph, print to the token model file
      if !invalid_paragraph {
        thread_buffer.push(linebreak);
        thread_buffer.push_str(&paragraph_buffer);
      }
    }

    token_writer
      .lock()
      .unwrap()
      .write_all(thread_buffer.as_bytes())
      .expect("thread writing to token model buffer should always succeed.");

    let mut thread_counts = HashMap::new();
    thread_counts.insert(String::from("document_count"), 1);
    thread_counts.insert(String::from("paragraph_count"), paragraph_count);
    thread_counts.insert(String::from("word_count"), word_count);
    thread_counts.insert(String::from("overflow_count"), overflow_count);
    thread_counts.insert(String::from("formula_count"), formula_count);
    thread_counts.insert(String::from("citation_count"), citation_count);
    thread_counts.insert(String::from("num_count"), num_count);
    thread_counts
  });

  token_writer
    .lock()
    .unwrap()
    .flush()
    .expect("token writer failed to flush, data is likely incomplete.");

  Ok(corpus_counts)
}
