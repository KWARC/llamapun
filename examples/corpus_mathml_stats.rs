// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//

//! Given a `CorTeX` corpus of HTML5 documents, extract statistics on MathML use
//! as per https://github.com/mathml-refresh/mathml/issues/55#issuecomment-474768228
//!
//! example use for arXMLiv:
//!    `cargo run --release --example corpus_mathml_stats /data/datasets/dataset-arXMLiv-08-2018 arxmliv_mathml_statistics.txt`
//! example use for DLMF:
//!    `cargo run --release --example corpus_mathml_stats /var/local/dlmf dlmf_mathml_statistics.txt .html5`

extern crate libxml;
extern crate llamapun;
extern crate time;

use std::collections::{HashSet, HashMap};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, BufWriter};

use libxml::tree::Node;
use llamapun::data::Corpus;

static TAB: &'static [u8] = b"\t";
static NEWLINE: &'static [u8] = b"\n";
static BUFFER_CAPACITY: usize = 10_485_760;


pub fn main() -> Result<(), Error> {
  let start = time::get_time();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string(),
  };
  let node_statistics_filepath = match input_args.next() {
    Some(path) => path,
    None => "statistics_mathml.txt".to_string(),
  };

  let extension_filter = input_args.next();

  let node_statistics_file = File::create(node_statistics_filepath)?;
  let mut node_statistics_writer = BufWriter::with_capacity(BUFFER_CAPACITY, node_statistics_file);

  let mut catalog = HashMap::new();
  let mut corpus = Corpus::new(corpus_path);
  corpus.extension = extension_filter;

  // open-ended attributes, for which we won't add value categories in the statistics
  // (we still counjt the attributes)
  // some of the questions were interested in the numeric values used, so we best keep those...
  let mut open_ended = HashSet::new();
  open_ended.insert("id");
  open_ended.insert("xref");
  open_ended.insert("alttext");
  open_ended.insert("href");
  // open_ended.insert("width");
  // open_ended.insert("height");
  open_ended.insert("altimg");
  // open_ended.insert("altimg-width");
  // open_ended.insert("altimg-height");
  // open_ended.insert("altimg-valign");
  // open_ended.insert("minsize");
  // open_ended.insert("maxsize");
  // open_ended.insert("voffset");

  for document in corpus.iter() {
    // Recursively descend through the math nodes and increment the frequencies of occurrence
    for math in document.get_math_nodes() {
      dfs_record(&math, &open_ended, &mut catalog);
    }

    // Increment document counter, bokkeep
    let document_count = catalog
      .entry("document_count".to_string())
      .or_insert(0);
    *document_count += 1;
    if *document_count % 1000 == 0 {
      println!("-- processed documents: {:?}", document_count);
    }
  }

  let end = time::get_time();
  let duration_sec = (end - start).num_milliseconds() / 1000;
  println!("---");
  println!("MathML statistics finished in {:?}s", duration_sec);

  let mut catalog_vec: Vec<(&String, &u64)> = catalog.iter().collect();
  catalog_vec.sort_by(|a, b| b.1.cmp(a.1));

  for (key, val) in catalog_vec {
    node_statistics_writer.write(key.as_bytes())?;
    node_statistics_writer.write(TAB)?;
    node_statistics_writer.write(val.to_string().as_bytes())?;
    node_statistics_writer.write(NEWLINE)?;
  }
  // Close the writer
  node_statistics_writer.flush()
}

fn dfs_record(node: &Node, open_ended: &HashSet<&str>, catalog: &mut HashMap<String, u64>)
{
  if node.is_text_node() {
    return; // Skip text nodes.
  }

  let node_name = node.get_name();
  // Increment frequency for node name
  let node_count = catalog.entry(node_name.clone()).or_insert(0);
  *node_count += 1;

  for (attr, val) in node.get_attributes().into_iter() {
    // Increment frequency for attr name
    let node_attr_key = format!("{}@{}",node_name, attr);
    let node_attr_count = catalog.entry(node_attr_key.clone()).or_insert(0);
    *node_attr_count += 1;

    if !open_ended.contains(attr.as_str()) {
      let attr_values = val.split_whitespace().collect::<Vec<_>>();
      // altimg-* attributes have specific styling info, we don't need to record it here.
      for val in attr_values {
        let node_attr_val_key = format!("{}[{}]", node_attr_key, val);
        let node_attr_val_count = catalog.entry(node_attr_val_key).or_insert(0);
        *node_attr_val_count += 1;
      }
    }
  }

  // Recurse into all children (DFS)
  if let Some(child) = node.get_first_child() {
    dfs_record(&child, open_ended, catalog);
    let mut child_node = child;

    while let Some(child) = child_node.get_next_sibling() {
      dfs_record(&child, open_ended, catalog);
      child_node = child;
    }
  }
}
