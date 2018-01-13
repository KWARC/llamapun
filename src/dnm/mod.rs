//! The `dnm` can be used for easier switching between the DOM
//! (Document Object Model) representation and the plain text representation,
//! which is needed for most NLP tools.
mod range;
mod parameters;
mod c14n;

extern crate libc;
extern crate rustmorpha;
extern crate unidecode;

use std::collections::HashMap;
use unidecode::{unidecode, unidecode_char};
use libxml::tree::*;
pub use dnm::range::DNMRange;
pub use dnm::parameters::{DNMParameters, RuntimeParseData, SpecialTagsOption};

/// The `DNM` is essentially a wrapper around the plain text representation
/// of the document, which facilitates mapping plaintext pieces to the DOM.
/// This breaks, if the DOM is changed after the DNM generation!
pub struct DNM {
  /// The plaintext
  pub plaintext: String,
  /// As the plaintext is UTF-8: the byte offsets of the characters
  pub byte_offsets: Vec<usize>,
  /// The options for generation
  pub parameters: DNMParameters,
  /// The root node of the underlying xml tree
  pub root_node: Node,
  /// Maps nodes to plaintext offsets
  pub node_map: HashMap<usize, (usize, usize)>,
  /// A runtime object used for holding auxiliary state
  // TODO: Would love to make the runtime a `private` field,
  //       but it requires some refactoring and rethinking the DNM-creation API
  pub runtime: RuntimeParseData,
  /// maps an offset to the corresponding node, and the offset in the node
  /// offset -1 means that the offset corresponds to the entire node
  /// this is e.g. used if a node is replaced by a token.
  pub back_map: Vec<(Node, i32)>,
}

impl Default for DNM {
  fn default() -> DNM {
    DNM {
      parameters: DNMParameters::default(),
      root_node: Node::null(),
      plaintext: String::new(),
      byte_offsets: Vec::new(),
      node_map: HashMap::new(),
      runtime: RuntimeParseData::default(),
      back_map: Vec::new(),
    }
  }
}

// A handy macro for idiomatic recording in the node_map
#[macro_export]
macro_rules! record_node_map(
  ($dnm: expr, $node: expr, $offset_start: expr) => (
  {
    $dnm.node_map.insert($node.to_hashable(), ($offset_start, $dnm.runtime.chars.len()));
  }
  )
);

macro_rules! push_token(
  ($dnm: expr, $token: expr, $node: expr) => (
  {
    if $dnm.parameters.wrap_tokens {
      push_whitespace!($dnm, $node, -1);
    }

    if !$dnm.parameters.support_back_mapping {
      $dnm.runtime.chars.extend($token.chars());
    } else {
      for c in $token.chars() {
        $dnm.runtime.chars.push(c);
        $dnm.back_map.push(($node.clone(), -1));
      }
    }
    $dnm.runtime.had_whitespace = false;

    if $dnm.parameters.wrap_tokens {
      push_whitespace!($dnm, $node, -1);
    }
  }
  )
);

macro_rules! push_whitespace(
  ($dnm: expr, $node: expr, $offset: expr) => (
  {
    if !$dnm.runtime.had_whitespace || !$dnm.parameters.normalize_white_spaces {
      $dnm.runtime.chars.push(' ');
      $dnm.runtime.had_whitespace = true;
      if $dnm.parameters.support_back_mapping {
        $dnm.back_map.push(($node.clone(), $offset));
      }
      true
    } else {
      false
    }
  }
  )
);

impl DNM {
  /// Creates a `DNM` for `root`
  pub fn new(root: Node, parameters: DNMParameters) -> DNM {
    parameters.check();
    let mut dnm = DNM {
      parameters: parameters,
      root_node: root.clone(),
      ..DNM::default()
    };

    // Depth-first traversal of the DOM extracting a plaintext representation and building a node<->text map.
    dnm.recurse_node_create(&root);

    // generate plaintext
    assert_eq!(dnm.plaintext.len(), 0);
    for c in &dnm.runtime.chars {
      dnm.byte_offsets.push(dnm.plaintext.len());
      dnm.plaintext.push(*c);
    }
    dnm.byte_offsets.push(dnm.plaintext.len()); // to have the length of the last char as well

    dnm
  }

  /// Get the plaintext range of a node
  pub fn get_range_of_node(&self, node: &Node) -> Result<DNMRange, ()> {
    match self.node_map.get(&node.to_hashable()) {
      Some(&(start, end)) => Ok(DNMRange {
        start: start,
        end: end,
        dnm: self,
      }),
      None => Err(()),
    }
  }

  /// The heart of the dnm generation...
  fn recurse_node_create(&mut self, node: &Node) {
    if node.is_text_node() {
      self.text_node_create(node)
    } else {
      self.intermediate_node_create(node)
    }
  }

  fn text_node_create(&mut self, node: &Node) {
    let offset_start = self.runtime.chars.len();
    let mut string = node.get_content();
    let mut offsets: Vec<i32> = if self.parameters.support_back_mapping {
      (0i32..(string.chars().count() as i32)).collect()
    } else {
      Vec::new()
    };

    // string processing steps
    self.normalize_unicode(&mut string, &mut offsets);
    self.stem_words(&mut string /*, &mut offsets */);
    if self.parameters.convert_to_lowercase {
      string = string.to_lowercase();
    }
    self.normalize_whitespace(&mut string, &mut offsets);

    // push results
    self.runtime.chars.extend(string.chars());
    if self.parameters.support_back_mapping {
      assert_eq!(string.chars().count(), offsets.len());
      for offset in offsets {
        self.back_map.push((node.clone(), offset));
      }
    }

    record_node_map!(self, node, offset_start);
    return;
  }

  fn normalize_whitespace(&mut self, string: &mut String, offsets: &mut Vec<i32>) {
    if !self.parameters.normalize_white_spaces {
      return;
    }
    let mut new_string = String::new();
    let mut new_offsets: Vec<i32> = Vec::new();

    for (i, c) in string.chars().enumerate() {
      if c.is_whitespace() {
        if !self.runtime.had_whitespace {
          self.runtime.had_whitespace = true;
          new_string.push(' ');
          if self.parameters.support_back_mapping {
            new_offsets.push(offsets[i]);
          }
        }
      } else {
        new_string.push(c);
        self.runtime.had_whitespace = false;
        if self.parameters.support_back_mapping {
          new_offsets.push(offsets[i]);
        }
      }
    }

    *string = new_string;
    *offsets = new_offsets;
  }

  fn normalize_unicode(&self, string: &mut String, offsets: &mut Vec<i32>) {
    if !self.parameters.normalize_unicode {
      return;
    }
    if !self.parameters.support_back_mapping {
      *string = unidecode(&string);
      return;
    }

    // the tricky part: unidecode can replace a character by multiple characters.
    // We need to maintain the offsets for back mapping
    let mut new_string = String::new();
    let mut new_offsets: Vec<i32> = Vec::new();

    for (i, co) in string.chars().enumerate() {
      for cn in unidecode_char(co).chars() {
        new_string.push(cn);
        new_offsets.push(offsets[i]);
      }
    }

    *string = new_string;
    *offsets = new_offsets;
  }

  fn stem_words(&self, string: &mut String /*, offsets : &mut Vec<i32> */) {
    // TODO: Support back-mapping (using e.g. something like min. edit distance to map offsets)
    if self.parameters.support_back_mapping
      && (self.parameters.stem_words_full || self.parameters.stem_words_once)
    {
      panic!("llamapun::dnm: word stemming does not support back-mapping yet");
    }
    if self.parameters.stem_words_full {
      *string = rustmorpha::full_stem(&string);
    } else if self.parameters.stem_words_once {
      *string = rustmorpha::stem(&string);
    }
  }

  fn intermediate_node_create(&mut self, node: &Node) {
    let offset_start = self.runtime.chars.len();
    let name: String = node.get_name();
    {
      // Start scope of self.parameters borrow, to allow mutable self borrow for recurse_node_create
      let mut rules = Vec::new();
      // First class rules, as more specific
      for classname in node.get_class_names() {
        let class_rule = self.parameters.special_tag_class_options.get(&classname);
        rules.push(class_rule);
      }
      // Then element rules as more general
      rules.push(self.parameters.special_tag_name_options.get(&name));

      for rule in rules {
        // iterate over applying rules
        match rule {
          Some(&SpecialTagsOption::Enter) => break,
          Some(&SpecialTagsOption::Normalize(ref token)) => {
            push_token!(self, token, node);
            record_node_map!(self, node, offset_start);
            return;
          }
          Some(&SpecialTagsOption::FunctionNormalize(ref f)) => {
            push_token!(self, &f(node), node);
            record_node_map!(self, node, offset_start);
            return;
          }
          Some(&SpecialTagsOption::Skip) => {
            record_node_map!(self, node, offset_start);
            return;
          }
          None => continue,
        }
      }
    } // End scope of self.parameters borrow, to allow mutable self borrow for recurse_node_create
      // Recurse into children
    if let Some(child) = node.get_first_child() {
      self.recurse_node_create(&child);
      let mut child_node = child;
      while let Some(child) = child_node.get_next_sibling() {
        self.recurse_node_create(&child);
        child_node = child;
      }
    }
    record_node_map!(self, node, offset_start);
  }
}
