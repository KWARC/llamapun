//! Data structures and Iterators for convenient high-level syntax
use std::vec::IntoIter;
use std::cell::{RefCell, Cell};
use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use walkdir::Result as DirResult;

use dnm::{DNM, DNMRange, DNMParameters};
use tokenizer::Tokenizer;

use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;
use libxml::xpath::Context;
use libxml::parser::{Parser, XmlParseError};

use rustsenna::sennapath::SENNA_PATH;
use rustsenna::senna::{Senna, SennaParseOptions};
use rustsenna::pos::POS;
use rustsenna::sentence::Sentence as SennaSentence;

pub struct Corpus {
  // Directory-level
  pub path : String,
  // Document-level
  pub parser : Parser,
  pub tokenizer : Tokenizer,
  pub senna : RefCell<Senna>,
  pub senna_options : Cell<SennaParseOptions>,
}

pub struct DocumentIterator<'iter>{
  walker : Box<WalkDirIterator<Item=DirResult<DirEntry>>>,
  pub corpus : &'iter Corpus,
}

pub struct Document<'d> {
  pub dom : XmlDoc,
  pub path : String,
  pub corpus : &'d Corpus,
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
  // pub paragraph : &'iter Paragraph<'iter>
  pub document : &'iter Document<'iter>,
}

pub struct Sentence<'s> {
  pub range : DNMRange<'s>,
  // pub paragraph : &'s Paragraph<'s>
  pub document : &'s Document<'s>,
  pub senna_sentence : Option<SennaSentence<'s>>,
}

pub struct SimpleWordIterator<'iter> {
  walker : IntoIter<DNMRange<'iter>>,
  pub sentence : &'iter Sentence<'iter>
}

pub struct SennaWordIterator<'iter> {
  // walker : IntoIter<SennaWord<'iter>>,
  pos : usize,
  pub sentence : &'iter Sentence<'iter>
}

pub struct Word<'w> {
  pub range : DNMRange<'w>,// &'w str, // should we use the DNMRange instead???
  pub sentence : &'w Sentence<'w>,
  pub pos : POS,
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
      senna : RefCell::new(Senna::new(SENNA_PATH.to_owned())),
      senna_options : Cell::new(SennaParseOptions::default()),
    }
  }

  pub fn iter(& mut self) -> DocumentIterator {
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
      document : self.document,
    }
  }
}

impl<'iter> Iterator for SentenceIterator<'iter> {
  type Item = Sentence<'iter>;
  fn next(&mut self) -> Option<Sentence<'iter>> {
    match self.walker.next() {
      None => None,
      Some(range) => {
        let sentence = Sentence { range: range, document: self.document, senna_sentence : None };
        Some(sentence)
      }
    }
  }
}

impl<'s> Sentence<'s> {
  pub fn simple_iter(&'s mut self) -> SimpleWordIterator<'s> {
    let tokenizer = &self.document.corpus.tokenizer;
    let words = tokenizer.words(&self.range);
    SimpleWordIterator {
      walker : words.into_iter(),
      sentence : self
    }
  }

  pub fn senna_iter(&'s mut self) -> SennaWordIterator<'s> {
    SennaWordIterator {
      pos : 0usize,
      sentence : if self.senna_sentence.is_none() {self.senna_parse()} else { self },
    }
  }

  pub fn senna_parse(&'s mut self) -> &Self {
    self.senna_sentence = Some(self.document.corpus.senna.borrow_mut().parse((&self.range).get_plaintext(),
                                          self.document.corpus.senna_options.get()));
    self
  }
}

impl<'iter> Iterator for SimpleWordIterator<'iter> {
  type Item = Word<'iter>;
  fn next(&mut self) -> Option<Word<'iter>> {
    match self.walker.next() {
      None => None,
      Some(range) => {
        Some(Word {range : range, sentence : self.sentence, pos : POS::NOT_SET})
      }
    }
  }
}

impl<'iter> Iterator for SennaWordIterator<'iter> {
  type Item = Word<'iter>;
  fn next(&mut self) -> Option<Word<'iter>> {
    // match self.walker.next() {
    let pos = self.pos;
    self.pos += 1;
    let sent = &self.sentence;
    let sen_sent_wrapped = &sent.senna_sentence;
    let sen_sent = sen_sent_wrapped.as_ref().unwrap();
    if pos < sen_sent.get_words().len() {
      let senna_word = &sen_sent.get_words()[pos];
      let range = self.sentence.range.get_subrange(senna_word.get_offset_start(), senna_word.get_offset_end());
      Some(Word { range : range, sentence : self.sentence, pos : senna_word.get_pos() } )
    } else {
      None
    }
  }
}

