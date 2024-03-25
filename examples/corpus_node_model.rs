// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//

//! Given a `CorTeX` corpus of HTML5 documents, extract a node model as a single file
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufWriter, Error};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use libxml::readonly::RoNode;
use llamapun::parallel_data::Corpus;

static NEWLINE: &[u8] = b"\n";
static BUFFER_CAPACITY: usize = 10_485_760;

pub fn main() -> Result<(), Error> {
  let start = Instant::now();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string(),
  };
  let node_model_filepath = match input_args.next() {
    Some(path) => path,
    None => "node_model.txt".to_string(),
  };
  let node_statistics_filepath = match input_args.next() {
    Some(path) => path,
    None => "node_statistics.txt".to_string(),
  };

  let node_statistics_file = File::create(node_statistics_filepath)?;
  let node_model_file = File::create(node_model_filepath)?;
  let node_model_writer = Arc::new(Mutex::new(BufWriter::with_capacity(
    BUFFER_CAPACITY,
    node_model_file,
  )));

  let corpus = Corpus::new(corpus_path);
  let total_counts = corpus.catalog_with_parallel_walk(|document| {
    // Recursively descend the dom DFS and record to the token model
    let mut total_counts = HashMap::new();
    if let Some(root) = document.dom.get_root_readonly() {
      let thread_writer = node_model_writer.clone();
      let node_model =
        dfs_record(root, &mut total_counts).expect("dfs_record should not encounter any issues.");
      let mut writer_lock = thread_writer.lock().unwrap();
      writer_lock
        .write_all(&node_model)
        .expect("buffer writes should not encounter any issues");
    }
    total_counts
  });

  node_model_writer.lock().unwrap().flush()?;

  let duration_sec = start.elapsed().as_secs();
  println!("---");
  println!("Node model finished in {:?}s", duration_sec);

  let mut total_counts_vec: Vec<(&String, &u64)> = total_counts.iter().collect();
  total_counts_vec.sort_by(|a, b| b.1.cmp(a.1));

  let mut node_statistics_writer = BufWriter::with_capacity(BUFFER_CAPACITY, node_statistics_file);
  for (key, val) in total_counts_vec {
    node_statistics_writer.write_all(key.as_bytes())?;
    node_statistics_writer.write_all(b" ")?;
    node_statistics_writer.write_all(val.to_string().as_bytes())?;
    node_statistics_writer.write_all(NEWLINE)?;
  }
  // Close the writer
  node_statistics_writer.flush()
}

fn dfs_record(node: RoNode, total_counts: &mut HashMap<String, u64>) -> Result<Vec<u8>, Error> {
  let mut subtree_model = Vec::new();
  if node.is_text_node() {
    return Ok(subtree_model); // Skip text nodes.
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
    model_token.push('_');
    model_token.push_str(class_model_token);
  }
  // Increment counter for this type of node
  {
    let node_count = total_counts.entry(model_token.clone()).or_insert(0);
    *node_count += 1;
  }
  // Write the model_token of the current node into the buffer
  subtree_model.extend(model_token.as_bytes());
  subtree_model.push(b' ');

  // Recurse into all children (DFS), except for math and tables
  if (node_name != "math") && (node_name != "table") {
    if let Some(child) = node.get_first_child() {
      subtree_model.extend(dfs_record(child, total_counts)?);

      let mut child_node = child;
      while let Some(sibling) = child_node.get_next_sibling() {
        subtree_model.extend(dfs_record(sibling, total_counts)?);
        child_node = sibling;
      }
    }
  }
  Ok(subtree_model)
}
