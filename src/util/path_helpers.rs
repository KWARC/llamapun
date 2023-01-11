//! Helpers intended mostly for non-Rust use, where rust is passed in a filesystem path
//! and a llamapun-supported processing is requested
use libxml::xpath::Context;

use crate::data::{Corpus, Document};
use crate::util::data_helpers;
use crate::util::data_helpers::LexicalOptions;

/// Given a path to a document, return a word-tokenized string of all of its paragraphs
pub fn path_to_words(path: String) -> String {
  let corpus = Corpus::default();
  let mut document = Document::new(path, &corpus).unwrap();
  let mut context = Context::new(&document.dom).unwrap();

  // We will tokenize each logical paragraph, which are the textual logical units
  // in an article
  let mut document_buffer = String::new();
  for mut paragraph in document.paragraph_iter() {
    let mut invalid_paragraph = false;
    let mut paragraph_buffer = String::new();
    'sentences: for mut sentence in paragraph.iter() {
      let mut sentence_buffer = String::new();
      for word in sentence.simple_iter() {
        if !word.range.is_empty() {
          let word_string = match data_helpers::ams_normalize_word_range(
            &word.range,
            &mut context,
            LexicalOptions::default(),
          ) {
            Ok(w) => w,
            Err(_) => {
              invalid_paragraph = true;
              break 'sentences;
            },
          };
          sentence_buffer.push_str(&word_string);
          sentence_buffer.push(' ');
        }
      }
      if !sentence_buffer.is_empty() {
        paragraph_buffer.push_str(&sentence_buffer);
        paragraph_buffer.push('\n');
      }
    }
    if !invalid_paragraph {
      document_buffer.push_str(&paragraph_buffer);
    }
  }
  document_buffer
}
