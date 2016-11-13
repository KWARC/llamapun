//! The `dnm` can be used for easier switching between the DOM
//! (Document Object Model) representation and the plain text representation,
//! which is needed for most NLP tools.

extern crate libc;
extern crate unidecode;
extern crate rustmorpha;

use std::collections::HashMap;
use std::mem;
use std::io::Write;
use unidecode::{unidecode, unidecode_char};
use libxml::tree::*;
use libxml::xpath::{Context};


/// Specifies how to deal with a certain tag
pub enum SpecialTagsOption {
  /// Recurse into tag (default behaviour)
  Enter,
  /// Normalize tag, replacing it by some token
  Normalize(String),
  /// Normalize tag, obtain replacement string by function call
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
  /// Replace unicode characters by the ascii code representation
  pub normalize_unicode: bool,
  /// Apply the morpha stemmer once to the text nodes
  pub stem_words_once: bool,
  /// Apply the morpha stemmer to the text nodes
  /// as often as it changes something
  pub stem_words_full: bool,
  /// Move to lowercase (remark: The stemmer does this automatically)
  pub convert_to_lowercase: bool,
  /// Support mapping plaintext offsets back to the DOM
  pub support_back_mapping: bool,
}

impl Default for DNMParameters {
  /// Don't do anything fancy and specific by default
  fn default() -> DNMParameters {
    DNMParameters {
      special_tag_name_options : HashMap::new(),
      special_tag_class_options : HashMap::new(),
      normalize_white_spaces: true,
      wrap_tokens: false,
      normalize_unicode: false,
      stem_words_once: false,
      stem_words_full: false,
      convert_to_lowercase: false,
      support_back_mapping: false,
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
    if (self.stem_words_once || self.stem_words_full)
        && self.convert_to_lowercase {
      println_stderr!("llamapun::dnm: Parameter option convert_to_lowercase\
  is redundant, because stemming converts to lowercase already");
    }
    if self.support_back_mapping && (self.stem_words_once || self.stem_words_full) {
        panic!("llamapun::dnm: Parameter option support_back_mapping\
        does not work in combination with options stem_words_once or stem_words_full");
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
  pub node_map : HashMap<usize, (usize, usize)>,
  /// Maps offsets to the corresponding (lowest) node
  pub offset_to_node : Vec<Node>,
  /// Maps an offset i to the corresponding string offset in offset_to_node[i]
  pub offset_to_node_offset : Vec<i32>,
}


/// Some temporary data for the parser
struct ParsingContext {
  /// plaintext is currently terminated by some whitespace
  had_whitespace : bool,
  plaintext : String,
  node_map : HashMap<usize, (usize, usize)>,
  pub offset_to_node : Vec<Node>,
  pub offset_to_node_offset : Vec<i32>,
}

/// Very often we'll talk about substrings of the plaintext - words, sentences,
/// etc. A `DNMRange` stores start and end point of such a substring and has
/// a reference to the `DNM`.
pub struct DNMRange <'dnmrange> {
  /// Offset of the beginning of the range
  pub start : usize,
  /// Offset of the end of the range
  pub end : usize,
  /// DNM containing this range
  pub dnm : &'dnmrange DNM,
}

impl <'dnmrange> DNMRange <'dnmrange> {
  /// Get the plaintext substring corresponding to the range
  pub fn get_plaintext(&self) -> &str {
    &(&self.dnm.plaintext)[self.start..self.end]
  }
  /// Get the plaintext without trailing white spaces
  pub fn get_plaintext_truncated(&self) -> &'dnmrange str {
    &(&self.dnm.plaintext)[self.start..self.end].trim_right()
  }

  /// Returns a `DNMRange` with the leading and trailing whitespaces removed
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
    // Edge case: when the given input is whitespace only, start will be larger than end.
    // In that case return the 0-width range at the original end marker.
    if trimmed_start >= trimmed_end {
      trimmed_start = self.end;
      trimmed_end = self.end;
    }
    DNMRange {start : trimmed_start, end: trimmed_end, dnm: self.dnm}
  }

  /// returns a subrange, with offsets relative to the beginning of `self`
  pub fn get_subrange(&self, rel_start: usize, rel_end: usize) -> DNMRange<'dnmrange> {
    DNMRange {start: self.start + rel_start, end: self.start + rel_end, dnm: self.dnm}
  }

  /// checks whether the range is empty
  pub fn is_empty(&self) -> bool {
      self.start == self.end
  }

  /// Serializes a pair of nodes and offsets into an xpointer
  pub fn serialize_core(&self, node1: &Node, offset1: i32, node2: &Node, offset2: i32) -> String {
        let mut s = String::new();
        s.push_str("arange(");
        s.push_str(&self.serialize_offset(node1, offset1, false));
        s.push_str(",");
        s.push_str(&self.serialize_offset(node2, offset2, true));
        s.push_str(")");
        return s;
  }

  /// Serializes a node and an offset into an xpointer
  pub fn serialize_offset(&self, node: &Node, offset: i32, is_end: bool) -> String {
      if offset < 0 {
          return self.serialize_node(&node, is_end);
      } else {
          let mut s = String::new();
          s.push_str("string-index(");
          s.push_str(&self.serialize_node(&node, is_end));
          s.push_str(",");
          s.push_str(&(offset+1).to_string());
          s.push_str(")");
          return s;
      }
  }

  /// serializes a node into an xpath expression
  pub fn serialize_node(&self, node: &Node, is_end: bool) -> String {
      match node.get_property("id") {
          None => {
              if *node == self.dnm.root_node {
                  return "/".to_string();
              }
              if node.is_text_node() {
                let parent = node.get_parent().unwrap();
                let base = self.serialize_node(&parent, false /* don't take next */);
                return format!("{}/text()[{}]", base, self.get_node_number(&parent, &node, &| n : &Node | n.is_text_node()).unwrap());
              } else {
                let act = if is_end { self.get_next_sibling(node).unwrap_or(node.clone()) } else { node.clone() };
                let parent = act.get_parent().unwrap();
                let base = self.serialize_node(&parent, false /* don't take next */ );
                return format!("{}/{}[{}]", base,
                               if act.is_text_node() { "text()".to_string() } else { act.get_name() },
                               self.get_node_number(&parent, &act, &| n : &Node | n.get_name() == act.get_name()).unwrap());
              }
          },
          Some(x) => format!("//*[@id=\"{}\"]", x),
      }
  }

  fn get_next_sibling(&self, node: &Node) -> Option<Node> {
    match node.get_next_sibling() {
        None => {
            if *node == self.dnm.root_node {
                println_stderr!("DNMRange::serialize: Warning: Can't annotate last node in document properly");
                None
            } else {
                self.get_next_sibling(&node.get_parent().unwrap())
            }
        }
        Some(n) => Some(n)
    }
  }

  fn get_node_number(&self, parent: &Node, target: &Node, rule: &Fn (&Node) -> bool) -> Result<i32, ()> {
    let mut cur = parent.get_first_child().expect("can't get child number - node has no children");
    let mut count = 1i32;
    while cur != *target {
        if rule(&cur) {
            count += 1;
        }
        match cur.get_next_sibling() {
            None => { return Err(()); },
            Some(n) => { cur = n; }
        }
    }
    return Ok(count);
  }

  /// serializes a DNMRange
  pub fn serialize(&self) -> String {
      return self.serialize_core(&self.dnm.offset_to_node[self.start], self.dnm.offset_to_node_offset[self.start],
                                 &self.dnm.offset_to_node[self.end],   self.dnm.offset_to_node_offset[self.end]);
  }

  fn get_position_of_lowest_parent(node : &Node, dnm : &'dnmrange DNM) -> usize {
      match dnm.get_range_of_node(node) {
          Ok(range) =>
              range.start,
          Err(()) =>
              DNMRange::get_position_of_lowest_parent(&(node.get_parent().unwrap()), dnm)
      }
  }


  fn deserialize_part(string : &str, dnm : &'dnmrange DNM, xpath_context : &Context) -> usize {
    if string.len() > 13 && &(string[0..13]) == "string-index(" {
        let comma = string.find(',').expect(&format!("DNM::deserialize_part: Malformed string: \"{}\"", string));
        let node_str = &string[13..comma];
        let node_set = xpath_context.evaluate(&node_str).unwrap();
        assert_eq!(node_set.get_number_of_nodes(), 1);
        let node = node_set.get_nodes_as_vec()[0].clone();
        match dnm.get_range_of_node(&node) {
            Ok(range) => {
                let mut pos = range.start;
                let offset = &string[comma+1..string.len()-1].parse::<i32>().unwrap()-1;
                while pos < range.end && &dnm.offset_to_node_offset[pos] < &offset { pos += 1; }
                return pos;
            }
            Err(()) => {
                return DNMRange::get_position_of_lowest_parent(&node, dnm);
            }
        }
    } else {
        let node_str = string;
        let node_set = xpath_context.evaluate(&node_str)
            .expect(&format!("DNMRange::deserialize: Malformed XPath: '{}'", &node_str));
        assert_eq!(node_set.get_number_of_nodes(), 1);
        let node = node_set.get_nodes_as_vec()[0].clone();
        return DNMRange::get_position_of_lowest_parent(&node, dnm);
    }
  }

  /// deserializes an xpointer into a `DNMRange`. Note that only a very limited subset of xpointers
  /// is supported. Essentially, you should not use it for deserialization of xpointers generated by
  /// any other tool
  pub fn deserialize(string : &str, dnm : &'dnmrange DNM, xpath_context : &Context) -> DNMRange<'dnmrange> {
    assert_eq!(&(string[0..7]), "arange(");
    assert_eq!(&(string[string.len() - 1..string.len()]), ")");

    let main_comma = 1 + string.find("),").unwrap_or(string.find("],").
                expect(&format!("DNMRange::deserialize: Malformed string: \"{}\"", string)));

    let start_str = &string[7..main_comma];
    let end_str = &string[main_comma+1..string.len()-1];

    let start_pos = DNMRange::deserialize_part(&start_str, dnm, xpath_context);
    let end_pos = DNMRange::deserialize_part(&end_str, dnm, xpath_context);

    return DNMRange {
        dnm : dnm,
        start : start_pos,
        end : end_pos
    };

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
      offset_to_node : Vec::new(),
      offset_to_node_offset : Vec::new(),
    };

    let mut context = ParsingContext {
      had_whitespace : true,  //no need for leading whitespaces
      plaintext : String::new(),
      node_map : HashMap::new(),
      offset_to_node : Vec::new(),
      offset_to_node_offset : Vec::new(),
    };

    dnm.handle_node(&root, &mut context);

    dnm.plaintext = context.plaintext;
    dnm.node_map = context.node_map;
    dnm.offset_to_node = context.offset_to_node;
    dnm.offset_to_node_offset = context.offset_to_node_offset;

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
  fn handle_node(&self, node: &Node, context: &mut ParsingContext) {
    match node.is_text_node() {
      true => self.handle_text_node(node, context),
      false => self.handle_intermediate_node(node,context)
    };
  }

  fn handle_text_node(&self, node : &Node, ctx : &mut ParsingContext) {
    let offset_start = ctx.plaintext.len();
    
    let mut string : String = node.get_content();
    let mut offsets : Vec<i32> = if self.parameters.support_back_mapping { (0i32..(string.len() as i32)).collect() } else { Vec::new() };
    
    self.normalize_unicode(&mut string, &mut offsets);
    
    self.stem_words(&mut string /*, &mut offsets */);
    
    if self.parameters.convert_to_lowercase {
      string = string.to_lowercase();
    }

    self.normalize_whitespace(&mut string, &mut offsets, ctx);

    ctx.plaintext.push_str(&string);
    if self.parameters.support_back_mapping {
        assert_eq!(string.len(), offsets.len());
        for offset in offsets {
            ctx.offset_to_node_offset.push(offset);
            ctx.offset_to_node.push(node.clone());
        }
    }
        

    self.insert_node_into_node_map(ctx, node, offset_start);
  }

  fn normalize_whitespace(&self, string : &mut String, offsets : &mut Vec<i32>, context : &mut ParsingContext) {
    if !self.parameters.normalize_white_spaces { return; }
    let mut new_string = String::new();
    let mut new_offsets : Vec<i32> = Vec::new();

    for (i, c) in string.chars().enumerate() {
        if c.is_whitespace() {
            if !context.had_whitespace {
                context.had_whitespace = true;
                new_string.push(' ');
                if self.parameters.support_back_mapping {
                    new_offsets.push(offsets[i]);
                }
            }
        } else {
            new_string.push(c);
            context.had_whitespace = false;
            if self.parameters.support_back_mapping {
                new_offsets.push(offsets[i]);
            }
        }
    }

    *string = new_string;
    *offsets = new_offsets;
  }

  fn normalize_unicode(&self, string : &mut String, offsets : &mut Vec<i32>) {
    if !self.parameters.normalize_unicode  { return; }
    if !self.parameters.support_back_mapping {
        *string = unidecode(&string);
        return;
    }

    let mut new_string = String::new();
    let mut new_offsets : Vec<i32> = Vec::new();

    for (i, co) in string.chars().enumerate() {
        for cn in unidecode_char(co).chars() {
            new_string.push(cn);
            new_offsets.push(offsets[i]);
        }
    }

    *string = new_string;
    *offsets = new_offsets;
  }

  fn stem_words(&self, string : &mut String /*, offsets : &mut Vec<i32> */) {
      // TODO: Support back-mapping (using e.g. something like min. edit distance to map offsets)
      if self.parameters.support_back_mapping && (self.parameters.stem_words_full || self.parameters.stem_words_once) {
          panic!("llamapun::dnm: word stemming does not support back-mapping yet");
      }
      if self.parameters.stem_words_full {
          *string = rustmorpha::full_stem(&string);
      } else if self.parameters.stem_words_once {
          *string = rustmorpha::stem(&string);
      }

  }

  fn push_whitespace(&self, context : &mut ParsingContext, node : &Node, offset : i32) -> bool {
      if !context.had_whitespace || !self.parameters.normalize_white_spaces {
          context.plaintext.push(' ');
          context.had_whitespace = true;
          if self.parameters.support_back_mapping {
              context.offset_to_node_offset.push(offset);
              context.offset_to_node.push(node.clone());
          }
          return true;
      }
      return false;
  }

  fn push_token(&self, context : &mut ParsingContext, token : &str, node : &Node) {
      if self.parameters.wrap_tokens {
          self.push_whitespace(context, node, -1);
      }

      if !self.parameters.support_back_mapping {
          context.plaintext.push_str(token);
      } else {
          for c in token.chars() {
              context.plaintext.push(c);
              context.offset_to_node.push(node.clone());
              context.offset_to_node_offset.push(-1);
          }
      }
      context.had_whitespace = false;  // disputable, but I don't consider tokens as whitespaces

      if self.parameters.wrap_tokens {
          self.push_whitespace(context, node, -1);
      }
  }

  fn insert_node_into_node_map(&self, context : &mut ParsingContext, node : &Node, start : usize) {
      context.node_map.insert(node_to_hashable(node), (start, context.plaintext.len()));
  }

  fn handle_intermediate_node(&self, node : &Node, context : &mut ParsingContext) {
    let offset_start = context.plaintext.len();
    let name : String = node.get_name();
    let mut rules = Vec::new();
    // First class rules, as more specific
    for classname in node.get_class_names() {
      let class_rule = self.parameters.special_tag_class_options.get(&classname);
      rules.push(class_rule.clone());
    }
    // Then element rules as more general
    rules.push(self.parameters.special_tag_name_options.get(&name).clone());

    for rule in rules {  //iterate over applying rules
      match rule {
        Some(&SpecialTagsOption::Enter) => break,
        Some(&SpecialTagsOption::Normalize(ref token)) => {
          self.push_token(context, &token, node);
          self.insert_node_into_node_map(context, node, offset_start);
          return;
        },
        Some(&SpecialTagsOption::FunctionNormalize(f)) => {
          self.push_token(context, &f(&node), node);
          self.insert_node_into_node_map(context, node, offset_start);
          return;
        },
        Some(&SpecialTagsOption::Skip) => {
          self.insert_node_into_node_map(context, node, offset_start);
          return;
        },
        None => continue
      }
    }

    // Recurse into children
    let mut child_option = node.get_first_child();
    loop {
      match child_option {
        Some(child) => {
          self.handle_node(&child, context);
          child_option = child.get_next_sibling();
        },
        None => break
      }
    }

    self.insert_node_into_node_map(context, node, offset_start);
  }
}

