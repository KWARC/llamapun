/// Extract unigram counts
///
/// Adapted from: https://raw.githubusercontent.com/stanfordnlp/GloVe/master/src/vocab_count.c

use std::cmp::Ordering;
use data::Corpus;
use ngrams::Unigrams;

pub struct CountOptions {
  verbose : bool,
  min_count : usize, // min occurrences for inclusion in vocab
  max_vocab : Option<usize>, // None for no limit
}
impl Default for CountOptions {
  fn default() -> CountOptions {
    CountOptions {
      verbose : true,
      min_count : 1,
      max_vocab : None
    }
  }
}

pub fn get_counts(mut corpus : Corpus, input_options: Option<CountOptions>) -> Vec<(String, usize)> {
  let options = match input_options {
    Some(opts) => opts,
    None => CountOptions::default()
  };
  let mut vocabulary = Unigrams::new();
  let mut token_count : usize = 0;

  if options.verbose {
    println_stderr!("BUILDING VOCABULARY");
    println_stderr!("Processed {:?} tokens.", token_count);
  }
  // Insert all tokens into a hash table
  for mut document in corpus.iter() {
    for mut paragraph in document.iter() {
      for mut sentence in paragraph.iter() {
        for word in sentence.iter() {
          vocabulary.insert(word.text.to_owned());
          token_count += 1;
          if options.verbose && (token_count % 100000 == 0) {
            println_stderr!("\033[11G{:?} tokens.", token_count);
          }
        }
      }
    }
  }
  if options.verbose {
    println_stderr!("\033[0GProcessed {:?} tokens.\n", token_count);
  }

  let unique_words_count = vocabulary.count();
  let mut vocab : Vec<(String , usize)> = Vec::with_capacity(unique_words_count);
  if options.verbose {
    println_stderr!("Counted {:?} unique words.\n", unique_words_count);
  }

  let truncated_size = match options.max_vocab {
    None => unique_words_count,
    Some(limit) => if limit < unique_words_count {
      // If the vocabulary exceeds limit, first sort full vocab by frequency without alphabetical tie-breaks.
      // This results in pseudo-random ordering for words with same frequency, so that when truncated, the words span whole alphabet
      vocab.sort_by(|a,b| a.1.cmp(&b.1));
      vocab.truncate(limit);
      limit
    } else {
      unique_words_count
    }
  };

  //After (possibly) truncating, sort (possibly again), breaking ties alphabetically
  vocab.sort_by(|a,b| match a.1.cmp(&b.1){
    Ordering::Less => Ordering::Less,
    Ordering::Greater => Ordering::Greater,
    Ordering::Equal => a.0.cmp(&b.0)
  });

  if options.min_count > 1 { // Truncate if requested
    let mut min_count_truncate_index = 0;
    for item in vocab.iter() {
      if item.1 < options.min_count {
        if options.verbose {
          println_stderr!("Truncating vocabulary at min count {:?}", options.min_count);
        }
        break;
      } else {
        min_count_truncate_index+=1;
      }
    }
    vocab.truncate(min_count_truncate_index);
    println_stderr!("Using vocabulary of size {:?}.\n", min_count_truncate_index);
  }
  if options.verbose {
    // If we're running in verbose mode, also write the vocabulary to disk

  }
  vocab
}
