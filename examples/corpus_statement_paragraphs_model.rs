// Copyright 2015-2019 KWARC research group. See the LICENSE
// file at the top-level directory of this distribution.
//
/// Extracts a corpus paragraph model from an unpacked corpus of HTML files
/// With math lexemes (default):
/// $ cargo run --release --example corpus_statement_paragraphs_model /path/to/corpus paragraph_data.tar
///
/// With math discarded:
/// $ cargo run --release --example corpus_statement_paragraphs_model /path/to/corpus paragraph_data_nomath.tar discard_math
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
use llamapun::parallel_data::Corpus;
use llamapun::dnm::SpecialTagsOption;
use llamapun::util::data_helpers;

use tar::{Builder, Header};

/// assume we are dealing with less than 100m items here
pub fn num_file_path(directory: &str, index: u64) -> String {
  let mut file_base = (100_000_000 + index).to_string();
  file_base.remove(0);
  directory.to_string() + "/" + &file_base + ".txt"
}
/// give a sha256 hash, assemble a filename based on it
pub fn hash_file_path(directory: &str, content: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.input_str(&content);
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
  /// This is a good place to discuss inodes. The expected number of paragraph files in arXiv 08.2018
  /// exceeds 50 million. Hence, one would expect a >1 TB ext4 drive, for the default inode
  /// allocation to suffice However, using a modern NVMe SSD for speed conflicts that requirement.
  /// Hence, solution -- write directly to a .tar file, and avoid the inode trouble.
  pub fn save(&mut self, data: &str, paragraph_filename: &str) -> Result<(), Error> {
    // if we see the same hash/name twice, ignore all following cases
    if self.names.contains(paragraph_filename) {
      return Ok(());
    } else {
      self.names.insert(paragraph_filename.to_string());
    }
    self.count += 1;
    // let paragraph_filename = num_file_path(in_tar_directory, self.count);
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
  let discard_math = match input_args.next() {
    Some(value) => match value.as_str() {
      "discard_math" => true, // should eventually become --discard_math flag, rushing for now.
      _ => false,
    },
    None => false,
  };

  let space = ' ';
  let linebreak = '\n';

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

  let catalog = corpus.catalog_with_parallel_walk(|document| {
    let mut paragraph_count: u64 = 0;
    let mut overflow_count = 0;
    let mut thread_data = Vec::new();
    let mut thread_counts = HashMap::new();
    thread_counts.insert(String::from("total_document_count"), 1);
    // Count if document contains AMS markup
    let has_ams_markup = ams::has_markup_xmldoc(&document.dom);
    if has_ams_markup {
      thread_counts.insert(String::from("ams_document_count"), 1);
    } else {
      thread_counts.insert(String::from("ams_document_count"), 0);
    }

    let mut context = Context::new(&document.dom).unwrap();

    'paragraphs: for mut paragraph in document.extended_paragraph_iter() {
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
      // Before we go into tokenization, ensure this is an English sentence on the math-normalized plain text.
      if data_helpers::invalid_for_english_latin(&paragraph.dnm) {
        continue 'paragraphs;
      }
      'sentences: for mut sentence in paragraph.iter() {
        sentence_buffer = String::new();
        for word in sentence.simple_iter() {
          if !word.range.is_empty() {
            let word_string =
              match data_helpers::ams_normalize_word_range(&word.range, &mut context, discard_math) {
                Ok(w) => w,
                Err(_) => {
                  overflow_count += 1;
                  invalid_paragraph = true;
                  break 'sentences;
                }
              };
            if !word_string.is_empty() {
              sentence_buffer.push_str(&word_string);
              sentence_buffer.push(space);
            }
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
        let ams_class = if has_ams_markup {
          ams::class_to_env(&parent_class)
        } else {
          None
        };

        let class_directory = if let Some(env) = ams_class {
          if env == AmsEnv::Other // Other and other-like entities that are too noisy to include
            || env == AmsEnv::Caption
            || env == AmsEnv::Algorithm
            || env == AmsEnv::Paragraph
          {
            // if Other markup (long tail ams env classes), ignore the paragraph to avoid
            // pollution by mis-counting class A paras as class B (or Other)
            continue 'paragraphs;
          } else {
            env.to_string()
          }
        } else if let Some(ref prev_node) = prev_opt {
          // if None AMS markup found, check for structural markup
          let env: StructuralEnv = prev_node.get_content().into();
          if env == StructuralEnv::Other {
            // if Other markup, ignore
            continue 'paragraphs;
          }
          env.to_string()
        } else {
          // if no markup at all, ignore the paragraph, as we don't have reliable classification information
          continue 'paragraphs;
        };
        paragraph_count += 1;
        // precompute sha inside the thread, to do more in parallel
        let paragraph_filename = hash_file_path(&class_directory, &paragraph_buffer);
        thread_data.push((paragraph_buffer, paragraph_filename));
      }
    }
    let mut builder_lock = tar_builder.lock().unwrap();
    for (paragraph_buffer, paragraph_filename) in thread_data.into_iter() {
      builder_lock
        .save(&paragraph_buffer, &paragraph_filename)
        .expect("Tar builder should always succeed.")
    }

    thread_counts.insert(String::from("paragraph_count"), paragraph_count);
    thread_counts.insert(String::from("overflow_count"), overflow_count);
    thread_counts
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
    "{:?} discarded paragraphs (long words)",
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

// Locking notes:
// I.Running a mutex lock on every paragraph write yields:
// ---
// AMS paragraph model finished in 272s, gathered:
// 12330 documents;
// 249880 paragraphs;
// 250 discarded paragraphs (long words)
//
// real	4m32.821s
// user	90m37.348s
// sys	1m14.862s
//
// II.Running a mutex lock once per document thread
//    (for in-order numbering of a document's paragraphs)
// ---
// AMS paragraph model finished in 267s, gathered:
// 12330 documents;
// 249880 paragraphs;
// 250 discarded paragraphs (long words)
//
// real	4m29.651s
// user	93m17.793s
// sys	1m12.189s
// ---
// I wasn't sure what was causing more overhead:
//   - lock rotation (more locks when locking on each paragraph),
//   - resource starvation (waiting for a document lock to become available, if writing all paragraphs was slow)
// Turns out the difference is minor enough to select for the better application effect:
// Locking once per document allows a guarantee that adjacent paragraphs will have adjacent IDs when saved to the tar,
// which allows to do some helpful data munging later on in paragraph exploration mode.
