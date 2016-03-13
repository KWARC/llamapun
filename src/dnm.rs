//! The `dnm` can be used for easier switching between the DOM
//! (Document Object Model) representation and the plain text representation,
//! which is needed for most NLP tools.

extern crate libc;
extern crate unidecode;
extern crate rustmorpha;

use std::collections::HashMap;
use std::mem;
use std::io::Write;
use unidecode::unidecode;
use libxml::tree::*;

/// Print error message to stderr
/// from http://stackoverflow.com/questions/27588416/how-to-send-output-to-stderr
macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


/// Specifies how to deal with a certain tag
pub enum SpecialTagsOption {
  /// Recurse into tag (default behaviour)
  Enter,
  /// Normalize tag, replacing it by some token
  Normalize(String),
  /// Normalize tag, obtain replacement string by function call
  //FunctionNormalize(|&Node| -> String),
  FunctionNormalize(fn(&Node) -> String),
  /// Skip tag
  Skip,
}


/// Paremeters for the DNM generation
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
  /// if there is a trailing white space in a tag, don't make it part
  /// of that tag. Requires `normalize_white_spaces` to be set.
  pub move_whitespaces_between_nodes: bool,
  /// Replace unicode characters by the ascii code representation
  pub normalize_unicode: bool,
  /// Apply the morpha stemmer once to the text nodes
  pub stem_words_once: bool,
  /// Apply the morpha stemmer to the text nodes
  /// as often as it changes something
  pub stem_words_full: bool,
  /// Move to lowercase (remark: The stemmer does automatically)
  pub convert_to_lowercase: bool,
}

impl Default for DNMParameters {
  /// Don't do anything fancy and specific by default
  fn default() -> DNMParameters {
    DNMParameters {
      special_tag_name_options : HashMap::new(),
      special_tag_class_options : HashMap::new(),
      normalize_white_spaces: true,
      wrap_tokens: false,
      move_whitespaces_between_nodes: false,
      normalize_unicode: false,
      stem_words_once: false,
      stem_words_full: false,
      convert_to_lowercase: false
    }
  }
}

impl DNMParameters {
  /// Normalize in a reasonable way for our math documents
  pub fn llamapun_normalization() -> DNMParameters {
    let mut name_options = HashMap::new();
    name_options.insert("math".to_string(), SpecialTagsOption::Normalize("MathFormula".to_string()));
    name_options.insert("cite".to_string(), SpecialTagsOption::Normalize("CitationElement".to_string()));
    name_options.insert("table".to_string(), SpecialTagsOption::Skip);
    name_options.insert("head".to_string(), SpecialTagsOption::Skip);
    let mut class_options = HashMap::new();
    class_options.insert("ltx_equation".to_string(), SpecialTagsOption::Normalize("\nMathFormula\n".to_string()));
    class_options.insert("ltx_equationgroup".to_string(), SpecialTagsOption::Normalize("\nMathFormula\n".to_string()));
    class_options.insert("ltx_note_mark".to_string(), SpecialTagsOption::Skip);
    class_options.insert("ltx_note_outer".to_string(), SpecialTagsOption::Skip);
    class_options.insert("ltx_bibliography".to_string(), SpecialTagsOption::Skip);

    DNMParameters {
      special_tag_name_options : name_options,
      special_tag_class_options : class_options,
      normalize_white_spaces : false, // Keeping it raw for tokenization best results, newlines are meaningful
      wrap_tokens : false,
      move_whitespaces_between_nodes: false, // Keeping it raw for tokenization best results
      normalize_unicode: true,
      ..Default::default()
    }
  }

  /// Prints warnings, if the parameter settings don't make sense.
  /// Doesn't check for every possible stupidity
  fn check(&self) {
    if self.stem_words_once && self.stem_words_full {
      println_stderr!("llamapun::dnm: Parameter options stem_words_once\
  and stem_words_full are both set");
    }
    if !self.normalize_white_spaces && self.move_whitespaces_between_nodes {
      println_stderr!("llamapun::dnm: Parameter option\
  move_whitespaces_between_nodes only works in combination with normalize_white_spaces\n\
  Consider using DNMRange::trim instead");
    }
    if !self.normalize_white_spaces && self.move_whitespaces_between_nodes {
      println_stderr!("llamapun::dnm: Parameter option\
  move_whitespaces_between_nodes only works in combination with normalize_white_spaces\n\
  Consider using DNMRange::trim instead");
    }
    if (self.stem_words_once || self.stem_words_full)
        && self.convert_to_lowercase {
      println_stderr!("llamapun::dnm: Parameter option convert_to_lowercase\
  is redundant, because stemming converts to lowercase already");
    }
  }
}

/// For some reason `libc::c_void` isn't hashable and cannot be made hashable
fn node_to_hashable(node : &Node) -> usize {
  unsafe { mem::transmute::<*mut libc::c_void, usize>(node.node_ptr) }
}

/// The `DNM` is essentially a wrapper around the plain text representation
/// of the document, which facilitates mapping plaintext pieces to the DOM.
/// This breaks, if the DOM is changed after the DNM generation!
pub struct DNM {
  /// The plaintext
  pub plaintext : String,
  /// The options for generation
  pub parameters : DNMParameters,
  /// The root node of the underlying xml tree
  pub root_node : Node,
  /// Maps nodes to plaintext offsets
  //pub node_map : HashMap<Node, (usize, usize)>,
  //pub node_map : HashMap<libc::c_void, (usize, usize)>,
  pub node_map : HashMap<usize, (usize, usize)>,
}


/// Some temporary data for the parser
struct RuntimeParseData {
  /// plaintext is currently terminated by some whitespace
  had_whitespace : bool,
}

/// Very often we'll talk about substrings of the plaintext - words, sentences,
/// etc. A `DNMRange` stores start and end point of such a substring and has
/// a reference to the `DNM`.
pub struct DNMRange <'dnmrange> {
  pub start : usize,
  pub end : usize,
  pub dnm : &'dnmrange DNM,
}

impl <'dnmrange> DNMRange <'dnmrange> {
  /// Get the plaintext substring corresponding to the range
  pub fn get_plaintext(&self) -> &'dnmrange str {
    &(&self.dnm.plaintext)[self.start..self.end]
  }
  /// Get the plaintext without trailing white spaces
  pub fn get_plaintext_truncated(&self) -> &'dnmrange str {
    &(&self.dnm.plaintext)[self.start..self.end].trim_right()
  }

  pub fn trim(&self) -> DNMRange <'dnmrange> {
    let mut trimmed_start = self.start;
    let mut trimmed_end = self.end;
    let range_text : &str = self.get_plaintext();

    for c in range_text.chars() {
      if c.is_whitespace() {
        trimmed_start += 1; }
      else {
        break; }}
    for c in range_text.chars().rev() {
      if c.is_whitespace() {
        trimmed_end -= 1; }
      else {
        break; }}
    if trimmed_start > trimmed_end {  // in case range contains only whitespaces
        trimmed_start = self.start;
        trimmed_end = self.start;
    }
    DNMRange {start : trimmed_start, end: trimmed_end, dnm: self.dnm}
  }

  /// returns a subrange, with offsets relative to the beginning of `self`
  pub fn get_subrange(&self, rel_start: usize, rel_end: usize) -> DNMRange<'dnmrange> {
    DNMRange {start: self.start + rel_start, end: self.start + rel_end, dnm: self.dnm}
  }

  pub fn is_empty(&self) -> bool {
      self.start == self.end
  }
}

impl <'dnmrange> Clone for DNMRange <'dnmrange> {
  fn clone(&self) -> DNMRange <'dnmrange> {
    DNMRange {start : self.start, end: self.end, dnm: self.dnm}
  }
}

impl DNM {
  /// Creates a `DNM` for `root`
  pub fn new(root: Node, parameters: DNMParameters) -> DNM {
    parameters.check();
    let mut dnm = DNM {
      plaintext : String::new(),
      parameters : parameters,
      root_node : root.clone(),
      node_map : HashMap::new(),
    };

    let mut runtime = RuntimeParseData {
      had_whitespace : true,  //no need for leading whitespaces
    };

    dnm.recurse_node_create(&root, &mut runtime);

    return dnm
  }

  /// Get the plaintext range of a node
  pub fn get_range_of_node (&self, node: &Node) -> Result<DNMRange, ()> {
    match self.node_map.get(&node_to_hashable(&node)) {
      Some(&(start, end)) => Ok(DNMRange {start:start, end:end, dnm: self}),
      None => Err(()),
    }
  }

  /// The heart of the dnm generation...
  fn recurse_node_create(&mut self, node: &Node, runtime: &mut RuntimeParseData) {
    match node.is_text_node() { 
      true => self.text_node_create(node, runtime),
      false => self.intermediate_node_create(node,runtime)
    };
  }

  fn text_node_create(&mut self, node : &Node, runtime : &mut RuntimeParseData) {
    let mut offset_start = self.plaintext.len();
    let mut still_in_leading_whitespaces = true;

    //possibly normalize unicode
    let mut content = if self.parameters.normalize_unicode { unidecode(&node.get_content()) } else { node.get_content() };
    if self.parameters.stem_words_once {
      content = rustmorpha::stem(&content);
    }
    if self.parameters.stem_words_full {
      content = rustmorpha::full_stem(&content);
    }
    if self.parameters.convert_to_lowercase {
      content = content.to_lowercase();
    }

    //if the option is set, reduce sequences of white spaces to single spaces
    if self.parameters.normalize_white_spaces {
      for c in content.to_string().chars() {
        if c.is_whitespace() {
          if runtime.had_whitespace { continue; }
          self.plaintext.push(' ');
          runtime.had_whitespace = true;
          if self.parameters.move_whitespaces_between_nodes {
            if still_in_leading_whitespaces {
              offset_start += 1;
            }
          }
        }
        else {
          self.plaintext.push(c);
          runtime.had_whitespace = false;
          still_in_leading_whitespaces = false;
        }
      }
    } else {
      self.plaintext.push_str(&content);
    }
    self.node_map.insert(node_to_hashable(node), (offset_start,
      if self.parameters.move_whitespaces_between_nodes && self.plaintext.len() > offset_start && runtime.had_whitespace {
        self.plaintext.len() - 1  //don't put trailing white space into node
      } else { self.plaintext.len() }));
    return;
  }

  fn intermediate_node_create(&mut self, node : &Node, runtime : &mut RuntimeParseData) {
    let offset_start = self.plaintext.len();
    let name : String = node.get_name();
    { // Start scope of self.parameters borrow, to allow mutable self borrow for recurse_node_create
    let mut rules = Vec::new();
    // First class rules, as more specific
    for classname in node.get_class_names() {
      let class_rule = self.parameters.special_tag_class_options.get(&classname);
      rules.push(class_rule);
    }
    // Then element rules as more general
    rules.push(self.parameters.special_tag_name_options.get(&name));

    for rule in rules {  //iterate over applying rules
      match rule {
        Some(&SpecialTagsOption::Enter) => break,
        Some(&SpecialTagsOption::Normalize(ref token)) => {
          if self.parameters.wrap_tokens {
            if !runtime.had_whitespace || !self.parameters.normalize_white_spaces {
              self.plaintext.push(' ');
            }
            self.plaintext.push_str(&token);
            self.plaintext.push(' ');
            runtime.had_whitespace = true;
          } else {
            self.plaintext.push_str(&token);
            //tokens are considered non-whitespace
            runtime.had_whitespace = false;
          }
          self.node_map.insert(node_to_hashable(node),
            (offset_start,
              if self.parameters.move_whitespaces_between_nodes && (self.plaintext.len() > offset_start) && runtime.had_whitespace {
                  self.plaintext.len() - 1    //don't put trailing white space into node
              } else { self.plaintext.len() }));
          return;
        },
        Some(&SpecialTagsOption::FunctionNormalize(f)) => {
          if self.parameters.wrap_tokens {
            if !runtime.had_whitespace ||
               !self.parameters.normalize_white_spaces {
                self.plaintext.push(' ');
            }
            self.plaintext.push_str(&f(&node));
            self.plaintext.push(' ');
            runtime.had_whitespace = true;
          } else {
            self.plaintext.push_str(&f(&node));
            //Return value of f is not considered a white space
            runtime.had_whitespace = false;
          }
          self.node_map.insert(node_to_hashable(node),
            (offset_start,
              if self.parameters.move_whitespaces_between_nodes && self.plaintext.len() > offset_start && runtime.had_whitespace {
                self.plaintext.len() - 1    //don't put trailing white space into node
              } else { self.plaintext.len() }));
          return;
        },
        Some(&SpecialTagsOption::Skip) => {
          self.node_map.insert(node_to_hashable(node),
            (offset_start,
            if self.parameters.move_whitespaces_between_nodes && self.plaintext.len() > offset_start && runtime.had_whitespace {
              self.plaintext.len() - 1    //don't put trailing white space into node
            } else { self.plaintext.len() }));
          return;
        },
        None => continue
      }
    }
    } // End scope of self.parameters borrow, to allow mutable self borrow for recurse_node_create
    // Recurse into children
    let mut child_option = node.get_first_child();
    loop {
      match child_option {
        Some(child) => {
          self.recurse_node_create(&child, runtime);
          child_option = child.get_next_sibling();
        },
        None => break
      }
    }

    self.node_map.insert(node_to_hashable(node), (offset_start, 
      if self.parameters.move_whitespaces_between_nodes && self.plaintext.len() > offset_start && runtime.had_whitespace {
        self.plaintext.len() - 1    //don't put trailing white space into node
      } else { self.plaintext.len()}));
  }
}

