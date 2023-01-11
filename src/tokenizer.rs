//! Provides functionality for tokenizing sentences and words
use crate::dnm::{DNMRange, DNM};
use crate::stopwords;
use std::cmp;
use std::collections::vec_deque::*;
use std::collections::HashSet;
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
      abbreviations : Regex::new(r"^(?:C(?:[ft]|o(?:n[jn]|lo?|rp)?|a(?:l(?:if)?|pt)|mdr|p?l|res)|M(?:[dst]|a(?:[jnry]|ss)|i(?:ch|nn|ss)|o(?:nt)?|ex?|rs?)|A(?:r(?:[ck]|iz)|l(?:t?a)?|ttys?|ssn|dm|pr|ug|ve)|c(?:o(?:rp|l)?|(?:ap)?t|mdr|p?l|res|f)|S(?:e(?:ns?|pt?|c)|(?:up|g)?t|ask|r)|s(?:e(?:ns?|pt?|c)|(?:up|g)?t|r)|a(?:ttys?|ssn|dm|pr|rc|ug|ve|l)|P(?:enna?|-a.s|de?|lz?|rof|a)|D(?:e(?:[cfl]|p?t)|ist|ak|r)|I(?:[as]|n[cd]|da?|.e|ll)|F(?:e[bd]|w?y|ig|la|t)|O(?:k(?:la)?|[cn]t|re)|d(?:e(?:p?t|c)|ist|r)|E(?:xpy?|.g|sp|tc|qs?)|R(?:e(?:ps?|sp|v)|d)|T(?:e(?:nn|x)|ce|hm)|e(?:xpy?|.g|sp|tc|qs?)|m(?:[st]|a[jry]|rs?)|r(?:e(?:ps?|sp|v)|d)|N(?:e(?:br?|v)|ov?)|W(?:isc?|ash|yo?)|f(?:w?y|eb|ig|t)|p(?:de?|lz?|rof)|J(?:u[ln]|an|r)|U(?:SAFA|niv|t)|j(?:u[ln]|an|r)|K(?:ans?|en|y)|B(?:lv?d|ros)|b(?:lv?d|ros)|G(?:en|ov|a)|L(?:td?|a)|g(?:en|ov)|i(?:.e|nc)|l(?:td?|a)|[Hh]wa?y|V[ast]|Que|nov?|univ|Yuk|oct|tce|vs)\s?$").unwrap(),
    }
  }
}

// fn is_alphabetic_and_uppercase(c_opt: Option<&char>) -> bool {
//   if let Some(c) = c_opt {
//     c.is_alphabetic() && c.is_uppercase()
//   } else {
//     false
//   }
// }

/// detects a wordlike sequence with *any* uppercase char, such as "foobaR"
fn wordlike_with_upper_next(peekable: Peekable<Chars>) -> bool {
  let mut detected = false;
  for char in peekable {
    if !char.is_alphabetic() {
      break;
    }
    if char.is_uppercase() {
      detected = true;
      break;
    }
  }
  detected
}

impl Tokenizer {
  fn abbreviation_check(&self, left_window: &VecDeque<char>) -> bool {
    // Check for abbreviations:
    // Longest abbreviation is 6 characters, but mathformula is 11, so take window
    // of window_size chars to the left (allow for a space before dot)
    let lw_string: String = left_window.clone().into_iter().collect();
    let lw_str: &str = &lw_string;
    let lw_opt = lw_str.trim().split(|c: char| !c.is_alphabetic()).last();
    let lw_word: &str = match lw_opt {
      None => lw_str,
      Some(w) => w,
    };
    // Don't consider single letters followed by a punctuation sign an end of a
    // sentence, Also "a.m." and "p.m." shouldn't get split
    ((lw_word.len() == 1) && (lw_word != "I")) || self.abbreviations.is_match(lw_word)
  }

  // TODO: Reduce complexity, this tokenization pass is terribly overengineered
  /// gets the sentences from a dnm
  #[allow(clippy::cognitive_complexity)]
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
          // Baseline condition - only split when we have a following word-ish string with an
          // uppercase letter Get next non-space, non-quote character
          while text_iterator.peek().unwrap_or(&'.').is_whitespace()
            || text_iterator.peek() == Some(&'\'')
          {
            let space_char = text_iterator.next().unwrap();
            end += space_char.len_utf8();
          }
          if text_iterator.peek().is_none() {
            break;
          }
          // Uppercase next?
          if wordlike_with_upper_next(text_iterator.clone()) {
            // Ok, uppercase, but is it a stopword? If so, we must ALWAYS break the
            // sentence:
            let (next_word_string, next_word_length) = next_word_with_length(&mut text_iterator);
            let next_word_lc = next_word_string.to_lowercase();
            // Always break the sentence when we see a stopword
            if self.stopwords.contains(next_word_lc.as_str()) {
              // Reset the left window
              left_window = VecDeque::with_capacity(window_size);
              // New sentence
              sentences.push(DNMRange { start, end, dnm }.trim());
              start = end;
            } else {
              // Regular word case.
              if self.abbreviation_check(&left_window) {
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
                sentences.push(DNMRange { start, end, dnm }.trim());
                start = end;
              }
              // We consumed the next word, so make sure we reflect that in either case:
              for next_word_char in next_word_string.chars() {
                left_window.push_back(next_word_char);
                if left_window.len() >= window_size {
                  left_window.pop_front();
                }
              }
            }
            end += next_word_length;
          } else {
            // lowercase and non-alphanum characters
            match text_iterator.peek() {
              Some(&'*') | Some(&'"') | Some(&'(') => {
                // Reset the left window
                left_window = VecDeque::with_capacity(window_size);
                // New sentence
                sentences.push(DNMRange { start, end, dnm }.trim());
                start = end;
              },
              Some(&c) => {
                if sentence_char == '.' && c.is_alphabetic() {
                  let (next_word_string, next_word_length) =
                    next_word_with_length(&mut text_iterator);
                  // TODO: Maybe extend to more lowercase stopwords here? unclear...
                  if next_word_string.to_lowercase().starts_with("mathformula")
                    && !self.abbreviation_check(&left_window)
                  {
                    // Reset the left window
                    left_window = VecDeque::with_capacity(window_size);
                    // New sentence
                    sentences.push(DNMRange { start, end, dnm }.trim());
                    start = end;
                  } else {
                    left_window.push_back('.');
                    if left_window.len() >= window_size {
                      left_window.pop_front();
                    }
                  }
                  // We consumed the next word, so make sure we reflect that in either case:
                  for next_word_char in next_word_string.chars() {
                    left_window.push_back(next_word_char);
                    if left_window.len() >= window_size {
                      left_window.pop_front();
                    }
                  }
                  end += next_word_length;
                } else {
                  left_window.push_back('.');
                  if left_window.len() >= window_size {
                    left_window.pop_front();
                  }
                }
              },
              None => {
                left_window.push_back('.');
                if left_window.len() >= window_size {
                  left_window.pop_front();
                }
              },
            }
          }
        },
        '?' | '!' => {
          if !is_bounded(left_window.back(), text_iterator.peek()) {
            // Reset the left window
            left_window = VecDeque::with_capacity(window_size);
            // New sentence
            sentences.push(DNMRange { start, end, dnm }.trim());
            start = end;
          }
        },
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
            if text_iterator.peek().is_none() {
              break;
            }
            // Get the next word
            let (next_word_string, next_word_length) = next_word_with_length(&mut text_iterator);
            // Sentence-break, UNLESS a "mathformula" or a "lowercase word" follows, or a
            // non-alpha char
            if next_word_string.is_empty()
              || next_word_string.starts_with("mathformula")
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
              sentences.push(DNMRange { start, end, dnm }.trim());
              start = end;
            }

            // We consumed the next word, so make sure we reflect that in either case:
            end += next_word_length;
          }
        },
        other_char => {
          // "mathformula\nCapitalized" case is a sentence break (but never
          // "mathformula\nmathformula")
          if other_char.is_uppercase() {
            let lw_string: String = left_window.iter().collect();
            if lw_string.starts_with("mathformula") {
              // Sentence-break found, but exclude the current letter from the end:
              left_window = VecDeque::with_capacity(window_size);
              sentences.push(
                DNMRange {
                  start,
                  end: end - other_char.len_utf8(),
                  dnm,
                }
                .trim(),
              );
              start = end - other_char.len_utf8();
            }
          }
          // Increment the left window
          left_window.push_back(other_char);
          if left_window.len() >= window_size {
            left_window.pop_front();
          }
        },
      }
    }

    end = cmp::min(end, text.chars().count());
    let last_left_window: String = left_window.into_iter().collect();
    if last_left_window.find(char::is_alphabetic).is_some() {
      sentences.push(DNMRange { start, end, dnm }.trim());
    }

    // Filter out edge cases that return empty ranges
    sentences
      .into_iter()
      .filter(|range| range.start < range.end)
      .collect()
  }

  /// returns the words of a sentence using simple heuristics
  pub fn words<'b>(&'b self, sentence_range: &DNMRange<'b>) -> Vec<DNMRange> {
    let mut text_iterator = sentence_range.get_plaintext().chars().peekable();
    let mut start = 0usize;
    let mut end = 0usize;
    let mut result: Vec<DNMRange> = Vec::new();
    while let Some(c) = text_iterator.next() {
      end += c.len_utf8();
      if !c.is_alphanumeric() {
        if c == '\'' || c == '’' {
          if let Some(peeked) = text_iterator.peek() {
            if peeked == &'s' {
              continue;
            }
          }
        }
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

  /// returns the words and punctuation of a sentence, using simple heuristics
  #[allow(unused_assignments)]
  pub fn words_and_punct<'b>(&'b self, range: &DNMRange<'b>) -> Vec<DNMRange> {
    let range_text = range.get_plaintext();
    let text_iterator = range_text.chars();
    let mut start = 0usize;
    let mut end = 0usize;
    let mut result: Vec<DNMRange> = Vec::new();
    let mut apostrophe_flag = false;
    macro_rules! complete_word {
      () => {
        if start < end {
          if apostrophe_flag {
            match &range_text[start + 1..end] {
              // Handle closed set of apostrophe cases, detach from all other cases
              "t" | "s" | "un" | "th" | "ll" | "d" | "ve" | "il" | "re" | "m" => {},
              _ => {
                result.push(range.get_subrange(start, start + 1));
                start += 1;
              },
            }
          }
          result.push(range.get_subrange(start, end));
          apostrophe_flag = false;
          start = end;
        }
      };
    }

    for c in text_iterator {
      // letters, numbers can accumulate
      if c.is_alphanumeric() {
        end += c.len_utf8();
      } else {
        // everything else completes a word and starts a new one
        complete_word!();
        // except that whitepace can be skipped over
        if c.is_whitespace() {
          end += c.len_utf8();
          start = end;
        }
        // non-alphanum chars are standalone words EXCEPT when connectors such as apostrophes
        else {
          end += c.len_utf8();
          if c == '\'' || c == '’' {
            apostrophe_flag = true;
          } else {
            // standalone char word case
            complete_word!();
          }
        }
      }
    }
    complete_word!();
    result
  }
}
/// checks whether two characters are matching brackets or quotation marks
fn is_bounded<'a>(left: Option<&'a char>, right: Option<&'a char>) -> bool {
  let pair = [left, right];
  matches!(
    pair,
    [Some(&'['), Some(&']')]
      | [Some(&'('), Some(&')')]
      | [Some(&'{'), Some(&'}')]
      | [Some(&'\''), Some(&'\'')]
      | [Some(&'"'), Some(&'"')]
  )
}

/// Obtains the next word from the `Peekable<Chars>` iterator, where only
/// alphabetic characters are accepted, and a max length of 20 is imposed
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
