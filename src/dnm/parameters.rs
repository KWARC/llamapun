//! The `dnm::parameters` submodule provides data structures for customizing and configuring a DNM's construction and use

use std::collections::HashMap;
use libxml::tree::*;
use std::rc::Rc;

/// Some temporary data for the parser
pub struct RuntimeParseData {
  /// plaintext is currently terminated by some whitespace
  pub had_whitespace: bool,
  /// plaintext representation as vector of chars (to deal with UTF-8 mess)
  /// TODO: Use plaintext/byte_offsets directly instead
  pub chars : Vec<char>,
}
impl Default for RuntimeParseData {
  fn default() -> RuntimeParseData {
    RuntimeParseData {
      had_whitespace: true, // skip leading whitespace
      chars : Vec::new(),
    }
  }
}


/// Specifies how to deal with a certain tag
#[derive(Clone)]
pub enum SpecialTagsOption {
  /// Recurse into tag (default behaviour)
  Enter,
  /// Normalize tag, replacing it by some token
  Normalize(String),
  /// Normalize tag, obtain replacement string by function call
  FunctionNormalize(Rc<fn(&Node) -> String>),
  /// Skip tag
  Skip,
}


/// Parameters for the DNM generation
#[derive(Clone)]
pub struct DNMParameters {
  /// How to deal with special tags (e.g. `<math>` tags)
  pub special_tag_name_options: HashMap<String, SpecialTagsOption>,
  /// How to deal with tags with special class names (e.g. ltx_note_mark)
  /// *Remark*: If both a tag name and a tag class match, the tag name rule
  /// will be applied.
  pub special_tag_class_options: HashMap<String, SpecialTagsOption>,
  /// merge sequences of whitespaces into a single ' '.
  /// *Doesn't affect tokens*
  pub normalize_white_spaces: bool,
  /// put spaces before and after tokens
  pub wrap_tokens: bool,
  /// Replace unicode characters by the ascii code representation
  pub normalize_unicode: bool,
  /// Apply the morpha stemmer once to the text nodes
  pub stem_words_once: bool,
  /// Apply the morpha stemmer to the text nodes
  /// as often as it changes something
  pub stem_words_full: bool,
  /// Move to lowercase (remark: The stemmer does this automatically)
  pub convert_to_lowercase: bool,
  /// Support back mapping, i.e. mapping plaintext offsets back to the DOM
  pub support_back_mapping: bool,
}

impl Default for DNMParameters {
  /// Don't do anything fancy and specific by default
  fn default() -> DNMParameters {
    DNMParameters {
      special_tag_name_options: HashMap::new(),
      special_tag_class_options: HashMap::new(),
      normalize_white_spaces: true,
      wrap_tokens: false,
      normalize_unicode: false,
      stem_words_once: false,
      stem_words_full: false,
      convert_to_lowercase: false,
      support_back_mapping: true,
    }
  }
}

impl DNMParameters {
  /// Normalize in a reasonable way for our math documents
  pub fn llamapun_normalization() -> DNMParameters {
    let mut name_options = HashMap::new();
    name_options.insert("math".to_string(),
                        SpecialTagsOption::Normalize("MathFormula".to_string()));
    name_options.insert("cite".to_string(),
                        SpecialTagsOption::Normalize("CitationElement".to_string()));
    name_options.insert("table".to_string(), SpecialTagsOption::Skip);
    name_options.insert("head".to_string(), SpecialTagsOption::Skip);

    let mut class_options = HashMap::new();
    class_options.insert("ltx_equation".to_string(),
                         SpecialTagsOption::Normalize("\nMathFormula\n".to_string()));
    class_options.insert("ltx_equationgroup".to_string(),
                         SpecialTagsOption::Normalize("\nMathFormula\n".to_string()));
    class_options.insert("ltx_note_mark".to_string(), SpecialTagsOption::Skip);
    class_options.insert("ltx_note_outer".to_string(), SpecialTagsOption::Skip);
    class_options.insert("ltx_bibliography".to_string(), SpecialTagsOption::Skip);

    DNMParameters {
      special_tag_name_options: name_options,
      special_tag_class_options: class_options,
      normalize_white_spaces: false, // Keeping it raw for tokenization best results, newlines are meaningful
      wrap_tokens: false,
      normalize_unicode: true,
      ..Default::default()
    }
  }

  /// Prints warnings, if the parameter settings don't make sense.
  /// Doesn't check for every possible stupidity
  pub fn check(&self) {
    if self.stem_words_once && self.stem_words_full {
      println_stderr!("llamapun::dnm: Parameter options stem_words_once\
  and stem_words_full are both set");
    }
    if (self.stem_words_once || self.stem_words_full) && self.convert_to_lowercase {
      println_stderr!("llamapun::dnm: Parameter option convert_to_lowercase\
  is redundant, because stemming converts to lowercase already");
    }
    if (self.stem_words_once || self.stem_words_full) && self.support_back_mapping {
      println_stderr!("llamapun::dnm: Parameter option support_back_mapping\
  does not work in combination with word stemming yet");
    }
  }
}
