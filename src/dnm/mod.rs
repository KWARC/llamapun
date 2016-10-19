//! The `dnm` can be used for easier switching between the DOM
//! (Document Object Model) representation and the plain text representation,
//! which is needed for most NLP tools.
mod range;
mod parameters;

extern crate libc;
extern crate unidecode;
extern crate rustmorpha;

use std::collections::HashMap;
use unidecode::unidecode;
use libxml::tree::*;
pub use dnm::range::DNMRange;
pub use dnm::parameters::{SpecialTagsOption, RuntimeParseData, DNMParameters};

/// The `DNM` is essentially a wrapper around the plain text representation
/// of the document, which facilitates mapping plaintext pieces to the DOM.
/// This breaks, if the DOM is changed after the DNM generation!
pub struct DNM {
  /// The plaintext
  pub plaintext: String,
  /// The options for generation
  pub parameters: DNMParameters,
  /// The root node of the underlying xml tree
  pub root_node: Node,
  /// Maps nodes to plaintext offsets
  pub node_map: HashMap<usize, (usize, usize)>,
  /// A runtime object used for holding auxiliary state
  // TODO: Would love to make the runtime a `private` field,
  //       but it requires some refactoring and rethinking the DNM-creation API
  pub runtime : RuntimeParseData
}

impl Default for DNM {
  fn default() -> DNM {
    DNM {
      parameters: DNMParameters::default(),
      root_node: Node::mock(),
      plaintext: String::new(),
      node_map: HashMap::new(),
      runtime: RuntimeParseData::default(),
    }
  }
}

// A handy macro for idiomatic recording in the node_map
#[macro_export]
macro_rules! record_node_map(
  ($dnm: expr, $node: expr, $offset_start: expr) => (
  {
    // Record plaintext range in node map
    let mut offset_end = $dnm.plaintext.len();
    if $dnm.parameters.move_whitespaces_between_nodes && $dnm.runtime.had_whitespace && offset_end > $offset_start {
      offset_end -= 1
    }
    $dnm.node_map.insert($node.to_hashable(), ($offset_start,offset_end));
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

    dnm
  }

  /// Get the plaintext range of a node
  pub fn get_range_of_node(&self, node: &Node) -> Result<DNMRange, ()> {
    match self.node_map.get(&node.to_hashable()) {
      Some(&(start, end)) => {
        Ok(DNMRange {
          start: start,
          end: end,
          dnm: self,
        })
      }
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
    let mut offset_start = self.plaintext.len();
    let mut still_in_leading_whitespaces = true;

    let mut content = node.get_content();
    if self.parameters.normalize_unicode {
      content = unidecode(&content);
    }
    if self.parameters.stem_words_once {
      content = rustmorpha::stem(&content);
    }
    if self.parameters.stem_words_full {
      content = rustmorpha::full_stem(&content);
    }
    if self.parameters.convert_to_lowercase {
      content = content.to_lowercase();
    }
    if self.parameters.normalize_white_spaces { // squash multiple spaces to a single one
      for content_char in content.chars() {
        if content_char.is_whitespace() {
          if self.runtime.had_whitespace {
            continue;
          }
          self.plaintext.push(' ');
          self.runtime.had_whitespace = true;
          if self.parameters.move_whitespaces_between_nodes && still_in_leading_whitespaces {
            offset_start += 1;
          }
        } else {
          self.plaintext.push(content_char);
          self.runtime.had_whitespace = false;
          still_in_leading_whitespaces = false;
        }
      }
    } else {
      self.plaintext.push_str(&content);
    }

    record_node_map!(self, node, offset_start);
    return;
  }

  fn intermediate_node_create(&mut self, node: &Node) {
    let offset_start = self.plaintext.len();
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
            if self.parameters.wrap_tokens {
              if !self.runtime.had_whitespace || !self.parameters.normalize_white_spaces {
                self.plaintext.push(' ');
              }
              self.plaintext.push_str(token);
              self.plaintext.push(' ');
              self.runtime.had_whitespace = true;
            } else {
              self.plaintext.push_str(token);
              // tokens are considered non-whitespace
              self.runtime.had_whitespace = false;
            }
            record_node_map!(self, node, offset_start);
            return;
          }
          Some(&SpecialTagsOption::FunctionNormalize(f)) => {
            if self.parameters.wrap_tokens {
              if !self.runtime.had_whitespace || !self.parameters.normalize_white_spaces {
                self.plaintext.push(' ');
              }
              self.plaintext.push_str(&f(node));
              self.plaintext.push(' ');
              self.runtime.had_whitespace = true;
            } else {
              self.plaintext.push_str(&f(node));
              // Return value of f is not considered a white space
              self.runtime.had_whitespace = false;
            }
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
