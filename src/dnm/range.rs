//! The `dnm::range` submodule provides data structures for indexing into a DNM object's plaintext

use dnm::DNM;
use libxml::tree::Node;
use libxml::xpath::Context;

/// Very often we'll talk about substrings of the plaintext - words, sentences,
/// etc. A `DNMRange` stores start and end point of such a substring and has
/// a reference to the `DNM`.
pub struct DNMRange<'dnmrange> {
  /// Offset of the beginning of the range
  pub start: usize,
  /// Offset of the end of the range
  pub end: usize,
  /// DNM containing this range
  pub dnm: &'dnmrange DNM,
}

impl<'dnmrange> Clone for DNMRange<'dnmrange> {
  fn clone(&self) -> DNMRange<'dnmrange> {
    DNMRange {
      start: self.start,
      end: self.end,
      dnm: self.dnm,
    }
  }
}

impl<'dnmrange> DNMRange<'dnmrange> {
  /// Get the plaintext substring corresponding to the range
  pub fn get_plaintext(&self) -> &'dnmrange str {
    &(self.dnm.plaintext)[self.dnm.byte_offsets[self.start]..self.dnm.byte_offsets[self.end]]
  }
  /// Get the plaintext without trailing white spaces
  pub fn get_plaintext_truncated(&self) -> &'dnmrange str {
    self.get_plaintext().trim_right()
  }

  /// Returns a `DNMRange` with the leading and trailing whitespaces removed
  pub fn trim(&self) -> DNMRange<'dnmrange> {
    let mut trimmed_start = self.start;
    let mut trimmed_end = self.end;
    let range_text: &str = self.get_plaintext();

    for c in range_text.chars() {
      if c.is_whitespace() {
        trimmed_start += 1;
      } else {
        break;
      }
    }
    for c in range_text.chars().rev() {
      if c.is_whitespace() {
        trimmed_end -= 1;
      } else {
        break;
      }
    }
    // Edge case: when the given input is whitespace only, start will be larger than end.
    // In that case return the 0-width range at the original end marker.
    if trimmed_start >= trimmed_end {
      trimmed_start = self.end;
      trimmed_end = self.end;
    }
    DNMRange {
      start: trimmed_start,
      end: trimmed_end,
      dnm: self.dnm,
    }
  }

  /// returns a subrange, with offsets relative to the beginning of `self`
  pub fn get_subrange(&self, rel_start: usize, rel_end: usize) -> DNMRange<'dnmrange> {
    DNMRange {
      start: self.start + rel_start,
      end: self.start + rel_end,
      dnm: self.dnm,
    }
  }

  /// returns a subrange from a pair of byte offsets (not character offsets, remember, we're in
  /// UTF-8)
  pub fn get_subrange_from_byte_offsets(&self, rel_start: usize, rel_end: usize) -> DNMRange<'dnmrange> {
    DNMRange {
      start : self.byte_offset_bisection(self.dnm.byte_offsets[self.start] + rel_start, self.start, self.end),
      end : self.byte_offset_bisection(self.dnm.byte_offsets[self.start] + rel_end - 1, self.start, self.end)+1,
      dnm : self.dnm
    }
  }

  fn byte_offset_bisection(&self, target_byte: usize, lower_char: usize, upper_char: usize) -> usize {
    if lower_char == upper_char {
      return lower_char;
    } else if upper_char == lower_char + 1 {
      if self.dnm.byte_offsets[upper_char] <= target_byte {
        return upper_char;
      } else {
        return lower_char;
      }
    }

    let middle_char = (lower_char + upper_char)/2;
    if self.dnm.byte_offsets[middle_char] > target_byte {
      self.byte_offset_bisection(target_byte, lower_char, middle_char)
    } else {
      self.byte_offset_bisection(target_byte, middle_char, upper_char)
    }
  }

  /// checks whether the range is empty
  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }



  /*
   * SERIALIZATION CODE
   */


  /// serializes a DNMRange into an XPointer
  pub fn serialize(&self) -> String {
    if !self.dnm.parameters.support_back_mapping {
      panic!("DNMRange::serialize: DNM did not generate the back_map");
    }
    let (ref node1, offset1) = self.dnm.back_map[self.start];
    let (ref node2, offset2) = self.dnm.back_map[self.end];
    DNMRange::create_arange(&DNMRange::serialize_offset(&self.dnm.root_node, node1, offset1, false),
                            &DNMRange::serialize_offset(&self.dnm.root_node, node2, offset2, true))
  }

  /// creates an arange from to xpointers
  pub fn create_arange(from: &str, to: &str) -> String {
    format!("arange({},{})", from, to)
  }

  /// Serializes a node and an offset into an xpointer
  /// is_end indicates whether the node indicates the end of the interval
  pub fn serialize_offset(root_node: &Node, node: &Node, offset: i32, is_end: bool) -> String {
    if offset < 0 {
      DNMRange::serialize_node(root_node, &node, is_end)
    } else {
      format!("string-index({},{})", DNMRange::serialize_node(root_node, node, is_end),
                                     &(offset+1).to_string())
    }
  }

  /// serializes a node into an xpath expression
  pub fn serialize_node(root_node: &Node, node: &Node, is_end: bool) -> String {
    match node.get_property("id") {
      None => {
        if node == root_node {
          return "/".to_string();
        }
        if node.is_text_node() {
          let parent = node.get_parent().unwrap();
          let base = DNMRange::serialize_node(root_node, &parent, false /* don't take next */);
          return format!("{}/text()[{}]", base,
                get_node_number(&parent, &node, &| n : &Node | n.is_text_node()).unwrap());
        } else {
          let act = if is_end { get_next_sibling(root_node, node).unwrap_or(node.clone()) } else { node.clone() };
          let parent = act.get_parent().unwrap();
          let base = DNMRange::serialize_node(root_node, &parent, false /* don't take next */ );
          return format!("{}/{}[{}]",
                         base,
                         if act.is_text_node() {
                           "text()".to_string()
                         } else {
                           if let Some(ns) = act.get_namespace() {
                             let prefix = ns.get_prefix();
                             if prefix == "" {  // default namespace without prefix
                               format!("*[local-name() = '{}']", act.get_name())
                             } else {
                               format!("{}:{}", prefix, act.get_name())
                             }
                           } else {
                             act.get_name()
                           }
                         },
                         get_node_number(&parent, &act, &| n : &Node | n.get_name() == act.get_name()).unwrap());
        }
      },
      Some(x) => format!("//*[@id=\"{}\"]", x),
    }
  }

  /*
   * DESERIALIZATION CODE
   */


  /// deserializes an xpointer into a `DNMRange`. Note that only a very limited subset of xpointers
  /// is supported. Essentially, you should not use it for deserialization of xpointers generated by
  /// any other tool. (TODO: Support a wider range of xpointers)
  pub fn deserialize(string : &str, dnm : &'dnmrange DNM, xpath_context : &Context) -> DNMRange<'dnmrange> {
    assert_eq!(&(string[0..7]), "arange(");
    assert_eq!(&(string[string.len() - 1..string.len()]), ")");

    let main_comma = 1 + string.find("),")
                               .unwrap_or(string.find("],")
                                                .expect(&format!("DNMRange::deserialize: Malformed string: \"{}\"", string)));

    let start_str = &string[7..main_comma];
    let end_str = &string[main_comma+1..string.len()-1];

    let start_pos = DNMRange::xpointer_to_offset(&start_str, dnm, xpath_context);
    let end_pos = DNMRange::xpointer_to_offset(&end_str, dnm, xpath_context);

    return DNMRange {
      dnm : dnm,
      start : start_pos,
      end : end_pos
    };
  }


  /// Gets the plaintext offset corresponding to an XPath/string-index'ed XPointer,
  /// again, does not cover everything!
  fn xpointer_to_offset(string : &str, dnm : &'dnmrange DNM, xpath_context : &Context) -> usize {
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
          while pos < range.end && &dnm.back_map[pos].1 < &offset { pos += 1; }
          return pos;
        }
        Err(()) => {
          return get_position_of_lowest_parent(&node, dnm);
        }
      }
    } else {
      let node_str = string;
      let node_set = xpath_context.evaluate(&node_str)
        .expect(&format!("DNMRange::deserialize: Malformed XPath: '{}'", &node_str));
      assert_eq!(node_set.get_number_of_nodes(), 1);
      let node = node_set.get_nodes_as_vec()[0].clone();
      return get_position_of_lowest_parent(&node, dnm);
    }
  }
}


/*
 * (DE)?SERIALIZATION HELPER FUNCTIONS
 */


/// Helper function: Gets the start offset of the lowest parent recorded in the DNM
fn get_position_of_lowest_parent(node : &Node, dnm : &DNM) -> usize {
  match dnm.get_range_of_node(node) {
    Ok(range) =>
      range.start,
      Err(()) =>
        get_position_of_lowest_parent(&(node.get_parent().unwrap()), dnm)
  }
}

/// Helper function: Returns the next sibling of a node if it exists
/// (goes up in the tree if required)
fn get_next_sibling(root_node: &Node, node: &Node) -> Option<Node> {
  match node.get_next_sibling() {
    None => {
      if node == root_node {
        println_stderr!("DNMRange::serialize: Warning: Can't annotate last node in document properly");
        None
      } else {
        get_next_sibling(root_node, &node.get_parent().unwrap())
      }
    }
    Some(n) => Some(n)
  }
}


/// Helper function: Returns the number of a node (the how many-th sibling of its kind it is)
fn get_node_number(parent: &Node, target: &Node, rule: &Fn (&Node) -> bool) -> Result<i32, ()> {
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


