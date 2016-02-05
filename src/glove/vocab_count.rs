/// Extract unigram counts
///
/// Adapted from: https://raw.githubusercontent.com/stanfordnlp/GloVe/master/src/vocab_count.c

use data::Corpus;
use ngrams::Unigrams;

const MAX_STRING_LENGTH : i32 = 1000;
const TSIZE : i32 = 1048576;
const SEED : i32 = 1159241;

pub struct CountOptions {
  verbose : u32, // 0,1, or 2
  min_count : u32, // min occurrences for inclusion in vocab
  max_vocab : Option<u32>, // None for no limit
}
impl Default for CountOptions {
  fn default() -> CountOptions {
    CountOptions {
      verbose : 2,
      min_count : 1,
      max_vocab : None
    }
  }
}

pub fn get_counts(mut corpus : Corpus, input_options: Option<CountOptions>) {
  let options = match input_options {
    Some(opts) => opts,
    None => CountOptions::default()
  };
  let mut vocabulary = Unigrams::new();
  let mut token_count : usize = 0;
  let mut j : usize = 0;
  let mut vocab_size : usize = 12500;
  let mut format : String;

  println!("BUILDING VOCABULARY");
  if options.verbose > 1 {
    println!("Processed {:?} tokens.", token_count);
  }
  // Insert all tokens into a hash table
  for mut document in corpus.iter() {
    for mut paragraph in document.iter() {
      for mut sentence in paragraph.iter() {
        for mut word in sentence.iter() {
          vocabulary.insert(word.text.to_owned());
          token_count += 1;
          if options.verbose > 1 {
            if token_count % 100000 == 0 {
              println!("\033[11G{:?} tokens.", token_count);
            }
          }
        }
      }
    }
  }
  if options.verbose > 1 {
    println!("\033[0GProcessed {:?} tokens.\n", token_count);
  }
  // vocab = malloc(sizeof(VOCAB) * vocab_size);
  // for (i = 0; i < TSIZE; i++) { // Migrate vocab to array
  //   htmp = vocab_hash[i];
  //   while (htmp != NULL) {
  //     vocab[j].word = htmp->word;
  //     vocab[j].count = htmp->count;
  //     j++;
  //     if (j>=vocab_size) {
  //       vocab_size += 2500;
  //       vocab = (VOCAB *)realloc(vocab, sizeof(VOCAB) * vocab_size);
  //     }
  //     htmp = htmp->next;
  //   }
  // }
  // if (verbose > 1) fprintf(stderr, "Counted %lld unique words.\n", j);
  // if (max_vocab > 0 && max_vocab < j)
  //     // If the vocabulary exceeds limit, first sort full vocab by frequency without alphabetical tie-breaks.
  //     // This results in pseudo-random ordering for words with same frequency, so that when truncated, the words span whole alphabet
  //     qsort(vocab, j, sizeof(VOCAB), CompareVocab);
  // else max_vocab = j;
  // qsort(vocab, max_vocab, sizeof(VOCAB), CompareVocabTie); //After (possibly) truncating, sort (possibly again), breaking ties alphabetically

  // for (i = 0; i < max_vocab; i++) {
  //     if (vocab[i].count < min_count) { // If a minimum frequency cutoff exists, truncate vocabulary
  //         if (verbose > 0) fprintf(stderr, "Truncating vocabulary at min count %lld.\n",min_count);
  //         break;
  //     }
  //     printf("%s %lld\n",vocab[i].word,vocab[i].count);
  // }

  // if (i == max_vocab && max_vocab < j) if (verbose > 0) fprintf(stderr, "Truncating vocabulary at size %lld.\n", max_vocab);
  // fprintf(stderr, "Using vocabulary of size %lld.\n\n", i);
  // return 0;
}

/* Vocab frequency comparison; break ties alphabetically */
// int CompareVocabTie(const void *a, const void *b) {
//     long long c;
//     if ( (c = ((VOCAB *) b)->count - ((VOCAB *) a)->count) != 0) return ( c > 0 ? 1 : -1 );
//     else return (scmp(((VOCAB *) a)->word,((VOCAB *) b)->word));
// }

/* Vocab frequency comparison; no tie-breaker */
// int CompareVocab(const void *a, const void *b) {
//     long long c;
//     if ( (c = ((VOCAB *) b)->count - ((VOCAB *) a)->count) != 0) return ( c > 0 ? 1 : -1 );
//     else return 0;
// }


