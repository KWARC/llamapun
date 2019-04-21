//! Helpers with transactional logic related to llamapun::data
//! which doesn't fit with the main structs
//! TODO: May be reorganized better with some more thought, same as path_helpers

use lazy_static::lazy_static;
use libxml::xpath::Context;
use regex::Regex;

use crate::dnm;
use crate::dnm::DNMRange;

// Integers, floats, subfigure numbers
lazy_static! {
  static ref IS_NUMERC: Regex =
    Regex::new(r"^-?(?:\d+)(?:[a-k]|(?:\.\d+(?:[eE][+-]?\d+)?))?$").unwrap();
}

static MAX_WORD_LENGTH: usize = 25;

/// Normalization of word lexemes created for the "AMS paragraph classification" experiment
/// operating on a DNMRange representation
/// - numeric literals are replaced by NUM
/// - citations become citationelement
/// - math is replaced by its lexeme annotation (created by latexml), with a "mathformula" fallback
/// - of the word is longer than the max length of 25, an error is returned
pub fn ams_normalize_word_range(range: &DNMRange, mut context: &mut Context) -> Result<String, ()> {
  let mut word_string = range
    .get_plaintext()
    .chars()
    .filter(|c| c.is_alphanumeric()) // drop apostrophes, other noise?
    .collect::<String>()
    .to_lowercase();
  if word_string.len() > MAX_WORD_LENGTH {
    // Using a more aggressive normalization, large words tend to be conversion
    // errors with lost whitespace - drop the entire paragraph when this occurs.
    return Err(());
  }

  // Note: the formula and citation counts are an approximate lower bound, as
  // sometimes they are not cleanly tokenized, e.g. $k$-dimensional
  // will be the word string "mathformula-dimensional"
  if word_string.contains("mathformula") {
    word_string = dnm::node::lexematize_math(range.get_node(), &mut context);
  } else if word_string.contains("citationelement") {
    word_string = String::from("citationelement");
  } else if IS_NUMERC.is_match(&word_string) {
    word_string = String::from("NUM");
  }

  Ok(word_string)
}
