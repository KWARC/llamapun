//! The `dnm::range` submodule provides data structure for indexing into a DNM object's plaintext

use dnm::DNM;
/// Very often we'll talk about substrings of the plaintext - words, sentences,
/// etc. A `DNMRange` stores start and end point of such a substring and has
/// a reference to the `DNM`.
pub struct DNMRange<'dnmrange> {
  /// Offset of the beginning of the range
  pub start: usize,
  /// Offset of the end of the range
  pub end: usize,
  /// DNM containing this range
  pub dnm: &'dnmrange DNM,
}

impl<'dnmrange> DNMRange<'dnmrange> {
  /// Get the plaintext substring corresponding to the range
  pub fn get_plaintext(&self) -> &str {
    &(self.dnm.plaintext)[self.start..self.end]
  }
  /// Get the plaintext without trailing white spaces
  pub fn get_plaintext_truncated(&self) -> &'dnmrange str {
    (self.dnm.plaintext)[self.start..self.end].trim_right()
  }

  /// Returns a `DNMRange` with the leading and trailing whitespaces removed
  pub fn trim(&self) -> DNMRange<'dnmrange> {
    let mut trimmed_start = self.start;
    let mut trimmed_end = self.end;
    let range_text: &str = self.get_plaintext();

    for c in range_text.chars() {
      if c.is_whitespace() {
        trimmed_start += 1;
      } else {
        break;
      }
    }
    for c in range_text.chars().rev() {
      if c.is_whitespace() {
        trimmed_end -= 1;
      } else {
        break;
      }
    }
    // Edge case: when the given input is whitespace only, start will be larger than end.
    // In that case return the 0-width range at the original end marker.
    if trimmed_start >= trimmed_end {
      trimmed_start = self.end;
      trimmed_end = self.end;
    }
    DNMRange {
      start: trimmed_start,
      end: trimmed_end,
      dnm: self.dnm,
    }
  }

  /// returns a subrange, with offsets relative to the beginning of `self`
  pub fn get_subrange(&self, rel_start: usize, rel_end: usize) -> DNMRange<'dnmrange> {
    DNMRange {
      start: self.start + rel_start,
      end: self.start + rel_end,
      dnm: self.dnm,
    }
  }

  /// checks whether the range is empty
  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}

impl<'dnmrange> Clone for DNMRange<'dnmrange> {
  fn clone(&self) -> DNMRange<'dnmrange> {
    DNMRange {
      start: self.start,
      end: self.end,
      dnm: self.dnm,
    }
  }
}