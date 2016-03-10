use std::collections::HashMap;

/// Records single words, in order of appearance
pub struct Dictionary {
  pub map : HashMap<String, usize>,
  index : usize
}
impl Default for Dictionary {
  fn default() -> Dictionary {
    Dictionary { map : HashMap::new(), index : 0 }
  }
}
impl Dictionary {
  pub fn new() -> Self { Dictionary::default() }
  pub fn insert(&mut self, word : String) {
    let map = &mut self.map;
    // Only record if new
    if !map.contains_key(&word) {
      self.index += 1;
      map.insert(word, self.index);
    }
  }
  pub fn sort(&self) -> Vec<(String, usize)> {
    let mut as_vec = self.map.clone().into_iter().collect::<Vec<_>>();
    as_vec.sort_by(|a,b| a.1.cmp(&b.1));
    return as_vec
  }
  pub fn count(&self) -> usize {
    self.index.clone()
  }
}

/// Records the frequencies of single words
pub struct Unigrams {
  pub map : HashMap<String, usize>
}
impl Default for Unigrams {
  fn default() -> Unigrams {
    Unigrams { map : HashMap::new() }
  }
}

impl Unigrams {
  pub fn new() -> Self { Unigrams::default() }
  pub fn get(&self, word: &str) -> usize {
    match self.map.get(word) {
      Some(count) => count.clone(),
      None => 0
    }
  }
  pub fn insert(&mut self, word : String) {
    let counter = self.map.entry(word).or_insert(0);
    *counter += 1;
  }
  pub fn sort(&self) -> Vec<(String, usize)> {
    let mut as_vec = self.map.clone().into_iter().collect::<Vec<_>>();
    as_vec.sort_by(|a,b| a.1.cmp(&b.1));
    return as_vec
  }
  pub fn count(&self) -> usize {
    self.map.len()
  }
}
