// Copyright 2015-2018 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//

use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use libxml::xpath::Context;
use tar::{Builder, Header};

use llamapun::ams;
use llamapun::ams::{AmsEnv, StructuralEnv};
use llamapun::dnm;
use llamapun::parallel_data::Corpus;

static MAX_WORD_LENGTH: usize = 25;

/// assume we are dealing with less than 100m items here
pub fn num_file_path(directory: &str, index: u64) -> String {
  let mut file_base = (100_000_000 + index).to_string();
  file_base.remove(0);
  directory.to_string() + "/" + &file_base + ".txt"
}

struct TarBuilder {
  builder: Builder<File>,
  count: u64,
  stamp: u64,
}

impl TarBuilder {
  /// This is a good place to discuss inodes. The expected number of paragraph files in arXiv 08.2018
  /// exceeds 50 million. Hence, one would expect a >1 TB ext4 drive, for the default inode
  /// allocation to suffice However, using a modern NVMe SSD for speed conflicts that requirement.
  /// Hence, solution -- write directly to a .tar file, and avoid the inode trouble.
  pub fn save(&mut self, data: &str, in_tar_directory: &str) -> Result<(), Error> {
    self.count += 1;
    let paragraph_filename = num_file_path(in_tar_directory, self.count);

    let bytes = data.as_bytes();
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(self.stamp);
    header.set_cksum();
    self
      .builder
      .append_data(&mut header, paragraph_filename, bytes)
  }
}

/// Given a `CorTeX` corpus of HTML5 documents, extract a token model as a
/// single file
pub fn main() -> Result<(), Error> {
  let start = SystemTime::now();
  let stamp = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
  // Read input arguments
  let mut input_args = env::args();
  let _ = input_args.next(); // skip process name
  let corpus_path = match input_args.next() {
    Some(path) => path,
    None => "tests/resources/".to_string(),
  };
  let paragraph_model_file = match input_args.next() {
    Some(path) => path,
    None => "ams_paragraphs.tar".to_string(),
  };

  let space = ' ';
  let linebreak = '\n';
  // Integers, floats, subfigure numbers
  let is_numeric = Regex::new(r"^-?(?:\d+)(?:[a-k]|(?:\.\d+(?:[eE][+-]?\d+)?))?$").unwrap();

  let file = File::create(paragraph_model_file).unwrap();
  let tar_builder = Arc::new(Mutex::new(TarBuilder {
    count: 0,
    stamp,
    builder: Builder::new(file),
  }));

  let corpus = Corpus::new(corpus_path);
  let catalog = corpus.catalog_with_parallel_walk(|document| {
    let thread_builder = tar_builder.clone();
    let mut paragraph_count: u64 = 0;
    let mut overflow_count = 0;
    let mut thread_counts = HashMap::new();
    thread_counts.insert(String::from("total_document_count"), 1);
    // Only analyze if document contains AMS markup
    if !ams::has_markup_xmldoc(&document.dom) {
      return thread_counts;
    }
    thread_counts.insert(String::from("ams_document_count"), 1);
    let mut context = Context::new(&document.dom).unwrap();

    'paragraphs: for mut paragraph in document.paragraph_iter() {
      let mut paragraph_buffer = String::new();
      let mut sentence_buffer;
      let mut invalid_paragraph = false;
      let para_parent = paragraph.dnm.root_node.get_parent().unwrap();
      let mut prev_opt = paragraph.dnm.root_node.get_prev_sibling();
      let mut prev_name = String::new();
      // only record the First paragraph of a named class,
      // i.e. previous sibling needs to be an h* element, if any
      while let Some(prev_node) = prev_opt {
        if prev_node.is_element_node() {
          prev_name = prev_node.get_name();
          prev_opt = Some(prev_node);
          break;
        } else {
          prev_opt = prev_node.get_prev_sibling();
        }
      }
      if !prev_name.is_empty() && !prev_name.starts_with('h') {
        continue 'paragraphs;
      }

      'sentences: for mut sentence in paragraph.iter() {
        sentence_buffer = String::new();
        for word in sentence.simple_iter() {
          let lexeme_str: String;
          if !word.range.is_empty() {
            let word_string = word
              .range
              .get_plaintext()
              .chars()
              .filter(|c| c.is_alphanumeric()) // drop apostrophes, other noise?
              .collect::<String>()
              .to_lowercase();
            if word_string.len() > MAX_WORD_LENGTH {
              // Using a more aggressive normalization, large words tend to be conversion
              // errors with lost whitespace - drop the entire paragraph when this occurs.
              overflow_count += 1;
              invalid_paragraph = true;
              break 'sentences;
            }
            let mut word_str: &str = &word_string;
            // Note: the formula and citation counts are an approximate lower bound, as
            // sometimes they are not cleanly tokenized, e.g. $k$-dimensional
            // will be the word string "mathformula-dimensional"
            if word_string.contains("mathformula") {
              lexeme_str = dnm::node::lexematize_math(word.range.get_node(), &mut context);
              word_str = &lexeme_str;
            } else if word_string.contains("citationelement") {
              word_str = "citationelement";
            } else if is_numeric.is_match(&word_string) {
              word_str = "NUM";
            }
            sentence_buffer.push_str(word_str);
            sentence_buffer.push(space);
          }
        }

        if !sentence_buffer.is_empty() {
          paragraph_buffer.push_str(&sentence_buffer);
          paragraph_buffer.push(linebreak);
        }
      }
      // If paragraph was valid and contains text, record it
      if !invalid_paragraph && !paragraph_buffer.is_empty() {
        // paragraph was valid, what is its label?
        let parent_class = para_parent.get_attribute("class").unwrap_or_default();
        let ams_class = ams::class_to_env(&parent_class);

        let ams_dir = if let Some(env) = ams_class {
          if env == AmsEnv::Other {
            // if Other markup (long tail ams env classes), ignore the paragraph to avoid
            // pollution by mis-counting class A paras as class B (or Other)
            continue 'paragraphs;
          } else {
            env.to_string()
          }
        } else if let Some(ref prev_node) = prev_opt {
          // if None AMS markup found, check for structural markup, or record as "other" in model
          let env: StructuralEnv = prev_node.get_content().into();
          env.to_string()
        } else {
          String::from("other")
        };
        paragraph_count += 1;
        thread_builder
          .lock()
          .unwrap()
          .save(&paragraph_buffer, &ams_dir)
          .expect("Tar builder should always succeed.")
      }
    }
    thread_counts.insert(String::from("paragraph_count"), paragraph_count);
    thread_counts.insert(String::from("overflow_count"), overflow_count);
    thread_counts
  });

  let duration_sec = SystemTime::now().duration_since(start).unwrap().as_secs();
  println!("---");
  println!(
    "AMS paragraph model finished in {:?}s, gathered: ",
    duration_sec
  );

  println!(
    "{:?} documents;",
    catalog.get("ams_document_count").unwrap_or(&0)
  );
  println!(
    "{:?} paragraphs;",
    catalog.get("paragraph_count").unwrap_or(&0)
  );
  println!(
    "{:?} discarded paragraphs (long words)",
    catalog.get("overflow_count").unwrap_or(&0)
  );
  Ok(())
}
