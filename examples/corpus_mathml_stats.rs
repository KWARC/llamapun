// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//

//! Given a `CorTeX` corpus of HTML5 documents, extract statistics on MathML use
//! as per https://github.com/mathml-refresh/mathml/issues/55#issuecomment-474768228

extern crate libxml;
extern crate llamapun;
extern crate time;

use std::collections::HashMap;
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
    None => "mathml_statistics.txt".to_string(),
  };

  let node_statistics_file = File::create(node_statistics_filepath)?;
  let mut node_statistics_writer = BufWriter::with_capacity(BUFFER_CAPACITY, node_statistics_file);

  let mut catalog = HashMap::new();
  let mut corpus = Corpus::new(corpus_path);
  for document in corpus.iter() {
    // Recursively descend through the math nodes and increment the frequencies of occurrence
    for math in document.get_math_nodes() {
      dfs_record(&math, &mut catalog);
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

fn dfs_record(node: &Node, catalog: &mut HashMap<String, u64>)
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

    let attr_values = val.split_whitespace().collect::<Vec<_>>();
    if (attr != "id") && (attr != "xref") && (attr!="alttext") && (attr!="href") {
      for val in attr_values {
        let node_attr_val_key = format!("{}[{}]", node_attr_key, val);
        let node_attr_val_count = catalog.entry(node_attr_val_key).or_insert(0);
        *node_attr_val_count += 1;
      }
    }
  }

  // Recurse into all children (DFS)
  if let Some(child) = node.get_first_child() {
    dfs_record(&child, catalog);
    let mut child_node = child;

    while let Some(child) = child_node.get_next_sibling() {
      dfs_record(&child, catalog);
      child_node = child;
    }
  }
}
