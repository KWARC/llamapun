//! Data structures and Iterators for convenient high-level syntax
use std::vec::IntoIter;
use walkdir::{Error, DirEntry, WalkDir, WalkDirIterator};
use walkdir::Result as DirResult;

use dnm::{DNM, DNMRange, DNMParameters};
use tokenizer::Tokenizer;

use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;
use libxml::xpath::Context;
use libxml::parser::{Parser, XmlParseError};

pub struct Corpus {
  // Directory-level
  path : String,
  // Document-level
  parser : Parser,
  tokenizer : Tokenizer,
}

pub struct DocumentIterator<'iter>{
  walker : Box<WalkDirIterator<Item=DirResult<DirEntry>>>,
  corpus : &'iter Corpus,
}

pub struct Document<'d> {
  dom : XmlDoc,
  tokenizer : &'d Tokenizer
}

pub struct ParagraphIterator<'iter> {
  walker : IntoIter<Node>,
  document : &'iter Document<'iter>
}

pub struct Paragraph {
  dnm : DNM
}

pub struct SentenceIterator<'iter> {
  paragraph : &'iter Paragraph
}

pub struct Sentence<'s> {
  range : DNMRange<'s>,
}

pub struct WordIterator<'iter> {
  sentence : &'iter Sentence<'iter>
}

pub struct Word<'w> {
  range : DNMRange<'w>,
}

// TODO: May be worth refactoring into several layers of iterators - directory, document, paragraph, sentence, etc. 
impl<'iter> Iterator for DocumentIterator<'iter> {
  type Item = Document<'iter>;
  fn next(&mut self) -> Option<Document<'iter>> {
    let mut walker = &mut self.walker;
    loop {
      let next_entry = walker.next();
      if next_entry.is_none() {
        break;
      } else {
        let next_entry_result = next_entry.unwrap();
        if next_entry_result.is_err() {
          continue;
        } else {
          let entry = next_entry_result.unwrap(); // unwrap the walkdir::Result
          let file_name = entry.file_name().to_str().unwrap_or("").to_owned();
          if !file_name.ends_with(".html") {
            continue;
          }
          println!("Next entry: {:?}", entry);
          let doc_result = Document::new(file_name, self.corpus);
          return match doc_result {
            Ok(doc) => Some(doc),
            _ => None
          }
        }
      }
    }
    return None
  }
}

impl Corpus {
  pub fn new(dirpath : String) -> Self {
    Corpus {
      path : dirpath,
      tokenizer : Tokenizer::default(),
      parser : Parser::default_html(),
    }
  }

  pub fn iter(&mut self) -> DocumentIterator {
    DocumentIterator {
      walker : Box::new(WalkDir::new(self.path.clone()).into_iter()),
      corpus : self
    }
  }
}

impl<'d> Document<'d> {
  pub fn new(filepath: String, owner: &'d Corpus) -> Result<Self, XmlParseError> {
    let dom = try!(owner.parser.parse_file(&filepath));

    Ok(Document {
      dom : dom,
      tokenizer : &owner.tokenizer
    })
  }

  pub fn iter(&mut self) -> ParagraphIterator {
    let xpath_context = Context::new(&self.dom).unwrap();
    let paras = match xpath_context.evaluate("//*[contains(@class,'ltx_para')]") {
      Ok(xpath_result) => xpath_result.get_nodes_as_vec(),
      _ => Vec::new()
    };
    ParagraphIterator {
      walker : paras.into_iter(),
      document : self
    }
  }
}


impl<'iter> Iterator for ParagraphIterator<'iter> {
  type Item = Paragraph;
  fn next(&mut self) -> Option<Paragraph> {
    match self.walker.next() {
      None => None,
      Some(node) => {
        // Create a DNM for the current paragraph
        let dnm = DNM::new(node, DNMParameters::llamapun_normalization());    
        Some(Paragraph {dnm : dnm})
      }
    }
  }
}
  
  //   let ranges : Vec<DNMRange> = tokenizer.sentences(&dnm);

  //   for sentence_range in ranges {
  //     total_sentences += 1;
  //     for w in tokenizer.words(&sentence_range) {
  //       total_words += 1;
  //       let word = w.to_string().to_lowercase();
  //       let dictionary_index : &i64 = 
  //         match dictionary.contains_key(&word) {
  //         true => dictionary.get(&word).unwrap(),
  //         false => {
  //           word_index+=1;
  //           dictionary.insert(word.clone(), word_index);
  //           &word_index }
  //         };
  //       // print!("{}  ",dictionary_index);
  //       let word_frequency = frequencies.entry(*dictionary_index).or_insert(0);
  //       *word_frequency += 1;
  //       word_frequencies.insert(word.clone(), word_frequency.clone());
  //     }
  //     // println!("");
  //   }
  // }