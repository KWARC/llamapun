// Copyright 2015-2019 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//

//! Given a `CorTeX` corpus of HTML5 documents, extract statistics on MathML use
//! as per https://github.com/mathml-refresh/mathml/issues/55#issuecomment-474768228
//!
//! example use for arXMLiv:
//!    `cargo run --release --example corpus_mathml_stats /data/datasets/dataset-arXMLiv-08-2018
//! arxmliv_mathml_statistics.csv` example use for DLMF:
//!    `cargo run --release --example corpus_mathml_stats /var/local/dlmf dlmf_mathml_statistics.csv
//! .html5`
//!
//! This script extracts the raw data from a "blind" descent over each `<math>` element, and may
//! require additional cutoffs and post-processing over uncurated corpora.
//! You can find an example of post-processing done for the data of arXMLiv here:
//! https://gist.github.com/dginev/e50a632d31be05bb87d64cc1800f6fd4#file-apply_cutoffs-pl
#![allow(clippy::unused_io_amount)]

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Error};
use std::time::Instant;

use libxml::readonly::RoNode;
use llamapun::parallel_data::Corpus;

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
  let node_statistics_filepath = match input_args.next() {
    Some(path) => path,
    None => "corpus_statistics_mathml.csv".to_string(),
  };

  let extension_filter = input_args.next();

  let node_statistics_file = File::create(node_statistics_filepath)?;

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

  let catalog = corpus.catalog_with_parallel_walk(|document| {
    println!(
      "doc: {:?}",
      document.path
    );

    // Recursively descend through the math nodes and increment the frequencies of occurrence
    document
      .get_math_nodes()
      .into_par_iter()
      .map(|math| {
        let mut catalog = HashMap::new();
        dfs_record(math, &open_ended, &mut catalog);
        catalog
      })
      .reduce(HashMap::new, |mut map1, map2| {
        for (k, v) in map2 {
          let entry = map1.entry(k).or_insert(0);
          *entry += v;
        }
        map1
      })
  });

  let duration_sec = start.elapsed().as_millis();
  println!("---");
  println!("MathML statistics finished in {:?}ms", duration_sec);

  let mut catalog_vec: Vec<(&String, &u64)> = catalog.iter().collect();
  catalog_vec.sort_by(|a, b| b.1.cmp(a.1));

  let buffered_writer = BufWriter::with_capacity(BUFFER_CAPACITY, node_statistics_file);
  let mut csv_writer = csv::Writer::from_writer(buffered_writer);
  csv_writer.write_record(["name@attr[value]", "frequency"])?;

  for (key, val) in catalog_vec {
    csv_writer.write_record([key, &val.to_string()])?;
  }
  // Close the writer
  csv_writer.flush()
}

fn dfs_record(node: RoNode, open_ended: &HashSet<&str>, catalog: &mut HashMap<String, u64>) {
  if node.is_text_node() {
    return; // Skip text nodes.
  }

  let node_name = node.get_name();
  // Increment frequency for node name
  let node_count = catalog.entry(node_name.clone()).or_insert(0);
  *node_count += 1;

  for (attr, val) in node.get_attributes().into_iter() {
    // Increment frequency for attr name
    let node_attr_key = format!("{}@{}", node_name, attr);
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
    dfs_record(child, open_ended, catalog);
    let mut child_node = child;

    while let Some(child) = child_node.get_next_sibling() {
      dfs_record(child, open_ended, catalog);
      child_node = child;
    }
  }
}
