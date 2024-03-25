extern crate libc;
extern crate libxml;
extern crate llamapun;

use std::collections::HashMap;
use std::time::Instant;

use llamapun::data::{Corpus, Document};
use llamapun::ngrams::{Dictionary, Ngrams};
use llamapun::util::plot::*;

fn main() {
  let start_example = Instant::now();

  let mut dictionary = Dictionary::new();
  let mut unigrams = Ngrams::default();
  let mut total_words = 0;
  let mut total_sentences = 0;
  let mut total_paragraphs = 0;

  // let corpus = Corpus {
  //   path: "tests/resources/".to_string(),
  //   parser : Parser::default_html(),
  //   // Use the default tokenizer, in a single variable globally to the document
  //   tokenizer : Tokenizer::default()
  // };
  let corpus = Corpus::new("tests/resources/".to_string());
  let arxivid = "0903.1000";
  let mut document =
    Document::new("tests/resources/".to_string() + arxivid + ".html", &corpus).unwrap();
  let end_parse = start_example.elapsed().as_millis();

  // We will tokenize each logical paragraph, which are the textual logical units
  // in an article
  for mut paragraph in document.paragraph_iter() {
    total_paragraphs += 1;
    for mut sentence in paragraph.iter() {
      total_sentences += 1;
      for sent_word in sentence.simple_iter() {
        total_words += 1;
        let word = sent_word.range.get_plaintext().to_string().to_lowercase();
        dictionary.insert(word.clone());
        unigrams.insert(word);
      }
    }
  }
  let end_example = start_example.elapsed().as_millis();

  // Word frequencies in order of document appearance
  let inorder_dictionary = dictionary.sorted();
  let mut inorder_frequency: Vec<(usize, usize)> = Vec::new();
  for entry in &inorder_dictionary {
    let frequency = unigrams.get(entry.0);
    inorder_frequency.push((entry.1, frequency));
  }
  plot_simple(
    &inorder_frequency,
    "Word index, in order of document occurrence",
    "Frequency counts (log2)",
    "Word Frequencies",
    "inorder_word_freq.png",
  );

  // Sorted gnuplot of frequency distribution:
  let mut frequency_distribution = HashMap::new();
  // Obtain the distribution from the raw frequency data
  for (_, value) in inorder_frequency {
    let words_with_frequency = frequency_distribution.entry(value).or_insert(0);
    *words_with_frequency += 1;
  }
  // Perform sort
  let mut value_sorted_frequencies = Vec::new();
  for (index, value) in frequency_distribution {
    value_sorted_frequencies.push((value, index)); // ( # Distinct words , Frequency )
  }
  value_sorted_frequencies.sort_by(|a, b| a.1.cmp(&b.1));
  plot_simple(
    &value_sorted_frequencies,
    "Distinct words with this frequency",
    "Frequency (log2)",
    "Frequency Distribution",
    "distribution_word_freq.png",
  );

  // Print out the final report:
  println!("--- Dictionary: \n{:?}\n", inorder_dictionary);
  println!("--- Frequencies: \n{:?}\n", unigrams.sorted());
  println!(
    "--- Frequency distribution: \n{:?}\n",
    value_sorted_frequencies
  );
  println!("--- Paragraphs total: {:?}", total_paragraphs);
  println!("--- Sentences total: {:?}", total_sentences);
  println!("--- Words total: {:?}", total_words);
  println!("--- Words distinct: {:?}", dictionary.count());
  println!();

  // As well as some basic Benchmarking info:
  let end_reports = start_example.elapsed().as_millis();
  println!("--- Benchmark report:");
  println!("    LibXML parse took {:?}ms", end_parse);
  println!(
    "    LLaMaPun word tokenization took {:?}ms",
    end_example - end_parse
  );
  println!(
    "    Finished report generation in {:?}ms",
    end_reports - end_example
  );
  println!("    Total time: {:?}ms", end_reports);
}
