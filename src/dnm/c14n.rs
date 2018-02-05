//! The `dnm::c14n` submodule offers lightweight canonicalization for a DOM node.
//!      The core purpose for canonicalization is linguistic comparison,
//!      so the c14n module tries to strip away markup artefacts unrelated to the underlying content, such as xml:ids.
use libxml::tree::Node;
use libxml::tree::NodeType::{ElementNode, TextNode};
use crypto::md5::Md5;
use crypto::digest::Digest;
use std::sync::Mutex;
use dnm::DNM;

lazy_static! {
  static ref MD5_HASHER : Mutex<Md5> = Mutex::new(Md5::new());
}

impl DNM {
  /// Our linguistic canonical form will only include 1) node name, 2) class attribute and 3) textual content
  ///  - excludes certain experimental markup, such as all math annotation elements
  ///  - excludes whitespace nodes and comment nodes
  pub fn to_c14n_basic(&self) -> String { self.node_c14n_basic(&self.root_node) }

  /// Canonicalize a single node of choice
  pub fn node_c14n_basic(&self, node: &Node) -> String {
    let mut canonical_node = String::new();
    self.canonical_internal(node, None, &mut canonical_node);
    canonical_node
  }

  /// Obtain an MD5 hash from the canonical string of the entire DOM
  pub fn to_hash_basic(&self) -> String { self.node_hash_basic(&self.root_node) }

  /// Obtain an MD5 hash from the canonical string of a Node
  pub fn node_hash_basic(&self, node: &Node) -> String {
    let mut hasher = MD5_HASHER.lock().unwrap();
    hasher.reset();
    hasher.input_str(&self.node_c14n_basic(node));
    hasher.result_str()
  }

  fn canonical_internal(&self, node: &Node, indent: Option<u32>, mut canonical_node: &mut String) {
    // Bookkeep indents, if requested
    let indent_string = match indent {
      Some(level) => String::new() + "\n" + &(1..level).map(|_| " ").collect::<String>(),
      None => String::new(),
    };
    let next_indent_level = match indent {
      Some(level) => Some(level + 2),
      None => None,
    };

    match node.get_type() {
      Some(TextNode) => {
        if let Ok(range) = self.get_range_of_node(node) {
          let text = range.get_plaintext();
          if !text.trim().is_empty() {
            canonical_node.push_str(&indent_string);
            canonical_node.push_str(text);
          } else {
            // ignore empty nodes
          }
        }
      },
      Some(ElementNode) => {
        // Skip artefact nodes
        let name: String = node.get_name();
        if (name == "annotation") || (name == "annotation-xml") {
          return;
        }

        // Open the current node
        if name != "semantics" {
          // ignore unwrappable nodes
          canonical_node.push_str(&indent_string);
          canonical_node.push('<');
          canonical_node.push_str(&name);

          let class_attr = node.get_property("class").unwrap_or_default();
          let mut classes_split = class_attr.split_whitespace().collect::<Vec<_>>();
          if !classes_split.is_empty() {
            classes_split.sort();
            canonical_node.push_str(" class=\"");
            for class_value in classes_split {
              canonical_node.push_str(class_value);
              canonical_node.push(' ');
            }
            canonical_node.pop();
            canonical_node.push('"');
          }
          canonical_node.push_str(">");
        }

        // Recurse into children
        if let Some(child) = node.get_first_child() {
          self.canonical_internal(&child, next_indent_level, &mut canonical_node);
          let mut child_node = child;

          while let Some(child) = child_node.get_next_sibling() {
            self.canonical_internal(&child, next_indent_level, &mut canonical_node);
            child_node = child;
          }
        }

        // Close the current node
        if name != "semantics" {
          // ignore unwrappable nodes
          canonical_node.push_str(&indent_string);
          canonical_node.push_str("</");
          canonical_node.push_str(&name);
          canonical_node.push_str(">");
        }
      },
      _ => {
        println!("-- Skipping node {:?}", node.get_name());
      }, // skip all other node types for now
    }
    return;
  }
}
