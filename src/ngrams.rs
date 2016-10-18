//! A small ngram library
//! ngrams are sequences of n consecutive words

use std::collections::HashMap;

/// Records single words, in order of appearance
pub struct Dictionary {
  /// hashmap for the records
  pub map : HashMap<String, usize>,
  /// index of the next word
  index : usize
}
impl Default for Dictionary {
  fn default() -> Dictionary {
    Dictionary { map : HashMap::new(), index : 0 }
  }
}
impl Dictionary {
  /// create a new dictionary
  pub fn new() -> Self { Dictionary::default() }
  /// insert a new word into the dictionary (if it hasn't been inserted yet)
  pub fn insert(&mut self, word : String) {
    let map = &mut self.map;
    // Only record if new
    if !map.contains_key(&word) {
      self.index += 1;
      map.insert(word, self.index);
    }
  }
  /// get the entries of the dictionary sorted by occurence
  pub fn sort(&self) -> Vec<(String, usize)> {
    let mut as_vec = self.map.clone().into_iter().collect::<Vec<_>>();
    as_vec.sort_by(|a,b| a.1.cmp(&b.1));
    as_vec
  }
  /// get the number of entries in the dictionary
  pub fn count(&self) -> usize {
    self.index
  }
}

/// Records the frequencies of single words
pub struct Unigrams {
  /// hashmap for the unigram counts
  pub map : HashMap<String, usize>
}
impl Default for Unigrams {
  fn default() -> Unigrams {
    Unigrams { map : HashMap::new() }
  }
}

impl Unigrams {
  /// Creates a new, empty Unigrams struct
  pub fn new() -> Self { Unigrams::default() }
  /// Get the word count
  pub fn get(&self, word: &str) -> usize {
    match self.map.get(word) {
      Some(count) => *count,
      None => 0
    }
  }
  /// insert a word
  pub fn insert(&mut self, word : String) {
    let counter = self.map.entry(word).or_insert(0);
    *counter += 1;
  }
  /// get the inserted words, sorted by frequency
  pub fn sort(&self) -> Vec<(String, usize)> {
    let mut as_vec = self.map.clone().into_iter().collect::<Vec<_>>();
    as_vec.sort_by(|a,b| a.1.cmp(&b.1));
    as_vec
  }
  /// get the number of different words inserted
  pub fn count(&self) -> usize {
    self.map.len()
  }
}
