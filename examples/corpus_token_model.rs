// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
extern crate libxml;
extern crate llamapun;
extern crate regex;
extern crate time;

use regex::Regex;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use libxml::xpath::Context;
use llamapun::data::Corpus;
use llamapun::dnm;

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

  let mut document_count = 0;
  let mut paragraph_count = 0;
  let mut sentence_count = 0;
  let mut word_count = 0;
  let mut formula_count = 0;
  let mut citation_count = 0;
  let mut num_count = 0;
  let mut overflow_count = 0;

  let token_model_file = match File::create(token_model_filepath) {
    Ok(fh) => fh,
    Err(e) => {
      println!(
        "Failed to open token model output file, aborting. Reason: {:?}",
        e
      );
      return;
    },
  };
  let mut token_writer = BufWriter::with_capacity(BUFFER_CAPACITY, token_model_file);
  let space = ' ';
  let linebreak = '\n';
  // Integers, floats, subfigure numbers
  let is_numeric = Regex::new(r"^-?(?:\d+)(?:[a-k]|(?:\.\d+(?:[eE][+-]?\d+)?))?$").unwrap();

  let mut corpus = Corpus::new(corpus_path);
  for mut document in corpus.iter() {
    document_count += 1;
    let mut context = Context::new(&document.dom).unwrap();
    for mut paragraph in document.paragraph_iter() {
      paragraph_count += 1;
      for mut sentence in paragraph.iter() {
        let mut sentence_buffer = String::new();
        let mut invalid_sentence = true;
        'words: for word in sentence.simple_iter() {
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
              lexeme_str = dnm::node::lexematize_math(word.range.get_node(), &mut context);
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
          sentence_buffer.push(linebreak);
          if let Err(e) = token_writer.write(sentence_buffer.as_bytes()) {
            println!(
              "-- Failed to print to output buffer! Proceed with caution;\n{:?}",
              e
            );
          }
        }
      }
    }

    if document_count % 1000 == 0 {
      println!("-- processed documents: {:?}", document_count);
    }
  }

  if let Err(e) = token_writer.flush() {
    println!(
      "-- Failed to print to output buffer! Proceed with caution;\n{:?}",
      e
    );
  }

  let end = time::get_time();
  let duration_sec = (end - start).num_milliseconds() / 1000;
  println!("---");
  println!("Token model finished in {:?}s, gathered: ", duration_sec);
  println!("{:?} documents;", document_count);
  println!("{:?} paragraphs;", paragraph_count);
  println!("{:?} sentences;", sentence_count);
  println!("{:?} discarded sentences (long words)", overflow_count);
  println!("{:?} words;", word_count);
  println!("{:?} numeric literals;", num_count);
  println!("{:?} formulas;", formula_count);
  println!("{:?} inline cites;", citation_count);
}

// fn utf_truncate(input: &mut String, maxsize: usize) {
//   let mut utf_maxsize = input.len();
//   if utf_maxsize >= maxsize {
//     {
//       let mut char_iter = input.char_indices();
//       while utf_maxsize >= maxsize {
//         utf_maxsize = match char_iter.next_back() {
//           Some((index, _)) => index,
//           _ => 0,
//         };
//       }
//     } // Extra {} wrap to limit the immutable borrow of char_indices()
//     input.truncate(utf_maxsize);
//   }
// }
