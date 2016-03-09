// Copyright 2015-2016 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
extern crate llamapun;

use std::env;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use llamapun::data::Corpus;

/// Given a CorTeX corpus of HTML5 documents, extract a token model as a single file
pub fn main() {
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string()
  };
  let token_model_filepath = match input_args.next() {
    Some(path) => path,
    None => "token_model.txt".to_string()
  };

  let mut document_count = 0;
  let mut paragraph_count = 0;
  let mut sentence_count = 0;
  let mut word_count = 0;
  let mut formula_count = 0;
  let mut citation_count = 0;
  let token_model_file = match File::create(token_model_filepath) {
    Ok(fh) => fh,
    Err(e) => {
      println!("Failed to open token model output file, aborting. Reason: {:?}", e);
      return;
    }
  };
  let mut token_writer = BufWriter::with_capacity(10485760, token_model_file);
  let space = " ".as_bytes();

  let mut corpus = Corpus::new(corpus_path);
  for mut document in corpus.iter() {
    document_count += 1;
    for mut paragraph in document.paragraph_iter() {
      paragraph_count += 1;
      for mut sentence in paragraph.iter() {
        sentence_count += 1;
        for word in sentence.simple_iter() {
          if !word.range.is_empty() {
            let word_string = word.range.get_plaintext().to_lowercase();
            if word_string == "mathformula" {
              formula_count += 1;
            } else if word_string == "citationelement" {
              citation_count +=1;
            } else {
              word_count += 1;
            }
            // print to the token model file
            match token_writer.write(word_string.as_bytes()) {
              Err(e) => println!("-- Failed to print to output buffer! Proceed with caution;\n{:?}",e),
              _ => {}
            };
            match token_writer.write(space) {
              Err(e) => println!("-- Failed to print to output buffer! Proceed with caution;\n{:?}",e),
              _ => {}
            };
          }
        }
      }
    }
  }

  match token_writer.flush() {
    Err(e) => println!("-- Failed to print to output buffer! Proceed with caution;\n{:?}",e),
    _ => {}
  };

  println!("Token model finished, gathered: ");
  println!("{:?} documents;", document_count);
  println!("{:?} paragraphs;", paragraph_count);
  println!("{:?} sentences;", sentence_count);
  println!("{:?} words;", word_count);
  println!("{:?} formulas;", formula_count);
  println!("{:?} inline cites;", citation_count);

}

