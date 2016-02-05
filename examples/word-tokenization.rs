extern crate llamapun;
extern crate libxml;
extern crate libc;
extern crate time;

use std::collections::HashMap;
use time::PreciseTime;
use libxml::parser::Parser;

use llamapun::tokenizer::Tokenizer;
use llamapun::ngrams::{Dictionary,Unigrams};
use llamapun::data::{Corpus,Document};
use llamapun::util::plot::*;

fn main() {
  let start_example = PreciseTime::now();
  let mut dictionary = Dictionary::new();
  let mut unigrams = Unigrams::new();
  let mut word_index = 0;
  let mut total_words = 0;
  let mut total_sentences = 0;
  let mut total_paragraphs = 0;
  let mut dict = Dictionary::default();
  let mut unigrams = Unigrams::default();

  // let corpus = Corpus {
  //   path: "tests/resources/".to_string(),
  //   parser : Parser::default_html(),
  //   // Use the default tokenizer, in a single variable globally to the document
  //   tokenizer : Tokenizer::default()
  // };
  let corpus = Corpus::new("tests/resources/".to_string());
  let arxivid = "0903.1000";
  let mut document = Document::new("tests/resources/".to_string()+arxivid+".html", &corpus).unwrap();
  let end_parse = PreciseTime::now();

  // We will tokenize each logical paragraph, which are the textual logical units in an article
  for mut paragraph in document.paragraph_iter() {
    total_paragraphs += 1;
    for mut sentence in paragraph.iter() {
      total_sentences += 1;
      for sent_word in sentence.simple_iter() {
        total_words += 1;
        let word = sent_word.text.to_string().to_lowercase();
        dictionary.insert(word.clone());
        unigrams.insert(word);
      }
    }
  }
  let end_example = PreciseTime::now();

  // Word frequencies in order of document appearance
  let inorder_dictionary = dictionary.sort();
  let mut inorder_frequency: Vec<(usize, usize)> = Vec::new();
  for entry in inorder_dictionary.iter() {
    let frequency = unigrams.get(&entry.0);
    inorder_frequency.push((entry.1.clone(), frequency));
  }
  plot_simple(&inorder_frequency,
    "Word index, in order of document occurrence",
    "Frequency counts (log2)",
    "Word Frequencies",
    "inorder_word_freq.png");

  // Sorted gnuplot of frequency distribution:
  let mut frequency_distribution = HashMap::new();
  // Obtain the distribution from the raw frequency data
  for &(_,value) in inorder_frequency.iter() {
    let words_with_frequency = frequency_distribution.entry(value).or_insert(0);
    *words_with_frequency += 1;
  }
  // Perform sort
  let mut value_sorted_frequencies = Vec::new();
  for (index,value) in frequency_distribution.iter() {
    value_sorted_frequencies.push((value.clone(), index.clone())); // ( # Distinct words , Frequency )
  }
  value_sorted_frequencies.sort_by(|a, b| a.1.cmp(&b.1));
  plot_simple(&value_sorted_frequencies,
    "Distinct words with this frequency",
    "Frequency (log2)",
    "Frequency Distribution",
    "distribution_word_freq.png");

  // Print out the final report:
  println!("--- Dictionary: \n{:?}\n\n", inorder_dictionary);
  println!("--- Frequencies: \n{:?}\n\n", unigrams.sort());
  println!("--- Frequency distribution: \n{:?}\n\n", value_sorted_frequencies);
  println!("--- Paragraphs total: {:?}",total_paragraphs);
  println!("--- Sentences total: {:?}",total_sentences);
  println!("--- Words total: {:?}",total_words);
  println!("--- Words distinct: {:?}",dictionary.count());
  println!("");

  // As well as some basic Benchmarking info:
  let end_reports = PreciseTime::now();
  println!("--- Benchmark report:");
  println!("    LibXML parse took {:?}ms",start_example.to(end_parse).num_milliseconds());
  println!("    LLaMaPun word tokenization took {:?}ms",end_parse.to(end_example).num_milliseconds());
  println!("    Finished report generation in {:?}ms",end_example.to(end_reports).num_milliseconds());
  println!("    Total time: {:?}ms", start_example.to(end_reports).num_milliseconds());
}

