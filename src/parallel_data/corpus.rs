use jwalk::WalkDir as ParWalkDir;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;

use super::document::Document;
use crate::dnm::DNMParameters;
use crate::tokenizer::Tokenizer;

use libxml::parser::Parser;

/// A parallel iterable Corpus of HTML5 documents
pub struct Corpus {
  /// root directory
  pub path: String,
  /// document XHTML5 parser
  pub xml_parser: Parser,
  /// document HTML5 parser
  pub html_parser: Parser,
  /// `DNM`-aware sentence and word tokenizer
  pub tokenizer: Tokenizer,
  /// Default setting for `DNM` generation
  pub dnm_parameters: DNMParameters,
  /// Extension of corpus files (for specially tailored resources such as DLMF's .html5)
  /// defaults to selecting .html AND .xhtml files
  pub extension: Option<String>,
}

impl Default for Corpus {
  fn default() -> Corpus {
    Corpus {
      extension: None,
      path: ".".to_string(),
      tokenizer: Tokenizer::default(),
      xml_parser: Parser::default(),
      html_parser: Parser::default_html(),
      dnm_parameters: DNMParameters::llamapun_normalization(),
    }
  }
}

impl Corpus {
  /// Create a new parallel-processing corpus with the base directory `dirpath`
  pub fn new(dirpath: String) -> Self {
    Corpus {
      path: dirpath,
      ..Corpus::default()
    }
  }

  /// Get a parallel iterator over the documents, returning a single report catalog
  pub fn catalog_with_parallel_walk<F>(&self, closure: F) -> HashMap<String, u64>
  where F: Fn(Document) -> HashMap<String, u64> + Send + Sync {
    ParWalkDir::new(self.path.clone())
      .num_threads(rayon::current_num_threads())
      .skip_hidden(true)
      .sort(false)
      .into_iter()
      .filter_map(|each| {
        if let Ok(entry) = each {
          let file_name = entry.file_name.to_str().unwrap_or("");
          let selected = if let Some(ref extension) = self.extension {
            file_name.ends_with(extension)
          } else {
            file_name.ends_with(".html") || file_name.ends_with(".xhtml")
          };
          if selected {
            let path = entry.path().to_str().unwrap_or("").to_owned();
            if !path.is_empty() {
              return Some(path);
            }
          }
        }
        // all other cases
        None
      })
      .enumerate()
      .par_bridge()
      .map(|each| {
        let (index, path) = each;
        let document = Document::new(path, self).unwrap();
        if index % 1000 == 0 && index > 0 {
          println!(
            "-- catalog_with_parallel_walk now processing document {:?}",
            1 + index
          );
        }
        closure(document)
      })
      .reduce(HashMap::new, |mut map1, map2| {
        for (k, v) in map2 {
          let entry = map1.entry(k).or_insert(0);
          *entry += v;
        }
        map1
      })
  }

  /// Get a parallel iterator over the documents, returning a pair of report catalogs
    pub fn catalogs_with_parallel_walk<F>(&self, closure: F) -> (HashMap<String, u64>,HashMap<String, u64>)
  where F: Fn(Document) -> (HashMap<String, u64>,HashMap<String, u64>) + Send + Sync {
    ParWalkDir::new(self.path.clone())
      .num_threads(rayon::current_num_threads())
      .skip_hidden(true)
      .sort(false)
      .into_iter()
      .filter_map(|each| {
        if let Ok(entry) = each {
          let file_name = entry.file_name.to_str().unwrap_or("");
          let selected = if let Some(ref extension) = self.extension {
            file_name.ends_with(extension)
          } else {
            file_name.ends_with(".html") || file_name.ends_with(".xhtml")
          };
          if selected {
            let path = entry.path().to_str().unwrap_or("").to_owned();
            if !path.is_empty() {
              return Some(path);
            }
          }
        }
        // all other cases
        None
      })
      .enumerate()
      .par_bridge()
      .map(|each| {
        let (index, path) = each;
        let document = Document::new(path, self).unwrap();
        if index % 1000 == 0 && index > 0 {
          println!(
            "-- catalog_with_parallel_walk now processing document {:?}",
            1 + index
          );
        }
        closure(document)
      })
      .reduce(|| (HashMap::new(),HashMap::new()), |(mut map11, mut map12), (map21,map22)| {
        for (k, v) in map21 {
          let entry = map11.entry(k).or_insert(0);
          *entry += v;
        }
        for (k, v) in map22 {
          let entry = map12.entry(k).or_insert(0);
          *entry += v;
        }
        (map11,map12)
      })
  }
}
