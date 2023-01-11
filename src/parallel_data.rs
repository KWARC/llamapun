//! Data structures and Iterators for rayon-enabled parallel processing
//! including parallel I/O in walking a corpus
//! as well as DOM primitives that allow parallel iterators on XPath results, etc
use crate::dnm::{DNMParameters, DNMRange, DNM};
use libxml::readonly::RoNode;
use std::vec::IntoIter;

/* ---- Containers ----- */
/// container and API for a Corpus capable of parallel walks over its documents
pub mod corpus;
/// container and API for a Document yielded during a parallel corpus walk
pub mod document;
pub use self::corpus::Corpus;
pub use self::document::Document;

/// A DNM with associated document parent (e.g. for paragraphs, headings)
pub struct ItemDNM<'p> {
  /// The payload of the item
  pub dnm: DNM,
  /// A reference to the parent document
  pub document: &'p Document<'p>,
}

/// A DNMRange with associated document
pub struct ItemDNMRange<'s> {
  /// The range of the sentence
  pub range: DNMRange<'s>,
  /// The document containing this sentence
  pub document: &'s Document<'s>,
}

/* ---- Iterators ----- */

/// Generic iterater over read-only xml nodes. It is the responsibility of the abstraction returning
/// `NodeIterator` to specify the grouping principle for collecting the nodes
pub struct RoNodeIterator<'iter> {
  /// A walker over read-only nodes
  walker: IntoIter<RoNode>,
  /// A reference to the owner document
  pub document: &'iter Document<'iter>,
}

/// A generic iterator over DNMRanges with their associated document (e.g. for sentences)
pub struct DNMRangeIterator<'iter> {
  /// The walker over the sentence ranges
  walker: IntoIter<DNMRange<'iter>>,
  /// A reference to the document we are working on
  pub document: &'iter Document<'iter>,
}

impl<'iter> Iterator for RoNodeIterator<'iter> {
  type Item = ItemDNM<'iter>;
  fn next(&mut self) -> Option<ItemDNM<'iter>> {
    match self.walker.next() {
      None => None,
      Some(node) => {
        // Create a DNM for the current ItemDNM
        let dnm = DNM::new(node, DNMParameters::llamapun_normalization());
        Some(ItemDNM {
          dnm,
          document: self.document,
        })
      },
    }
  }
}

/// An iterator adaptor for filtered selections over a document
pub trait XPathFilteredIterator<'p> {
  /// the sentences for the resulting selection
  fn to_sentences(&'p self) -> Vec<DNMRange<'p>>;
  /// the owner document being selected over
  fn get_document(&'p self) -> &'p Document;

  /// Get an iterator over the sentences in this paragraph
  fn iter(&'p mut self) -> DNMRangeIterator<'p> {
    DNMRangeIterator {
      walker: self.to_sentences().into_iter(),
      document: self.get_document(),
    }
  }
}

impl<'p> XPathFilteredIterator<'p> for ItemDNM<'p> {
  fn get_document(&'p self) -> &Document { self.document }
  fn to_sentences(&'p self) -> Vec<DNMRange<'p>> {
    self.document.corpus.tokenizer.sentences(&self.dnm)
  }
}

impl<'iter> Iterator for DNMRangeIterator<'iter> {
  type Item = ItemDNMRange<'iter>;
  fn next(&mut self) -> Option<ItemDNMRange<'iter>> {
    if let Some(range) = self.walker.next() {
      if range.is_empty() {
        self.next()
      } else {
        Some(ItemDNMRange {
          range,
          document: self.document,
        })
      }
    } else {
      None
    }
  }
}

impl<'s> ItemDNMRange<'s> {
  /// Get an iterator over the words (using rudimentary heuristics)
  pub fn word_iter(&'s mut self) -> DNMRangeIterator<'s> {
    let tokenizer = &self.document.corpus.tokenizer;
    let words = tokenizer.words(&self.range);
    DNMRangeIterator {
      walker: words.into_iter(),
      document: self.document,
    }
  }
  /// Get an iterator over the words and punctuation (using rudimentary heuristics)
  pub fn word_and_punct_iter(&'s mut self) -> DNMRangeIterator<'s> {
    let tokenizer = &self.document.corpus.tokenizer;
    let words = tokenizer.words_and_punct(&self.range);
    DNMRangeIterator {
      walker: words.into_iter(),
      document: self.document,
    }
  }
}

impl<'s> ItemDNM<'s> {
  /// Get an iterator over the words (using rudimentary heuristics)
  pub fn word_iter(&'s mut self) -> DNMRangeIterator<'s> {
    let tokenizer = &self.document.corpus.tokenizer;
    let words = match self.dnm.get_range() {
      Ok(range) => tokenizer.words(&range),
      _ => Vec::new(),
    };
    DNMRangeIterator {
      walker: words.into_iter(),
      document: self.document,
    }
  }
  /// Get an iterator over the words and punctuation (using rudimentary heuristics)
  pub fn word_and_punct_iter(&'s mut self) -> DNMRangeIterator<'s> {
    let tokenizer = &self.document.corpus.tokenizer;
    let words = match self.dnm.get_range() {
      Ok(range) => tokenizer.words_and_punct(&range),
      _ => Vec::new(),
    };
    DNMRangeIterator {
      walker: words.into_iter(),
      document: self.document,
    }
  }
}
