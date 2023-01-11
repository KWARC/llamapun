//! Helpers with transactional logic related to llamapun::data
//! which doesn't fit with the main structs
//! TODO: May be reorganized better with some more thought, same as path_helpers

use lazy_static::lazy_static;
use libxml::readonly::RoNode;
use libxml::xpath::Context;
use regex::Regex;
use std::error::Error;
use whatlang::{detect, Lang, Script};

use crate::dnm;
use crate::dnm::{DNMParameters, DNMRange, DNM};
use crate::tokenizer::Tokenizer;

// Integers, floats, subfigure numbers
lazy_static! {
  static ref IS_NUMERIC: Regex =
    Regex::new(r"^-?(?:\d+)(?:[a-k]|(?:\.\d+(?:[eE][+-]?\d+)?))?$").unwrap();
  static ref IS_NUM: Regex = Regex::new(r"\s*NUM\s*").unwrap();
  static ref ROMAN_NUMERAL: Regex = Regex::new(r"(^|\s)[xiv]*(\s|$)").unwrap();
  static ref SINGLE_LEAD_LETTER: Regex = Regex::new(r"(^|\s)[abcdefghijklmnop](\s|$)").unwrap();
  static ref SINGLE_TRAIL_LETTER: Regex = Regex::new(r"\s[abcdefghijklmnop]$").unwrap();
  static ref LEAD_FIXED_PHRASE: Regex = Regex::new(r"^(comparison with|(?:list|summary|outline|sketch|overview|start|end) of|general|other|additional|completion|finishing|first|second|third|new|alternative|chapter|section|some|basic|closely|an|our|the)\s").unwrap();
  static ref TRAILING_FIXED_WORD: Regex = Regex::new(r"\s(see|of|for|of the paper)$").unwrap();
  static ref COMMON_PLURALS : Regex = Regex::new(
    r"(a(?:xiom|lgorithm|ssumption|pplication)|con(?:jecture|dition|clusion|tribution)|d(?:ata\s?set|efinition|iscussion)|e(?:xperiment|xample|xercise)|lemma|m(?:odel|ethod|otivation)|notation|observation|pr(?:oof|oposition|oblem)|question|re(?:sult|mark)|s(?:ubject|tep|imulation)|theorem|work)s(?:\s|$)"
  ).unwrap();
}

static MAX_WORD_LENGTH: usize = 25;

/// Options for lexical normalization on an individual word
pub struct LexicalOptions {
  /// math will be entirely omitted when set
  pub discard_math: bool,
  /// non-alphanumeric characters will be entirely omitted when set
  pub discard_punct: bool,
  /// all letters will be lowercased when set
  pub discard_case: bool,
}
impl Default for LexicalOptions {
  fn default() -> Self {
    LexicalOptions {
      discard_math: false,
      discard_punct: true,
      discard_case: true,
    }
  }
}
/// Normalization of word lexemes created for the "AMS paragraph classification" experiment
/// operating on a DNMRange representation
/// - numeric literals are replaced by NUM
/// - citations become citationelement
/// - math is replaced by its lexeme annotation (created by latexml), with a "mathformula" fallback
/// - of the word is longer than the max length of 25, an error is returned
pub fn ams_normalize_word_range(
  range: &DNMRange,
  context: &mut Context,
  options: LexicalOptions,
) -> Result<String, Box<dyn Error>> {
  let mut word_string = if options.discard_punct {
    range
      .get_plaintext()
      .to_lowercase()
      .chars()
      .filter(|c| c.is_alphanumeric()) // drop apostrophes, other noise?
      .collect::<String>()
  } else {
    range.get_plaintext().to_lowercase()
  };
  if word_string.len() > MAX_WORD_LENGTH {
    // Using a more aggressive normalization, large words tend to be conversion
    // errors with lost whitespace - drop the entire paragraph when this occurs.
    return Err("exceeded max length".into());
  }

  // Note: the formula and citation counts are an approximate lower bound, as
  // sometimes they are not cleanly tokenized, e.g. $k$-dimensional
  // will be the word string "mathformula-dimensional"
  if word_string.contains("mathformula") {
    if options.discard_math {
      word_string = String::new();
    } else {
      word_string = dnm::node::lexematize_math(range.get_node(), context);
    }
  } else if word_string.contains("citationelement") {
    word_string = String::from("citationelement");
  } else if IS_NUMERIC.is_match(&word_string) {
    word_string = String::from("NUM");
  }

  Ok(word_string)
}

/// Provides a string for a given heading node, using DNM-enabled word-tokenization
/// TODO: This is a low-level auxiliary function, we may need to build more user-facing interfaces
/// if it becomes more widely useful
pub fn heading_from_node_aux(
  node: RoNode,
  tokenizer: &Tokenizer,
  context: &mut Context,
) -> Option<String> {
  let heading_dnm = DNM::new(node, DNMParameters::llamapun_normalization());
  let heading_range = match heading_dnm.get_range() {
    Ok(range) => range,
    _ => return None,
  };
  let mut heading_text = String::new();
  for word_range in tokenizer.words(&heading_range) {
    if word_range.is_empty() {
      continue;
    }
    let heading_word =
      match ams_normalize_word_range(&word_range, context, LexicalOptions::default()) {
        Ok(w) => w,
        Err(_) => return None,
      };
    if !heading_word.is_empty() && heading_word != "NUM" {
      heading_text.push_str(&heading_word);
      heading_text.push(' ');
    }
  }
  Some(heading_text)
}

/// Attempt to recover the "type" of a potentially specialized heading,
/// e.g. "definition xiii a"->"definition"
#[allow(clippy::cognitive_complexity)]
pub fn normalize_heading_title(heading: &str) -> String {
  let simple_heading = ROMAN_NUMERAL.replace_all(heading.trim(), "");
  let simple_heading = IS_NUM.replace_all(simple_heading.trim(), " ");
  let simple_heading = SINGLE_LEAD_LETTER.replace_all(simple_heading.trim(), "");
  let simple_heading = SINGLE_TRAIL_LETTER.replace_all(simple_heading.trim(), "");
  let simple_heading = LEAD_FIXED_PHRASE.replace_all(simple_heading.trim(), "");
  let simple_heading = TRAILING_FIXED_WORD.replace_all(simple_heading.trim(), "");
  let simple_heading = COMMON_PLURALS.replace_all(simple_heading.trim(), "$1");
  if simple_heading.is_empty() {
    // quick exit if empty
    String::new()
  } else if simple_heading != heading {
    // if the individual regexes reduced the heading, try them again, since we may have intermixed
    // cases
    normalize_heading_title(&simple_heading)
  } else {
    // Otherwise, just look for simple variations of known cases, or return as-is:
    match simple_heading.as_ref() {
      // ignore non-English
      "lemme" | "remarque" | "corollaire" | "dokazatelstvo" => "",
      // synonyms
      "hypothesis" | "hypotheses" => "conjecture",
      "implementation details" => "implementation",
      "mathematics subject classification" | "subject headings" => "subject",
      "bibliography" => "references",
      "previous work" | "prior work" | "related literature" | "related research"
      | "related studies" | "literature review" => "related work",
      "preliminary" => "preliminaries",
      "analyses" => "analysis",
      "theoretical background" => "background",
      "exemple" => "example",
      "exercise" => "problem",
      // starts are strong cueues than ends
      h if h.starts_with("demonstration ") => "demonstration",
      h if h.starts_with("simulation result") => "result",
      h if h.starts_with("simulation ") => "simulation",
      h if h.starts_with("acknowledg") || h.starts_with("aknowledg") => "acknowledgement",
      h if h.starts_with("proof") => "proof",
      h if h.starts_with("remark ") => "remark",
      h if h.starts_with("experiment") => "experiment",
      h if h.starts_with("key word") || h.starts_with("keyword") => "keywords",
      h if h.starts_with("introduction") => "introduction",
      h if h.starts_with("related work") => "related work",
      h if h.starts_with("background ") => "background",
      h if h.starts_with("appendi") => "appendix",
      h if h.starts_with("notation") => "notation",
      h if h.starts_with("theorem") => "theorem",
      h if h.starts_with("lemma") => "lemma",
      h if h.starts_with("corollary") => "corollary",
      h if h.starts_with("proposition") => "proposition",
      h if h.starts_with("definition") => "definition",
      h if h.starts_with("axiom") => "axiom",
      h if h.starts_with("conjecture") || h.starts_with("hypothesis") => "conjecture",
      h if h.starts_with("fact ") => "fact",
      h if h.starts_with("problem ") || h.starts_with("exercise ") => "problem",
      h if h.starts_with("question ") => "question",
      h if h.starts_with("result") => "result",
      h if h.starts_with("msc") => "subject",
      h if h.starts_with("conclusion") || h.starts_with("concluding remarks") => "conclusion",
      h if h.starts_with("summary ") => "summary",
      h if h.starts_with("observation") => "observation",
      h if h.starts_with("model") => "model",
      h if h.starts_with("method") => "methods",
      h if h.starts_with("future") => "future work",
      h if h.starts_with("description") => "description",
      h if h.starts_with("discussion") => "discussion",
      h if h.starts_with("example") => "example",
      h if h.starts_with("properties") || h.starts_with("property ") => "property",
      h if h.starts_with("preliminaries ") => "preliminaries",
      h if h.starts_with("condition ") => "condition",
      h if h.starts_with("contribution ") => "contribution",
      h if h.starts_with("analaysis") || h.starts_with("analysis ") => "analysis",
      h if h.starts_with("motivation ") => "motivation",
      // ends are still usable clues
      h if h.ends_with(" demonstration") => "demonstration",
      h if h.ends_with(" simulation") => "simulation",
      h if h.ends_with(" proof") => "proof",
      h if h.ends_with(" remark") => "remark",
      h if h.ends_with(" notation") => "notation",
      h if h.ends_with(" experiment") => "experiment",
      h if h.ends_with(" theorem") => "theorem",
      h if h.ends_with(" lemma") => "lemma",
      h if h.ends_with(" corollary") => "corollary",
      h if h.ends_with(" proposition") => "proposition",
      h if h.ends_with(" definition") => "definition",
      h if h.ends_with(" axiom") => "axiom",
      h if h.ends_with(" conjecture") || h.ends_with(" hypothesis") => "conjecture",
      h if h.ends_with(" conclusion") => "conclusion",
      h if h.ends_with(" summary") => "summary",
      h if h.ends_with(" problem") || h.ends_with("exercise") => "problem",
      h if h.ends_with(" question") => "question",
      h if h.ends_with(" result") => "result",
      h if h.ends_with(" method") => "methods",
      h if h.ends_with(" model") => "model",
      h if h.ends_with(" description") => "description",
      h if h.ends_with(" discussion") => "discussion",
      h if h.ends_with(" example") => "example",
      h if h.ends_with(" property") || h.ends_with(" properties") => "property",
      h if h.ends_with(" preliminaries") => "preliminaries",
      h if h.ends_with(" condition") => "condition",
      h if h.ends_with(" contribution") => "contribution",
      h if h.ends_with(" analysis") => "analysis",
      h if h.ends_with(" motivation") => "motivation",
      // self if no known case
      any => any,
    }
    .to_string()
  }
}

// Analysis is a can of worms... there are many more, and they seem to be varying from extremely
// narrow to extremely broad discussions some are even false friends, such as method names
// "principal component analysis"
//
// there may be other cans of worms out there, normalization may end up a lot more aggressive than
// desired... but best to start somewhere
//
// "spectral analysis" | //= result
// "data analysis" | //= result (broad)
// "numerical analysis" | // result
// "convergence analysis" | // result
// "error analysis" | // result (broad)
// "performance analysis" | // result (broad)
// "principal component analysis" | // technique
// "stability analysis" | // result
// "theoretical analysis" | // result (broad)
// "complexity analysis" | // result
// "timing analysis" |
// "statistical analysis" |
// "qualitative analysis" |
// "sensitivity analysis" |
// "data and analysis" |
// "linear stability analysis" |
// "asymptotic analysis" |
// "security analysis" |
// "data reduction and analysis" |
// "abundance analysis" |
// "image analysis" |
// "real data analysis" |
// "light curve analysis" |
// "spectroscopic analysis"  => "analysis",

/// Check if the given DNM contains valid English+Latin content
pub fn invalid_for_english_latin(dnm: &DNM) -> bool {
  let detectable_with_spaces = dnm
    .plaintext
    .replace("mathformula", " ")
    .replace("CitationElement", " ")
    .replace("REF", " ");
  let detectable = detectable_with_spaces.trim();
  if let Some(info) = detect(detectable) {
    info.script() != Script::Latin || (info.lang() != Lang::Eng && info.confidence() > 0.93)
  } else {
    false
  }
}
