//! Helpers with transactional logic related to llamapun::data
//! which doesn't fit with the main structs
//! TODO: May be reorganized better with some more thought, same as path_helpers

use lazy_static::lazy_static;
use libxml::xpath::Context;
use regex::Regex;
use whatlang::{detect, Lang, Script};

use crate::dnm;
use crate::dnm::DNMRange;

// Integers, floats, subfigure numbers
lazy_static! {
  static ref IS_NUMERC: Regex =
    Regex::new(r"^-?(?:\d+)(?:[a-k]|(?:\.\d+(?:[eE][+-]?\d+)?))?$").unwrap();
  static ref ROMAN_NUMERAL: Regex = Regex::new(r"(^|\s)[xiv]*(\s|$)").unwrap();
  static ref SINGLE_LEAD_LETTER: Regex = Regex::new(r"(^|\s)[abcdefghijklmnop](\s|$)").unwrap();
  static ref SINGLE_TRAIL_LETTER: Regex = Regex::new(r"\s[abcdefghijklmnop]$").unwrap();
  static ref LEAD_FIXED_WORD: Regex = Regex::new(r"^(chapter|section|an|our|the)\s").unwrap();
  static ref TRAILING_FIXED_WORD: Regex = Regex::new(r"\s(see|of|for)$").unwrap();
  static ref TRAILING_PLURALS : Regex = Regex::new(r"(notation|definition|discussion|axiom|conjecture|experiment|algorithm|assumption|step|application|model|question|conclusion|theorem|lemma|proof|method|result|proposition|remark|problem|observation)s$").unwrap();
}

static MAX_WORD_LENGTH: usize = 25;

/// Normalization of word lexemes created for the "AMS paragraph classification" experiment
/// operating on a DNMRange representation
/// - numeric literals are replaced by NUM
/// - citations become citationelement
/// - math is replaced by its lexeme annotation (created by latexml), with a "mathformula" fallback
/// - of the word is longer than the max length of 25, an error is returned
pub fn ams_normalize_word_range(
  range: &DNMRange,
  mut context: &mut Context,
  discard_math: bool,
) -> Result<String, ()> {
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
    if !discard_math {
      word_string = dnm::node::lexematize_math(range.get_node(), &mut context);
    } else {
      word_string = String::new();
    }
  } else if word_string.contains("citationelement") {
    word_string = String::from("citationelement");
  } else if IS_NUMERC.is_match(&word_string) {
    word_string = String::from("NUM");
  }

  Ok(word_string)
}

/// Attempt to recover the "type" of a potentially specialized heading,
/// e.g. "definition xiii a"->"definition"
#[allow(clippy::cognitive_complexity)]
pub fn normalize_heading_title(heading: &str) -> String {
  let simple_heading = ROMAN_NUMERAL.replace_all(heading.trim(), "");
  let simple_heading = SINGLE_LEAD_LETTER.replace_all(simple_heading.trim(), "");
  let simple_heading = SINGLE_TRAIL_LETTER.replace_all(simple_heading.trim(), "");
  let simple_heading = LEAD_FIXED_WORD.replace(simple_heading.trim(), "");
  let simple_heading = TRAILING_FIXED_WORD.replace_all(simple_heading.trim(), "");
  let simple_heading = TRAILING_PLURALS.replace_all(simple_heading.trim(), "$1");
  // let simple_heading = simple_heading.to_string();
  match &simple_heading {
    h if h.starts_with("demonstration") || h.ends_with(" demonstration") => "demonstration",
    h if h.starts_with("proof") || h.ends_with(" proof") => "proof",
    h if h.starts_with("remark") || h.ends_with(" remark") => "remark",
    h if h.starts_with("experiment") || h.ends_with(" experiment") => "experiment",
    h if h.starts_with("key word") || h.starts_with("keyword") => "keywords",
    h if h.starts_with("introduction") => "introduction",
    h if h.starts_with("related work") => "related work",
    h if h.starts_with("acknowledg") => "acknowledgement",
    h if h.starts_with("appendi") => "appendix",
    h if h.starts_with("lemma") || h.ends_with(" lemma") => "lemma",
    h if h.starts_with("theorem") || h.ends_with(" theorem") => "theorem",
    h if h.starts_with("notation") || h.ends_with(" notation") => "notation",
    h if h.starts_with("corollary") || h.ends_with(" corollary") => "corollary",
    h if h.starts_with("proposition") || h.ends_with(" proposition") => "proposition",
    h if h.starts_with("definition") || h.ends_with(" definition") => "definition",
    h if h.starts_with("axiom") || h.ends_with(" axiom") => "axiom",
    h if h.starts_with("conjecture") || h.ends_with(" conjecture") => "conjecture",
    h if h.starts_with("hypothesis") || h.ends_with(" hypothesis") => "hypothesis",
    h if h.starts_with("problem") || h.ends_with(" problem") => "problem",
    h if h.starts_with("result") || h.ends_with(" result") => "result",
    h if h.starts_with("method") || h.ends_with(" method") => "method",
    h if h.starts_with("msc") => "mathematics subject classification",
    h if h.starts_with("conclusion") || h.ends_with(" conclusion") => "conclusion",
    h if h.starts_with("observation") => "observation",
    h if h.starts_with("model") || h.ends_with(" model") => "model",
    h if h.starts_with("method") || h.ends_with(" method") => "methods",
    h if h.starts_with("future") => "future work",
    h if h.starts_with("description") || h.ends_with(" description") => "description",
    h if h.starts_with("discussion") || h.ends_with(" discussion") => "discussion",
    any => any,
  }
  .to_string()
}

/// Check if the given DNM contains valid English+Latin content
pub fn invalid_for_english_latin(dnm: &dnm::DNM) -> bool {
  let detectable_with_spaces = dnm
    .plaintext
    .replace("mathformula", " ")
    .replace("CitationElement", " ")
    .replace("REF", " ");
  let detectable = detectable_with_spaces.trim();
  if let Some(info) = detect(&detectable) {
    info.script() != Script::Latin || (info.lang() != Lang::Eng && info.confidence() > 0.93)
  } else {
    false
  }
}
