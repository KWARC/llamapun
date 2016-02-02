//! Data structures and Iterators for convenient high-level syntax
use std::vec::IntoIter;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use walkdir::Result as DirResult;

use dnm::{DNM, DNMRange, DNMParameters};
use tokenizer::Tokenizer;

use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;
use libxml::xpath::Context;
use libxml::parser::{Parser, XmlParseError};

pub struct Corpus {
  // Directory-level
  pub path : String,
  // Document-level
  pub parser : Parser,
  pub tokenizer : Tokenizer,
}

pub struct DocumentIterator<'iter>{
  walker : Box<WalkDirIterator<Item=DirResult<DirEntry>>>,
  pub corpus : &'iter Corpus,
}

pub struct Document<'d> {
  pub dom : XmlDoc,
  pub path : String,
  pub corpus : &'d Corpus
}

pub struct ParagraphIterator<'iter> {
  walker : IntoIter<Node>,
  pub document : &'iter Document<'iter>
}

pub struct Paragraph<'p> {
  pub dnm : DNM,
  pub document : &'p Document<'p>
}

pub struct SentenceIterator<'iter> {
  walker : IntoIter<DNMRange<'iter>>,
  pub paragraph : &'iter Paragraph<'iter>
}

pub struct Sentence<'s> {
  pub range : DNMRange<'s>,
  pub paragraph : &'s Paragraph<'s>
}

pub struct WordIterator<'iter> {
  walker : IntoIter<&'iter str>,
  pub sentence : &'iter Sentence<'iter>
}

pub struct Word<'w> {
  pub text : &'w str, // should we use the DNMRange instead???
  pub sentence : &'w Sentence<'w>
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
          let path = entry.path().to_str().unwrap_or("").to_owned();
          let doc_result = Document::new(path, self.corpus);
          return match doc_result {
            Ok(doc) => {
              Some(doc)
            },
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
      path : filepath,
      dom : dom,
      corpus : owner
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
  type Item = Paragraph<'iter>;
  fn next(&mut self) -> Option<Paragraph<'iter>> {
    match self.walker.next() {
      None => None,
      Some(node) => {
        // Create a DNM for the current paragraph
        let dnm = DNM::new(node, DNMParameters::llamapun_normalization());    
        Some(Paragraph {dnm : dnm, document : self.document})
      }
    }
  }
}

impl<'p> Paragraph<'p> {
  pub fn iter(&'p mut self) -> SentenceIterator<'p> {
    let tokenizer = &self.document.corpus.tokenizer;
    let sentences = tokenizer.sentences(&self.dnm);
    SentenceIterator {
      walker : sentences.into_iter(),
      paragraph : self
    }
  }
}

impl<'iter> Iterator for SentenceIterator<'iter> {
  type Item = Sentence<'iter>;
  fn next(&mut self) -> Option<Sentence<'iter>> {
    match self.walker.next() {
      None => None,
      Some(range) => {
        Some(Sentence {range : range, paragraph : self.paragraph})
      }
    }
  }
}

impl<'s> Sentence<'s> {
  pub fn iter(&'s mut self) -> WordIterator<'s> {
    let tokenizer = &self.paragraph.document.corpus.tokenizer;
    let words = tokenizer.words(&self.range);
    WordIterator {
      walker : words.into_iter(),
      sentence : self
    }
  }
}

impl<'iter> Iterator for WordIterator<'iter> {
  type Item = Word<'iter>;
  fn next(&mut self) -> Option<Word<'iter>> {
    match self.walker.next() {
      None => None,
      Some(text) => {
        Some(Word {text : text, sentence : self.sentence})
      }
    }
  }
}
