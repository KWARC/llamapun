//! A small ngram library
//! ngrams are sequences of n consecutive words
use circular_queue::CircularQueue;
use std::collections::HashMap;

/// Records single words, in order of appearance
#[derive(Debug, Default)]
pub struct Dictionary {
  /// hashmap for the records
  pub map: HashMap<String, usize>,
  /// index of the next word
  index: usize,
}
impl Dictionary {
  /// create a new dictionary
  pub fn new() -> Self { Dictionary::default() }
  /// insert a new word into the dictionary (if it hasn't been inserted yet)
  pub fn insert(&mut self, word: String) {
    let map = &mut self.map;
    // Only record if new
    let word_index = map.entry(word).or_insert(self.index + 1);
    if *word_index > self.index {
      self.index += 1;
    }
  }
  /// get the entries of the dictionary sorted by occurence
  pub fn sorted(&self) -> Vec<(&String, usize)> {
    let mut as_vec = self.map.iter().map(|(x, y)| (x, *y)).collect::<Vec<_>>();
    as_vec.sort_by(|a, b| b.1.cmp(&a.1));
    as_vec
  }
  /// get the number of entries in the dictionary
  pub fn count(&self) -> usize { self.index }
}

/// Ngrams are dictionaries with
pub struct Ngrams {
  /// anchor word that must be present in all ngram contexts (in their window)
  pub anchor: Option<String>,
  /// if an anchor word is given, word window size, applied to the left and to the right of the
  /// anchor word
  pub window_size: usize,
  /// n-grams for a sequence of n words
  pub n: usize,
  /// statistics hashmap for the occurence counts
  pub counts: HashMap<String, usize>,
}
impl Default for Ngrams {
  fn default() -> Ngrams {
    Ngrams {
      anchor: None,
      window_size: 0,
      n: 1,
      counts: HashMap::new(),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum AnchorSide {
  Left,
  Right,
}

impl Ngrams {
  /// Get the word count
  pub fn get(&self, word: &str) -> usize {
    match self.counts.get(word) {
      Some(count) => *count,
      None => 0,
    }
  }
  /// count a newly seen ngram phrase
  pub fn insert(&mut self, phrase: String) {
    let counter = self.counts.entry(phrase).or_insert(0);
    *counter += 1;
  }
  /// obtain the ngram report, sorted by descending frequency
  pub fn sorted(&self) -> Vec<(&String, usize)> {
    let mut as_vec = self.counts.iter().map(|(x, y)| (x, *y)).collect::<Vec<_>>();
    as_vec.sort_by(|a, b| b.1.cmp(&a.1));
    as_vec
  }
  /// get the number of distinct ngrams recorded
  pub fn distinct_count(&self) -> usize { self.counts.len() }

  /// add content for ngram analysis, typically a paragraph or a line of text
  pub fn add_content(&mut self, content: &str) {
    if self.anchor.is_some() && self.window_size > 0 {
      self.add_anchored_content(content)
    } else {
      unimplemented!(); // TODO: the basic ngram case
    }
  }

  /// In essence, for a given window size W, a word at index i is justified to participate in the
  /// ngrams if there is an instance of an anchor word in the range of words [i-W, i+W].
  /// this can be highly irregular e.g. "word word anchor word anchor word word", so we record
  /// flexibly looking for no-justification cutoffs, where a continuous word sequence is recorded
  /// for ngram counts
  pub fn add_anchored_content(&mut self, content: &str) {
    // content to add through the ngram builder
    let mut continuous_buffer = Vec::new();
    let mut context_window = CircularQueue::with_capacity(self.window_size);
    let mut words_since_anchor_seen = 0;
    let mut side = AnchorSide::Left;

    for w in content
      .split_ascii_whitespace()
      .filter(|&w| w.chars().next().unwrap().is_alphanumeric())
    {
      // add the current word, potentially erasing the oldest word that falls outside the window
      context_window.push(w);
      let anchor = self.anchor.as_ref().unwrap();
      if w == anchor {
        // we've hit an anchor word, the current content of the window should be analyzed
        words_since_anchor_seen = 0;
        side = AnchorSide::Right;
        // analyze whatever is in the buffer, and empty it
        continuous_buffer = context_window.asc_iter().copied().collect();
        context_window.clear();
      } else {
        words_since_anchor_seen += 1;
        if words_since_anchor_seen == self.window_size && side == AnchorSide::Right {
          // it has been too long since we saw an anchor, add to the current buffer, record and
          // reset
          self.record_words(std::mem::take(&mut continuous_buffer));
          context_window.clear();
          side = AnchorSide::Left;
        }
      }
    }
    // Any remaining content should be added
    continuous_buffer.extend(context_window.asc_iter().copied());
    self.record_words(std::mem::take(&mut continuous_buffer));
  }

  /// Take an arbitrarily long vector of words, and record all (overlapping) ngrams obtainable from
  /// it
  pub fn record_words(&mut self, words: Vec<&str>) {
    if words.len() < self.n {
      // nothing to do unless we have at least n words
      return;
    }
    let mut gram_window = CircularQueue::with_capacity(self.n);
    for w in words.into_iter() {
      gram_window.push(w);
      if gram_window.len() == self.n {
        let key = gram_window
          .asc_iter()
          .copied()
          .collect::<Vec<_>>()
          .join(" ");
        self.insert(key);
      }
    }
  }
}
