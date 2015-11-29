extern crate llamapun;
extern crate libxml;
extern crate libc;
extern crate gnuplot;
extern crate time;

use std::collections::HashMap;
use time::PreciseTime;
use libxml::xpath::*;
use libxml::parser::Parser;
use gnuplot::*;

use llamapun::dnm::*;
use llamapun::tokenizer::*;

fn main() {
  let start_example = PreciseTime::now();
  let parser = Parser::default_html();
  let arxivid = "0903.1000";
  let doc = parser.parse_file(&("tests/resources/".to_string()+arxivid+".html")).unwrap();
  let end_parse = PreciseTime::now();
  let mut dictionary: HashMap<String, i64> = HashMap::new();
  let mut word_frequencies: HashMap<String, i64> = HashMap::new();
  let mut frequencies: HashMap<i64, i64> = HashMap::new();
  let mut word_index = 0;

  // We will tokenize each logical paragraph, which are the textual logical units in an article
  let xpath_context = Context::new(&doc).unwrap();
  let para_xpath_result = xpath_context.evaluate("//*[contains(@class,'ltx_para')]").unwrap();

  for para in para_xpath_result.get_nodes_as_vec().iter() {
    let dnm = DNM::new(&para, DNMParameters::llamapun_normalization());

    let tokenizer = Tokenizer::default();
    let ranges : Vec<DNMRange> = tokenizer.sentences(&dnm).unwrap();

    for range in ranges {
      let sentence = range.get_plaintext();
      for w in sentence.split(|c: char| !c.is_alphabetic()) {
        if w.len() == 0 {
          continue;
        }
        let word = w.to_string().to_lowercase();
        let dictionary_index : &i64 = 
          match dictionary.contains_key(&word) {
          true => dictionary.get(&word).unwrap(),
          false => {
            word_index+=1;
            dictionary.insert(word.clone(), word_index);
            &word_index }
          };
        // print!("{}  ",dictionary_index);
        let word_frequency = frequencies.entry(*dictionary_index).or_insert(0);
        *word_frequency += 1;
        word_frequencies.insert(word.clone(), word_frequency.clone());
      }
      // println!("");
    }
  }
  // println!("");
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
  .set_x_label("Words, in order of appearance", &[Rotate(45.0)])
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
  let end_reports = PreciseTime::now();
  println!("--- Benchmark report:");
  println!("    LibXML parse took {:?}ms",start_example.to(end_parse).num_milliseconds());
  println!("    LLaMaPun word tokenization took {:?}ms",end_parse.to(end_example).num_milliseconds());
  println!("    Finished report generation in {:?}ms",end_example.to(end_reports).num_milliseconds());
  println!("    Total time: {:?}ms", start_example.to(end_reports).num_milliseconds());
}

