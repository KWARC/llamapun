// Copyright 2015-2019 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
/// Extracts a corpus paragraph model from an unpacked corpus of HTML files
/// With math lexemes (default):
/// $ cargo run --release --example corpus_statement_paragraphs_model /path/to/corpus
/// paragraph_data.tar
///
/// With math discarded:
/// $ cargo run --release --example corpus_statement_paragraphs_model /path/to/corpus
/// statement_paragraphs.tar discard_math
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use libxml::xpath::Context;
use llamapun::ams;
use llamapun::ams::{AmsEnv, StructuralEnv};
use llamapun::dnm::SpecialTagsOption;
use llamapun::parallel_data::*;
use llamapun::util::data_helpers;
use llamapun::util::data_helpers::LexicalOptions;
use tar::{Builder, Header};

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
    None => "statement_paragraphs.tar".to_string(),
  };
  let discard_math = match input_args.next() {
    Some(value) => match value.as_str() {
      "discard_math" => true, // should eventually become --discard_math flag, rushing for now.
      _ => false,
    },
    None => false,
  };

  let file = File::create(paragraph_model_file).unwrap();
  let tar_builder = Arc::new(Mutex::new(TarBuilder {
    count: 0,
    stamp,
    builder: Builder::new(file),
    names: HashSet::new(),
  }));

  let mut corpus = Corpus::new(corpus_path);
  if discard_math {
    println!("-- will discard math.");
    corpus
      .dnm_parameters
      .special_tag_name_options
      .insert("math".to_string(), SpecialTagsOption::Skip);
    corpus
      .dnm_parameters
      .special_tag_class_options
      .insert("ltx_equation".to_string(), SpecialTagsOption::Skip);
    corpus
      .dnm_parameters
      .special_tag_class_options
      .insert("ltx_equationgroup".to_string(), SpecialTagsOption::Skip);
  } else {
    println!("-- will lexematize math.")
  }

  let catalog = corpus.catalog_with_parallel_walk(|doc| {
    extract_document_statements(doc, tar_builder.clone(), discard_math)
  });

  println!(
    "{:?} Total traversed documents;",
    catalog.get("total_document_count").unwrap_or(&0)
  );
  println!(
    "{:?} AMS marked up documents;",
    catalog.get("ams_document_count").unwrap_or(&0)
  );
  println!(
    "{:?} paragraphs;",
    catalog.get("paragraph_count").unwrap_or(&0)
  );
  println!(
    "{:?} discarded paragraphs (irregular word count or word length)",
    catalog.get("overflow_count").unwrap_or(&0)
  );
  let mut builder_lock = tar_builder.lock().unwrap();
  println!(
    "{:?} paragraphs written to .tar destination (discarded duplicate SHA256-based filenames)",
    builder_lock.count
  );
  builder_lock
    .builder
    .finish()
    .expect("Tar builder should always succeed.");

  let duration_sec = SystemTime::now().duration_since(start).unwrap().as_secs();
  println!("---");
  println!("AMS paragraph model finished in {:?}s.", duration_sec);

  Ok(())
}

/// Extraction and preprocessing logic
fn extract_document_statements(
  document: Document,
  tar_builder: Arc<Mutex<TarBuilder>>,
  discard_math: bool,
) -> HashMap<String, u64> {
  // Document-level context variables
  let mut paragraph_count: u64 = 0;
  let mut overflow_count = 0;
  let mut thread_data = Vec::new();
  let mut thread_counts = HashMap::new();
  thread_counts.insert(String::from("total_document_count"), 1);
  // Check if document contains AMS markup
  let has_ams_markup = ams::has_markup_xmldoc(&document.dom);
  if has_ams_markup {
    thread_counts.insert(String::from("ams_document_count"), 1);
  }
  let mut context = Context::new(&document.dom).unwrap();

  'paragraphs: for mut paragraph in document.extended_paragraph_iter() {
    // I. Determine the class for this paragraph entry, so that we can iterate over its content
    // after if no markup at all, ignore the paragraph and skip to next
    let para = paragraph.dnm.root_node;
    let para_parent = para.get_parent().unwrap();
    let mut prev_heading_opt = paragraph.dnm.root_node.get_prev_sibling();
    let mut prev_name = String::new();
    // in regular div.ltx_para cases, only record the First paragraph of a named class,
    // i.e. previous sibling needs to be an h* element, if any
    'find_prev_sibling: while let Some(prev_node) = prev_heading_opt {
      if prev_node.is_element_node() {
        prev_name = prev_node.get_name();
        break 'find_prev_sibling;
      } else {
        prev_heading_opt = prev_node.get_prev_sibling();
      }
    }
    let para_class = para.get_attribute("class").unwrap_or_default();
    // Check if we are looking at the two current special markup casesthread::spawn(move || {
    // div.ltx_acknowledgement
    let special_marker = if para_class.contains("ltx_acknowledgement") {
      Some(StructuralEnv::Acknowledgement)
    } else if para_class.contains("ltx_caption") {
      Some(StructuralEnv::Caption)
    } else {
      // we can short-circuit here, if no special marker and no prior heading, just skip paragraph
      if !prev_name.is_empty() && !prev_name.starts_with('h') {
        continue 'paragraphs;
      }
      None
    };
    // Before we go into tokenization, ensure this is an English paragraph
    if data_helpers::invalid_for_english_latin(&paragraph.dnm) {
      continue 'paragraphs;
    }
    let class_directory = if let Some(env) = special_marker {
      // specific element markup is an override to heading siblings
      env.to_string()
    } else {
      // set ams class, if any
      let ams_class = if has_ams_markup {
        let parent_class = para_parent.get_attribute("class").unwrap_or_default();
        ams::class_to_env(&parent_class)
      } else {
        None
      };
      if let Some(env) = ams_class {
        match env {
          // Other and other-like entities that are too noisy to include
          // New for 2019: ignore the low-volume cases as well
          AmsEnv::Affirmation
          | AmsEnv::Algorithm
          | AmsEnv::Answer
          | AmsEnv::Bound
          | AmsEnv::Caption
          | AmsEnv::Comment
          | AmsEnv::Constraint
          | AmsEnv::Convention
          | AmsEnv::Criterion
          | AmsEnv::Expansion
          | AmsEnv::Expectation
          | AmsEnv::Explanation
          | AmsEnv::Hint
          | AmsEnv::Issue
          | AmsEnv::Keywords
          | AmsEnv::Note
          | AmsEnv::Notice
          | AmsEnv::Paragraph
          | AmsEnv::Principle
          | AmsEnv::Rule
          | AmsEnv::Solution
          | AmsEnv::Other => continue 'paragraphs,
          _ => env.to_string(),
        }
      } else if let Some(heading_node) = prev_heading_opt {
        // if no AMS markup found, check for structural markup
        if let Some(heading_text) = data_helpers::heading_from_node_aux(
          heading_node,
          &document.corpus.tokenizer,
          &mut context,
        ) {
          let env: StructuralEnv = heading_text.as_str().into();
          if env == StructuralEnv::Other {
            // if Other markup, ignore
            continue 'paragraphs;
          }
          env.to_string()
        } else {
          continue 'paragraphs;
        }
      } else {
        continue 'paragraphs;
      }
    };
    // II. We have a labeled statement. Extract content of current paragraph, validating basic data
    // quality
    let mut word_count = 0;
    let mut invalid_paragraph = false;
    let mut paragraph_buffer = String::new();
    'words: for word in paragraph.word_and_punct_iter() {
      if word.range.is_empty() {
        continue 'words;
      }
      let word_string = match data_helpers::ams_normalize_word_range(
        &word.range,
        &mut context,
        LexicalOptions {
          discard_math,
          discard_punct: false,
          discard_case: true,
        },
      ) {
        Ok(w) => w,
        Err(_) => {
          overflow_count += 1;
          invalid_paragraph = true;
          break 'words;
        },
      };
      if !word_string.is_empty() {
        word_count += 1;
        paragraph_buffer.push_str(&word_string);
        paragraph_buffer.push(' ');
      }
    }
    // Discard paragraphs outside of a reasonable [4,1024] word count range
    if !(4..=1024).contains(&word_count) {
      overflow_count += 1;
      invalid_paragraph = true;
    }

    // If paragraph was valid and contains text, record it
    if !invalid_paragraph {
      paragraph_buffer.push('\n');

      paragraph_count += 1;
      // precompute sha inside the thread, to do more in parallel
      let paragraph_filename = hash_file_path(&class_directory, &paragraph_buffer);
      thread_data.push((paragraph_buffer, paragraph_filename));
    }
  }
  // III. Record valid entries into archive target, having collected all labeled samples for this
  // document
  let mut builder_lock = tar_builder.lock().unwrap();
  for (paragraph_buffer, paragraph_filename) in thread_data.into_iter() {
    builder_lock
      .save(&paragraph_buffer, &paragraph_filename)
      .expect("Tar builder should always succeed.")
  }
  // IV. Bookkeep counts for final report and finish this document
  thread_counts.insert(String::from("paragraph_count"), paragraph_count);
  thread_counts.insert(String::from("overflow_count"), overflow_count);
  thread_counts
}

//
// -- Auxiliary helpers that don't yet have a home in llamapun
//

/// give a sha256 hash, assemble a filename based on it
fn hash_file_path(directory: &str, content: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.input_str(content);
  let hash = hasher.result_str();
  directory.to_string() + "/" + &hash + ".txt"
}

struct TarBuilder {
  builder: Builder<File>,
  count: u64,
  stamp: u64,
  names: HashSet<String>,
}

impl TarBuilder {
  /// This is a good place to discuss inodes. The expected number of paragraph files in arXiv
  /// 08.2018 exceeds 50 million. Hence, one would expect a >1 TB ext4 drive, for the default
  /// inode allocation to suffice However, using a modern NVMe SSD for speed conflicts that
  /// requirement. Hence, solution -- write directly to a .tar file, and avoid the inode trouble.
  pub fn save(&mut self, data: &str, paragraph_filename: &str) -> Result<(), Error> {
    // if we see the same hash/name twice, ignore all following cases
    if self.names.contains(paragraph_filename) {
      return Ok(());
    } else {
      self.names.insert(paragraph_filename.to_string());
    }
    self.count += 1;
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
