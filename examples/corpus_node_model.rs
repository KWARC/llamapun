// Copyright 2015-2016 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
extern crate llamapun;
extern crate time;
extern crate libxml;

use std::env;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::collections::HashMap;

use libxml::tree::Node;
use llamapun::data::Corpus;

static SPACE : &'static [u8] = b" ";
static NEWLINE : &'static [u8] = b"\n";

/// Given a `CorTeX` corpus of HTML5 documents, extract a node model as a single file
pub fn main() {
  let start = time::get_time();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string()
  };
  let node_model_filepath = match input_args.next() {
    Some(path) => path,
    None => "node_model.txt".to_string()
  };
  let node_statistics_filepath = match input_args.next() {
    Some(path) => path,
    None => "node_statistics.txt".to_string()
  };

  let node_model_file = match File::create(node_model_filepath) {
    Ok(fh) => fh,
    Err(e) => {
      println!("Failed to open node model output file, aborting. Reason: {:?}", e);
      return;
    }
  };
  let mut node_model_writer = BufWriter::with_capacity(10485760, node_model_file);

  let node_statistics_file = match File::create(node_statistics_filepath) {
    Ok(fh) => fh,
    Err(e) => {
      println!("Failed to open node statistics output file, aborting. Reason: {:?}", e);
      return;
    }
  };
  let mut node_statistics_writer = BufWriter::with_capacity(10485760, node_statistics_file);


  let mut total_counts = HashMap::new();
  let mut corpus = Corpus::new(corpus_path);
  for document in corpus.iter() {
    // Recursively descend the dom DFS and record to the token model
    let root = document.dom.get_root_element().unwrap();
    dfs_record(&root, &mut total_counts, &mut node_model_writer);

    // Increment document counter, bokkeep
    let document_count = total_counts.entry("document_count".to_string()).or_insert(0);
    *document_count += 1;
    if *document_count % 1000 == 0 {
      println!("-- processed documents: {:?}", document_count);
    }
  }

  if let Err(e) = node_model_writer.flush() {
    println!("-- Failed to print to model output buffer! Proceed with caution;\n{:?}",e);
  }

  let end = time::get_time();
  let duration_sec = (end - start).num_milliseconds() / 1000;
  println!("---");
  println!("Node model finished in {:?}s", duration_sec);

  let mut total_counts_vec: Vec<(&String, &u32)> = total_counts.iter().collect();
  total_counts_vec.sort_by(|a, b| b.1.cmp(a.1));

  for (key, val) in total_counts_vec {
    if let Err(e) = node_statistics_writer.write(key.as_bytes()) {
      println!("-- Failed to print to statistics output buffer! Proceed with caution;\n{:?}",e);
    }
    if let Err(e) = node_statistics_writer.write(SPACE) {
      println!("-- Failed to print to statistics output buffer! Proceed with caution;\n{:?}",e);
    }
    if let Err(e) = node_statistics_writer.write(val.to_string().as_bytes()) {
      println!("-- Failed to print to statistics output buffer! Proceed with caution;\n{:?}",e);
    }
    if let Err(e) = node_statistics_writer.write(NEWLINE) {
      println!("-- Failed to print to statistics output buffer! Proceed with caution;\n{:?}",e);
    }
  }
  // Close the writer
  if let Err(e) = node_statistics_writer.flush() {
    println!("-- Failed to print to statistics output buffer! Proceed with caution;\n{:?}",e);
  }
}


fn dfs_record<W>(node: &Node, total_counts: &mut HashMap<String, u32>, node_model_writer: &mut BufWriter<W>)
                  where W: std::io::Write {
  if node.is_text_node() {
    return; // Skip text nodes.
  }

  let node_name = node.get_name();
  let mut model_token = node_name.clone();
  let class_attr = node.get_property("class").unwrap_or_default();
  let mut classes_split = class_attr.split(' ').collect::<Vec<_>>();
  classes_split.sort();
  for class_model_token in classes_split {
    if class_model_token.is_empty() {
      continue;
    }
    model_token.push_str("_");
    model_token.push_str(class_model_token);
  }
  // Increment counter for this type of node
  {
    let node_count = total_counts.entry(model_token.clone()).or_insert(0);
    *node_count += 1;
  }
  // Write the model_token of the current node into the buffer
  if let Err(e) = node_model_writer.write(model_token.as_bytes()) {
    println!("-- Failed to print to model output buffer! Proceed with caution;\n{:?}",e);
  }
  if let Err(e) = node_model_writer.write(SPACE) {
    println!("-- Failed to print to model output buffer! Proceed with caution;\n{:?}",e);
  }

  // Recurse into all children (DFS), except for math and tables
  if (node_name != "math") && (node_name != "table") {
    if let Some(child) = node.get_first_child() {
      dfs_record(&child, total_counts, node_model_writer);
      let mut child_node = child;

      while let Some(child) = child_node.get_next_sibling() {
        dfs_record(&child, total_counts, node_model_writer);
        child_node = child;
      }
    }
  }
}