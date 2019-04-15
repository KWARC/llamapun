//! Data structures and Iterators for rayon-enabled parallel processing
//! including parallel I/O in walking a corpus
//! as well as DOM primitives that allow parallel iterators on XPath results, etc
use jwalk::WalkDir as ParWalkDir;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::vec::IntoIter;

use libxml::parser::{Parser, XmlParseError};
use libxml::readonly::RoNode;
use libxml::tree::Document as XmlDoc;
use libxml::xpath::Context;

use crate::dnm::{DNMParameters, DNMRange, DNM};
use crate::tokenizer::Tokenizer;

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

/// One of our math documents, thread-friendly
pub struct Document<'d> {
  /// The DOM of the document
  pub dom: XmlDoc,
  /// The file path of the document
  pub path: String,
  /// A reference to the corpus containing this document
  pub corpus: &'d Corpus,
  /// If it exists, the DNM corresponding to this document
  pub dnm: Option<DNM>,
}

/// An iterator over paragraphs of a `Document`. Ignores paragraphs containing `ltx_ERROR` markup
pub struct ParagraphIterator<'iter> {
  /// A walker over paragraph nodes
  walker: IntoIter<RoNode>,
  /// A reference to the document over which we iterate
  pub document: &'iter Document<'iter>,
}

/// A paragraph of a document with a DNM
pub struct Paragraph<'p> {
  /// The dnm of this paragraph
  pub dnm: DNM,
  /// A reference to the document containing this paragraph
  pub document: &'p Document<'p>,
}

/// An iterator over the sentences of a document/paragraph
pub struct SentenceIterator<'iter> {
  /// The walker over the sentence ranges
  walker: IntoIter<DNMRange<'iter>>,
  // pub paragraph : &'iter Paragraph<'iter>
  /// A reference to the document we are working on
  pub document: &'iter Document<'iter>,
}

/// A sentence in a document
pub struct Sentence<'s> {
  /// The range of the sentence
  pub range: DNMRange<'s>,
  // pub paragraph : &'s Paragraph<'s>
  /// The document containing this sentence
  pub document: &'s Document<'s>,
}

/// An iterator over the words of a sentence, where the words are only defined
/// by their ranges
pub struct SimpleWordIterator<'iter> {
  /// The walker over the words
  walker: IntoIter<DNMRange<'iter>>,
  /// The sentence containing the words
  pub sentence: &'iter Sentence<'iter>,
}

/// A word with a DNM range and a reference to its owner parent
pub struct Word<'w> {
  /// The range of the word
  pub range: DNMRange<'w>, // &'w str, // should we use the DNMRange instead???
  /// The sentence containing this word
  pub sentence: &'w Sentence<'w>,
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

  /// Get a parallel iterator over the documents
  pub fn catalog_with_parallel_walk<F>(&mut self, closure: F) -> HashMap<String, u64>
  where
    F: Fn(Document) -> HashMap<String, u64> + Send + Sync,
  {
    ParWalkDir::new(self.path.clone())
      .num_threads(dbg!(rayon::current_num_threads()))
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
            if path.is_empty() {
              None
            } else {
              Some(path)
            }
          } else {
            None
          }
        } else {
          None
        }
      })
      .par_bridge()
      .map(|path| {
        let document = Document::new(path, &self).unwrap();
        closure(document)
      })
      .reduce(
        || HashMap::new(),
        |mut map1, map2| {
          for (k, v) in map2 {
            let entry = map1.entry(k).or_insert(0);
            *entry += v;
          }
          map1
        },
      )
  }
}

impl<'d> Document<'d> {
  /// Load a new document
  pub fn new(filepath: String, corpus: &'d Corpus) -> Result<Self, XmlParseError> {
    let dom = if filepath.ends_with(".xhtml") {
      corpus.xml_parser.parse_file(&filepath)?
    } else {
      corpus.html_parser.parse_file(&filepath)?
    };

    Ok(Document {
      path: filepath,
      dom,
      corpus,
      dnm: None,
    })
  }

  /// Obtain the problem-free logical paragraphs of a libxml `Document`
  pub fn paragraph_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate(
      "//*[local-name()='div' and contains(@class,'ltx_para') and not(descendant::*[contains(@class,'ltx_ERROR')]) and not(preceding-sibling::*[contains(@class,'ltx_ERROR')])]",
    ) {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }

  /// Get an iterator over the paragraphs of the document
  pub fn paragraph_iter(&mut self) -> ParagraphIterator {
    let paras = Document::paragraph_nodes(&self.dom);
    ParagraphIterator {
      walker: paras.into_iter(),
      document: self,
    }
  }

  /// Obtain the MathML <math> nodes of a libxml `Document`
  pub fn get_math_nodes(&self) -> Vec<RoNode> {
    Document::math_nodes(&self.dom)
  }

  /// Associated function for `get_math_nodes`
  fn math_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate("//*[local-name()='math']") {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }
  /// Obtain the <span[class=ltx_ref]> nodes of a libxml `Document`
  pub fn get_ref_nodes(&self) -> Vec<RoNode> {
    Document::ref_nodes(&self.dom)
  }
  /// Associated function for `get_ref_nodes`
  fn ref_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate("//*[(local-name()='span' or local-name()='a') and (contains(@class,'ltx_ref ') or @class='ltx_ref')]") {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }

  /// Get an iterator over the sentences of the document
  pub fn sentence_iter(&mut self) -> SentenceIterator {
    if self.dnm.is_none() {
      if let Some(root) = self.dom.get_root_readonly() {
        self.dnm = Some(DNM::new(root, self.corpus.dnm_parameters.clone()));
      }
    }
    let tokenizer = &self.corpus.tokenizer;
    let sentences = tokenizer.sentences(self.dnm.as_ref().unwrap());
    SentenceIterator {
      walker: sentences.into_iter(),
      document: self,
    }
  }
}

impl<'iter> Iterator for ParagraphIterator<'iter> {
  type Item = Paragraph<'iter>;
  fn next(&mut self) -> Option<Paragraph<'iter>> {
    match self.walker.next() {
      None => None,
      Some(node) => {
        // Create a DNM for the current paragraph
        let dnm = DNM::new(node, DNMParameters::llamapun_normalization());
        Some(Paragraph {
          dnm,
          document: self.document,
        })
      }
    }
  }
}

impl<'p> Paragraph<'p> {
  /// Get an iterator over the sentences in this paragraph
  pub fn iter(&'p mut self) -> SentenceIterator<'p> {
    let tokenizer = &self.document.corpus.tokenizer;
    let sentences = tokenizer.sentences(&self.dnm);
    SentenceIterator {
      walker: sentences.into_iter(),
      document: self.document,
    }
  }
}

impl<'iter> Iterator for SentenceIterator<'iter> {
  type Item = Sentence<'iter>;
  fn next(&mut self) -> Option<Sentence<'iter>> {
    match self.walker.next() {
      None => None,
      Some(range) => {
        if range.is_empty() {
          self.next()
        } else {
          let sentence = Sentence {
            range,
            document: self.document,
          };
          Some(sentence)
        }
      }
    }
  }
}

impl<'s> Sentence<'s> {
  /// Get an iterator over the words (using rudimentary heuristics)
  pub fn simple_iter(&'s mut self) -> SimpleWordIterator<'s> {
    let tokenizer = &self.document.corpus.tokenizer;
    let words = tokenizer.words(&self.range);
    SimpleWordIterator {
      walker: words.into_iter(),
      sentence: self,
    }
  }
}

impl<'iter> Iterator for SimpleWordIterator<'iter> {
  type Item = Word<'iter>;
  fn next(&mut self) -> Option<Word<'iter>> {
    match self.walker.next() {
      None => None,
      Some(range) => Some(Word {
        range,
        sentence: self.sentence,
      }),
    }
  }
}
