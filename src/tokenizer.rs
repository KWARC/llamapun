use dnm::*;
use stopwords;
use std::collections::vec_deque::*;
use std::collections::HashSet;
use std::cmp;
use regex::Regex;


// Only initialize auxiliary resources once and keep them in a Tokenizer struct
pub struct Tokenizer <'a> {
 pub stopwords : HashSet<&'a str>,
 pub abbreviations : Regex,
}
impl <'a> Default for Tokenizer <'a> {
  fn default() -> Tokenizer <'a> {
    Tokenizer {
      stopwords : stopwords::load(),
      abbreviations : Regex::new(r"^(?:C(?:[ft]|o(?:n[jn]|lo?|rp)?|a(?:l(?:if)?|pt)|mdr|p?l|res)|M(?:[dst]|a(?:[jnry]|ss)|i(?:ch|nn|ss)|o(?:nt)?|ex?|rs?)|A(?:r(?:[ck]|iz)|l(?:t?a)?|ttys?|ssn|dm|pr|ug|ve)|c(?:o(?:rp|l)?|(?:ap)?t|mdr|p?l|res|f)|S(?:e(?:ns?|pt?|c)|(?:up|g)?t|ask|r)|s(?:e(?:ns?|pt?|c)|(?:up|g)?t|r)|a(?:ttys?|ssn|dm|pr|rc|ug|ve|l)|P(?:enna?|-a.s|de?|lz?|rof|a)|D(?:e(?:[cfl]|p?t)|ist|ak|r)|I(?:[as]|n[cd]|da?|.e|ll)|F(?:e[bd]|w?y|ig|la|t)|O(?:k(?:la)?|[cn]t|re)|d(?:e(?:p?t|c)|ist|r)|E(?:xpy?|.g|sp|tc|q)|R(?:e(?:ps?|sp|v)|d)|T(?:e(?:nn|x)|ce|hm)|e(?:xpy?|.g|sp|tc|q)|m(?:[st]|a[jry]|rs?)|r(?:e(?:ps?|sp|v)|d)|N(?:e(?:br?|v)|ov?)|W(?:isc?|ash|yo?)|f(?:w?y|eb|ig|t)|p(?:de?|lz?|rof)|J(?:u[ln]|an|r)|U(?:SAFA|niv|t)|j(?:u[ln]|an|r)|K(?:ans?|en|y)|B(?:lv?d|ros)|b(?:lv?d|ros)|G(?:en|ov|a)|L(?:td?|a)|g(?:en|ov)|i(?:.e|nc)|l(?:td?|a)|[Hh]wa?y|V[ast]|Que|nov?|univ|Yuk|oct|tce|vs)\s?$").unwrap(),
    }
  }
}

impl <'a> Tokenizer <'a> {
  pub fn sentences(&self, dnm: &'a DNM) -> Vec<DNMRange <'a>> {
    let text = dnm.plaintext.clone();
    let mut sentences : Vec<DNMRange <'a>> = Vec::new();
    let mut text_iterator = text.chars().peekable();
    let mut start = 0;
    let mut end = 0;
    let mut left_window : VecDeque<char> = VecDeque::with_capacity(9);

    loop {
      let c = text_iterator.next();
      // Length increase:
      match c {
        Some(x) => {
          end += x.len_utf8(); }
        None => {}
      }
      
      match c {
        Some('.') | Some(':') => {
          // Baseline condition - only split when we have a following uppercase letter
          // Get next non-space, non-quote character
          while (text_iterator.peek() != None) && 
                (text_iterator.peek().unwrap().is_whitespace() ||
                 text_iterator.peek() == Some(&'\'')) {
            let space_char = text_iterator.next().unwrap();
            end+= space_char.len_utf8();
          }
          if text_iterator.peek() == None {break;}
          // Uppercase next?
          if text_iterator.peek().unwrap().is_uppercase() {
            // Ok, uppercase, but is it a stopword? If so, we must ALWAYS break the sentence:
            let mut next_word_length = 0;
            let mut next_word : Vec<char> = Vec::new();
            while (text_iterator.peek() != None) && text_iterator.peek().unwrap().is_alphabetic() && (next_word_length<20) {
              let word_char = text_iterator.next().unwrap();
              next_word.push(word_char);
              next_word_length += word_char.len_utf8();
            }
            // There must be a cleaner way of doing this recast into &str
            let next_word_string : String = next_word.into_iter().collect();
            let lower_word_string : String = next_word_string.to_lowercase();
            let lower_word_str : &str = &lower_word_string;
            // Always break the sentence when we see a stopword
            if self.stopwords.contains(lower_word_str) {
              // Reset the left window        
              left_window = VecDeque::with_capacity(9);
              // New sentence
              sentences.push(DNMRange{start: start, end: end, dnm: dnm}.trim());
              start = end;
              end+=next_word_length;
            } else { // Regular word case.
              // Check for abbreviations:
              // Longest abbreviation is 6 characters, so take window of 8 chars to the left (allow for a space before dot)
              let lw_string : String = left_window.clone().into_iter().collect();
              let lw_str : &str = &lw_string;
              let lw_opt = lw_str.trim().split(|c: char| !c.is_alphabetic()).last();
              let lw_word : &str = match lw_opt {
                None => lw_str,
                Some(w) => w 
              };
              // Don't consider single letters followed by a punctuation sign an end of a sentence,
              // Also "a.m." and "p.m." shouldn't get split
              if (lw_word.len() == 1) && (lw_word != "I") ||
                // Don't sentence-break colons followed by a formula
                (c == Some(':')) && (next_word_string == "MathFormula") ||
                self.abbreviations.is_match(lw_word) {
                
                left_window.push_back('.'); 
                if left_window.len() > 8 { left_window.pop_front(); }
              }
              //TODO: Handle dot-dot-dot "..."
              else { // Not a special case, break the sentence
                // Reset the left window        
                left_window = VecDeque::with_capacity(9);
                // New sentence
                sentences.push(DNMRange{start: start, end: end, dnm: dnm}.trim());
                start = end;
              }
              // We consumed the next word, so make sure we reflect that in either case:
              for next_word_char in next_word_string.chars() {
                left_window.push_back(next_word_char); 
                if left_window.len() > 8 { left_window.pop_front(); } 
              }
              end += next_word_length;
            }
          }
          else { // lowercase and non-alphanum characters
            match text_iterator.peek() {
              Some(&'*') | Some(&'"') | Some(&'(') => {
                // Reset the left window
                left_window = VecDeque::with_capacity(9);
                // New sentence
                sentences.push(DNMRange{start: start, end: end, dnm: dnm}.trim());
                start = end; 
              },
              _ => {
                // TODO: Maybe break on lowercase stopwords here? unclear...
                left_window.push_back('.'); 
                if left_window.len() > 8 { left_window.pop_front(); } 
              }
            }
          }
        },
        Some('?') | Some('!') => {
          if !is_bounded(left_window.back(),text_iterator.peek()) {
            // Reset the left window
            left_window = VecDeque::with_capacity(9);
            // New sentence
            sentences.push(DNMRange{start: start, end: end, dnm: dnm}.trim());
            start = end;
          }
        },
        // TODO: 
        // Some('\u{2022}'),Some('*') => { // bullet point for itemize
        // Some('\u{220e}') => { // QED symbol 
        Some('\n') => { // newline
          match text_iterator.peek() {
            Some(&'\n') => { // second newline             
              // Get next non-space character
              while (text_iterator.peek() != None) && text_iterator.peek().unwrap().is_whitespace() {
                let space_char = text_iterator.next().unwrap();
                end+= space_char.len_utf8();
              }
              if text_iterator.peek() == None {break;}
              // Get the next word
              let mut next_word_length = 0;
              let mut next_word : Vec<char> = Vec::new();
              while (text_iterator.peek() != None) && text_iterator.peek().unwrap().is_alphabetic() && (next_word_length<20) {
                let word_char = text_iterator.next().unwrap();
                next_word.push(word_char);
                next_word_length += word_char.len_utf8();
              }
              // There must be a cleaner way of doing this recast into &str
              let is_lower_word = next_word.len()>0 && next_word[0].is_lowercase();
              let next_word_string : String = next_word.into_iter().collect();
              // Sentence-break, UNLESS a "MathFormula" or a "lowercase word" follows, or a non-alpha char
              if (next_word_string == "") || (next_word_string == "MathFormula") || (is_lower_word) {
                // We consumed the next word, add it to the left window
                for next_word_char in next_word_string.chars() {
                  left_window.push_back(next_word_char); 
                  if left_window.len() > 8 { left_window.pop_front(); } 
                }
              }
              else {
                // Sentence-break found:
                left_window = VecDeque::with_capacity(9);
                sentences.push(DNMRange{start: start, end: end, dnm: dnm}.trim());
                start = end;
              }

              // We consumed the next word, so make sure we reflect that in either case:
              end += next_word_length;
            }
            _ => {}
          }
        },
        Some(x) => {
          // Increment the left window
          left_window.push_back(x);
          if left_window.len() > 8 {
            left_window.pop_front();
          }
        },
        None => { break; }
      }
    }
    end = cmp::min(end, text.len());
    let last_left_window : String = left_window.clone().into_iter().collect();
    let alpha_char = last_left_window.find(|c: char| c.is_alphabetic());
    if alpha_char != None {
      sentences.push(DNMRange{start: start, end: end, dnm: dnm}.trim());
    }
    return sentences;
  }

  pub fn words(&self, sentence_range: &'a DNMRange) -> Vec<&'a str>  {
    sentence_range.get_plaintext().split(|c: char| !c.is_alphabetic()).filter(|w| w.len() > 0).collect()
  }
}

fn is_bounded<'a>(left: Option<&'a char>, right: Option<&'a char>) -> bool {
  let pair = [left, right];
  return match pair {
    [Some(&'['), Some(&']')] | [Some(&'('), Some(&')')] | [Some(&'{'), Some(&'}')] | [Some(&'"'), Some(&'"')] => {
      true
    },
    _ => {
      false
    }
  };
}
