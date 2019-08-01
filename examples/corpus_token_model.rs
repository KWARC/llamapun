// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

use libxml::xpath::Context;
use llamapun::dnm;
use llamapun::dnm::SpecialTagsOption;
use llamapun::parallel_data::*;

static BUFFER_CAPACITY: usize = 10_485_760;
static MAX_WORD_LENGTH: usize = 25;

/// Given a `CorTeX` corpus of HTML5 documents, extract a token model as a
/// single file
pub fn main() {
  let start = time::get_time();
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

  let token_model_file = match File::create(token_model_filepath) {
    Ok(fh) => fh,
    Err(e) => {
      println!(
        "Failed to open token model output file, aborting. Reason: {:?}",
        e
      );
      return;
    }
  };

  let discard_math = match input_args.next() {
    Some(value) => match value.as_str() {
      "discard_math" => true, // should eventually become --discard_math flag, rushing for now.
      _ => false,
    },
    None => false,
  };

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
      mut sentence_count,
      mut paragraph_count,
      mut word_count,
      mut overflow_count,
      mut formula_count,
      mut citation_count,
      mut num_count,
    ) = (0, 0, 0, 0, 0, 0, 0);
    let mut thread_buffer = String::new();

    let mut context = Context::new(&document.dom).unwrap();
    for mut paragraph in document.paragraph_iter() {
      paragraph_count += 1;
      for mut sentence in paragraph.iter() {
        let mut sentence_buffer = String::new();
        let mut invalid_sentence = true;
        'words: for word in sentence.word_iter() {
          let lexeme_str: String;
          if !word.range.is_empty() {
            let word_string = word
              .range
              .get_plaintext()
              .chars()
              .filter(|c| c.is_alphanumeric()) // drop apostrophes, other noise?
              .collect::<String>()
              .to_lowercase();
            if word_string.len() > MAX_WORD_LENGTH {
              // Using a more aggressive normalization, large words tend to be conversion
              // errors with lost whitespace - drop the entire sentence when this occurs.
              overflow_count += 1;
              invalid_sentence = true;
              break 'words;
            }
            let mut word_str: &str = &word_string;
            // Note: the formula and citation counts are an approximate lower bound, as
            // sometimes they are not cleanly tokenized, e.g. $k$-dimensional
            // will be the word string "mathformula-dimensional"
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

            invalid_sentence = false;
            sentence_buffer.push_str(word_str);
            sentence_buffer.push(space);
          }
        }

        // if valid sentence, print to the token model file
        if !invalid_sentence {
          sentence_count += 1;
          thread_buffer.push(linebreak);
          thread_buffer.push_str(&sentence_buffer);
        }
      }
    }

    token_writer
      .lock()
      .unwrap()
      .write_all(thread_buffer.as_bytes())
      .expect("thread writing to token model buffer should always succeed.");

    let mut thread_counts = HashMap::new();
    thread_counts.insert(String::from("document_count"), 1);
    thread_counts.insert(String::from("sentence_count"), sentence_count);
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

  let end = time::get_time();
  let duration_sec = (end - start).num_milliseconds() / 1000;
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
    "{:?} sentences;",
    corpus_counts.get("sentence_count").unwrap_or(&0)
  );
  println!(
    "{:?} discarded sentences (long words)",
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
}
