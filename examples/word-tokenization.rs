extern crate llamapun;
extern crate libxml;
extern crate libc;
extern crate gnuplot;
extern crate time;

use std::collections::HashMap;
use time::PreciseTime;
use gnuplot::*;

use llamapun::data::{Corpus,Document};

fn main() {
  let start_example = PreciseTime::now();
  let mut dictionary: HashMap<String, i64> = HashMap::new();
  let mut word_frequencies: HashMap<String, i64> = HashMap::new();
  let mut frequencies: HashMap<i64, i64> = HashMap::new();
  let mut word_index = 0;
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
  let mut document = Document::new("tests/resources/".to_string()+arxivid+".html", &corpus).unwrap();
  let end_parse = PreciseTime::now();

  // We will tokenize each logical paragraph, which are the textual logical units in an article
  for mut paragraph in document.iter() {
    total_paragraphs += 1;
    for mut sentence in paragraph.iter() {
      total_sentences += 1;
      for sent_word in sentence.simple_iter() {
        total_words += 1;
        let word = sent_word.range.get_plaintext().to_string().to_lowercase();
        let dictionary_index : &i64 = 
          match dictionary.contains_key(&word) {
          true => dictionary.get(&word).unwrap(),
          false => {
            word_index+=1;
            dictionary.insert(word.clone(), word_index);
            &word_index }
          };

        let word_frequency = frequencies.entry(*dictionary_index).or_insert(0);
        *word_frequency += 1;
        word_frequencies.insert(word.clone(), word_frequency.clone());
      }
    }
  }
  let end_example = PreciseTime::now();
  
  let mut sorted_dictionary = Vec::new();
  for (word, index) in dictionary.iter() {
    sorted_dictionary.push((word,index));
  }
  sorted_dictionary.sort_by(|a,b| a.1.cmp(b.1));
  println!("--- Dictionary: \n{:?}\n\n", sorted_dictionary);
  
  // Unsorted gnuplot of frequencies:

  let freq_keys = frequencies.clone().into_iter().map(|entry| entry.0);
  let log_freq_values = frequencies.clone().into_iter().map(|entry| (entry.1.clone() as f64).log2());
  let mut fg = Figure::new();
  fg.axes2d()
  .points(freq_keys, log_freq_values, &[PointSymbol('O'), Color("#ffaa77"), PointSize(1.2)])
  .set_x_label("Word index, in order of document occurrence", &[Rotate(45.0)])
  .set_y_label("Frequency counts (log2)", &[Rotate(90.0)])
  .set_title(&("Word Frequencies (arXiv ".to_string()+arxivid+")"), &[]);

  fg.set_terminal("pngcairo", "word_frequencies_inorder.png");
  fg.show();
  
  // Sorted gnuplot of frequency distribution:
  let mut frequency_distribution = HashMap::new();
  // Obtain the distribution from the raw frequency data
  for (_,value) in frequencies.iter() {
    let words_with_frequency = frequency_distribution.entry(value).or_insert(0);
    *words_with_frequency += 1;
  }
  // Perform sort
  let mut value_sorted_frequencies = Vec::new();
  for (index,value) in frequency_distribution.iter() {
    value_sorted_frequencies.push((value.clone(),index.clone())); // ( # Distinct words , Frequency )
  }
  value_sorted_frequencies.sort_by(|a, b| a.1.cmp(b.1));

  let sorted_log_freq_values = value_sorted_frequencies.clone().into_iter().map(|entry| (entry.1.clone() as f64).log2());
  let ordered_indexes = value_sorted_frequencies.clone().into_iter().map(|entry| entry.0.clone());

  fg = Figure::new();
  fg.axes2d()
  .points(ordered_indexes, sorted_log_freq_values, &[PointSymbol('O'), Color("blue"), PointSize(1.2)])
  .set_x_label("Distinct words with this frequency", &[Rotate(45.0)])
  .set_y_label("Frequency (log2)", &[Rotate(90.0)])
  .set_title(&("Frequency Distribution (arXiv ".to_string()+arxivid+")"), &[]);

  fg.set_terminal("pngcairo", "word_frequencies_sorted.png");
  fg.show();

  // Print out data:
  let mut sorted_word_frequencies =  Vec::new();
  for (word,value) in word_frequencies.iter() {
    sorted_word_frequencies.push((word,value));
  }
  sorted_word_frequencies.sort_by(|a, b| a.1.cmp(b.1));

  println!("--- Frequencies: \n{:?}\n\n", sorted_word_frequencies);
  println!("--- Frequency distribution: \n{:?}\n\n", value_sorted_frequencies);
  println!("--- Paragraphs total: {:?}",total_paragraphs);
  println!("--- Sentences total: {:?}",total_sentences);
  println!("--- Words total: {:?}",total_words);
  println!("--- Words distinct: {:?}",word_index);
  println!("");
  let end_reports = PreciseTime::now();
  println!("--- Benchmark report:");
  println!("    LibXML parse took {:?}ms",start_example.to(end_parse).num_milliseconds());
  println!("    LLaMaPun word tokenization took {:?}ms",end_parse.to(end_example).num_milliseconds());
  println!("    Finished report generation in {:?}ms",end_example.to(end_reports).num_milliseconds());
  println!("    Total time: {:?}ms", start_example.to(end_reports).num_milliseconds());
}

