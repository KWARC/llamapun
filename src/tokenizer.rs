//! Provides functionality for tokenizing sentences and words
use dnm::{DNMRange, DNM};
use stopwords;
use std::collections::vec_deque::*;
use std::collections::HashSet;
use std::cmp;
use std::iter::Peekable;
use std::str::Chars;

use regex::Regex;

/// Stores auxiliary resources required by the tokenizer so that they need to be initialized only
/// once
pub struct Tokenizer {
  /// set of stopwords
  pub stopwords: HashSet<&'static str>,
  /// regular expression for abbreviations
  pub abbreviations: Regex,
}
impl Default for Tokenizer {
  fn default() -> Tokenizer {
    Tokenizer {
      stopwords : stopwords::load(),
      abbreviations : Regex::new(r"^(?:C(?:[ft]|o(?:n[jn]|lo?|rp)?|a(?:l(?:if)?|pt)|mdr|p?l|res)|M(?:[dst]|a(?:[jnry]|ss)|i(?:ch|nn|ss)|o(?:nt)?|ex?|rs?)|A(?:r(?:[ck]|iz)|l(?:t?a)?|ttys?|ssn|dm|pr|ug|ve)|c(?:o(?:rp|l)?|(?:ap)?t|mdr|p?l|res|f)|S(?:e(?:ns?|pt?|c)|(?:up|g)?t|ask|r)|s(?:e(?:ns?|pt?|c)|(?:up|g)?t|r)|a(?:ttys?|ssn|dm|pr|rc|ug|ve|l)|P(?:enna?|-a.s|de?|lz?|rof|a)|D(?:e(?:[cfl]|p?t)|ist|ak|r)|I(?:[as]|n[cd]|da?|.e|ll)|F(?:e[bd]|w?y|ig|la|t)|O(?:k(?:la)?|[cn]t|re)|d(?:e(?:p?t|c)|ist|r)|E(?:xpy?|.g|sp|tc|q)|R(?:e(?:ps?|sp|v)|d)|T(?:e(?:nn|x)|ce|hm)|e(?:xpy?|.g|sp|tc|q)|m(?:[st]|a[jry]|rs?)|r(?:e(?:ps?|sp|v)|d)|N(?:e(?:br?|v)|ov?)|W(?:isc?|ash|yo?)|f(?:w?y|eb|ig|t)|p(?:de?|lz?|rof)|J(?:u[ln]|an|r)|U(?:SAFA|niv|t)|j(?:u[ln]|an|r)|K(?:ans?|en|y)|B(?:lv?d|ros)|b(?:lv?d|ros)|G(?:en|ov|a)|L(?:td?|a)|g(?:en|ov)|i(?:.e|nc)|l(?:td?|a)|[Hh]wa?y|V[ast]|Que|nov?|univ|Yuk|oct|tce|vs)\s?$").unwrap(),
    }
  }
}

impl Tokenizer {
  /// gets the sentences from a dnm
  pub fn sentences<'a>(&self, dnm: &'a DNM) -> Vec<DNMRange<'a>> {
    let text = &dnm.plaintext;
    let mut sentences: Vec<DNMRange<'a>> = Vec::new();
    let mut text_iterator = text.chars().peekable();
    let mut start = 0;
    let mut end = 0;
    let window_size = 12; // size of max string + 1
    let mut left_window: VecDeque<char> = VecDeque::with_capacity(window_size);

    while let Some(sentence_char) = text_iterator.next() {
      // Bookkeep the end position
      end += sentence_char.len_utf8();

      match sentence_char {
        '.' | ':' => {
          // Baseline condition - only split when we have a following uppercase letter
          // Get next non-space, non-quote character
          while text_iterator.peek().unwrap_or(&'.').is_whitespace()
            || text_iterator.peek() == Some(&'\'')
          {
            let space_char = text_iterator.next().unwrap();
            end += space_char.len_utf8();
          }
          if text_iterator.peek() == None {
            break;
          }
          // Uppercase next?
          if text_iterator.peek().unwrap().is_uppercase() {
            // Ok, uppercase, but is it a stopword? If so, we must ALWAYS break the sentence:
            let (next_word_string, next_word_length) = next_word_with_length(&mut text_iterator);
            let next_word_lc = next_word_string.to_lowercase();
            // Always break the sentence when we see a stopword
            if self.stopwords.contains(next_word_lc.as_str()) {
              // Reset the left window
              left_window = VecDeque::with_capacity(window_size);
              // New sentence
              sentences.push(
                DNMRange {
                  start: start,
                  end: end,
                  dnm: dnm,
                }.trim(),
              );
              start = end;
              end += next_word_length;
            } else {
              // Regular word case.
              // Check for abbreviations:
              // Longest abbreviation is 6 characters, but MathFormula is 11, so take window of window_size chars to the left (allow for a space before dot)
              let lw_string: String = left_window.clone().into_iter().collect();
              let lw_str: &str = &lw_string;
              let lw_opt = lw_str.trim().split(|c: char| !c.is_alphabetic()).last();
              let lw_word: &str = match lw_opt {
                None => lw_str,
                Some(w) => w,
              };
              // Don't consider single letters followed by a punctuation sign an end of a sentence,
              // Also "a.m." and "p.m." shouldn't get split
              if ((lw_word.len() == 1) && (lw_word != "I")) ||
                  // Don't sentence-break colons followed by a formula
                  ((sentence_char == ':') && (next_word_string == "MathFormula"))
                || self.abbreviations.is_match(lw_word)
              {
                left_window.push_back('.');
                if left_window.len() >= window_size {
                  left_window.pop_front();
                }
              }
              //TODO: Handle dot-dot-dot "..."
              else {
                // Not a special case, break the sentence
                // Reset the left window
                left_window = VecDeque::with_capacity(window_size);
                // New sentence
                sentences.push(
                  DNMRange {
                    start: start,
                    end: end,
                    dnm: dnm,
                  }.trim(),
                );
                start = end;
              }
              // We consumed the next word, so make sure we reflect that in either case:
              for next_word_char in next_word_string.chars() {
                left_window.push_back(next_word_char);
                if left_window.len() >= window_size {
                  left_window.pop_front();
                }
              }
              end += next_word_length;
            }
          } else {
            // lowercase and non-alphanum characters
            match text_iterator.peek() {
              Some(&'*') | Some(&'"') | Some(&'(') => {
                // Reset the left window
                left_window = VecDeque::with_capacity(window_size);
                // New sentence
                sentences.push(
                  DNMRange {
                    start: start,
                    end: end,
                    dnm: dnm,
                  }.trim(),
                );
                start = end;
              }
              _ => {
                // TODO: Maybe break on lowercase stopwords here? unclear...
                left_window.push_back('.');
                if left_window.len() >= window_size {
                  left_window.pop_front();
                }
              }
            }
          }
        }
        '?' | '!' => {
          if !is_bounded(left_window.back(), text_iterator.peek()) {
            // Reset the left window
            left_window = VecDeque::with_capacity(window_size);
            // New sentence
            sentences.push(
              DNMRange {
                start: start,
                end: end,
                dnm: dnm,
              }.trim(),
            );
            start = end;
          }
        }
        // TODO:
        // Some('\u{2022}'),Some('*') => { // bullet point for itemize
        // Some('\u{220e}') => { // QED symbol
        '\n' => {
          // newline
          if let Some(&'\n') = text_iterator.peek() {
            // second newline
            // Get next non-space character
            while text_iterator.peek().unwrap_or(&'.').is_whitespace() {
              let space_char = text_iterator.next().unwrap();
              end += space_char.len_utf8();
            }
            if text_iterator.peek() == None {
              break;
            }
            // Get the next word
            let (next_word_string, next_word_length) = next_word_with_length(&mut text_iterator);
            // Sentence-break, UNLESS a "MathFormula" or a "lowercase word" follows, or a non-alpha char
            if next_word_string.is_empty() || (next_word_string == "MathFormula")
              || next_word_string.chars().next().unwrap().is_lowercase()
            {
              // We consumed the next word, add it to the left window
              for next_word_char in next_word_string.chars() {
                left_window.push_back(next_word_char);
                if left_window.len() >= window_size {
                  left_window.pop_front();
                }
              }
            } else {
              // Sentence-break found:
              left_window = VecDeque::with_capacity(window_size);
              sentences.push(
                DNMRange {
                  start: start,
                  end: end,
                  dnm: dnm,
                }.trim(),
              );
              start = end;
            }

            // We consumed the next word, so make sure we reflect that in either case:
            end += next_word_length;
          }
        }
        other_char => {
          // "MathFormula\nCapitalized" case is a sentence break (but never "MathFormula\nMathFormula")
          if other_char.is_uppercase() && other_char != 'M' {
            let lw_string: String = left_window.clone().into_iter().collect();
            if lw_string == "MathFormula" {
              // Sentence-break found, but exclude the current letter from the end:
              left_window = VecDeque::with_capacity(window_size);
              sentences.push(
                DNMRange {
                  start: start,
                  end: end - other_char.len_utf8(),
                  dnm: dnm,
                }.trim(),
              );
              start = end - other_char.len_utf8();
            }
          }
          // Increment the left window
          left_window.push_back(other_char);
          if left_window.len() >= window_size {
            left_window.pop_front();
          }
        }
      }
    }

    end = cmp::min(end, text.chars().count());
    let last_left_window: String = left_window.into_iter().collect();
    if let Some(_) = last_left_window.find(|c: char| c.is_alphabetic()) {
      sentences.push(
        DNMRange {
          start: start,
          end: end,
          dnm: dnm,
        }.trim(),
      );
    }

    // Filter out edge cases that return empty ranges
    sentences
      .into_iter()
      .filter(|range| range.start < range.end)
      .collect()
  }

  /// returns the words of a sentence using simple heuristics
  pub fn words<'a, 'b>(&'b self, sentence_range: &'a DNMRange<'b>) -> Vec<DNMRange> {
    let text_iterator = sentence_range.get_plaintext().chars().peekable();
    let mut start = 0usize;
    let mut end = 0usize;
    let mut result: Vec<DNMRange> = Vec::new();
    for c in text_iterator {
      end += c.len_utf8();
      if !c.is_alphanumeric() {
        if start < end - c.len_utf8() {
          result.push(sentence_range.get_subrange(start, end - c.len_utf8()));
        }
        start = end;
      }
    }
    if start < end {
      result.push(sentence_range.get_subrange(start, end));
    }
    result
  }
}

/// checks whether two characters are matching brackets or quotation marks
fn is_bounded<'a>(left: Option<&'a char>, right: Option<&'a char>) -> bool {
  let pair = [left, right];
  match pair {
    [Some(&'['), Some(&']')]
    | [Some(&'('), Some(&')')]
    | [Some(&'{'), Some(&'}')]
    | [Some(&'"'), Some(&'"')] => true,
    _ => false,
  }
}

/// Obtains the next word from the `Peekable<Chars>` iterator, where only alphabetic characters are accepted, and a max length of 20 is imposed
fn next_word_with_length(text_iterator: &mut Peekable<Chars>) -> (String, usize) {
  let mut next_word_length = 0;
  let mut next_word: Vec<char> = Vec::new();
  while next_word_length < 20 && text_iterator.peek().unwrap_or(&'.').is_alphabetic() {
    let word_char = text_iterator.next().unwrap();
    next_word.push(word_char);
    next_word_length += word_char.len_utf8();
  }

  let next_word: String = next_word.into_iter().collect();
  (next_word, next_word_length)
}
