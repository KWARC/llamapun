//! Data structures and Iterators for convenient high-level syntax

use std::io;
use std::io::Write;
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

use senna::sennapath::SENNA_PATH;
use senna::senna::{Senna, SennaParseOptions};
use senna::pos::POS;
use senna::sentence::Sentence as SennaSentence;
use senna::sentence::Word as SennaWord;
use senna::util::parse_psg;

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
  pub dnm : Option<DNM>,
  pub xpath_context: Context,
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

pub struct AnnotatedSentenceIterator<'iter> {
  pos: usize,
  sentences: Vec<Node>,
  pub document : &'iter Document<'iter>,
}

pub struct Sentence<'s> {
  pub range : DNMRange<'s>,
  // pub paragraph : &'s Paragraph<'s>
  pub document : &'s Document<'s>,
  pub senna_sentence : Option<SennaSentence<'s>>,
  pub node : Option<Node>,
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

  pub fn set_senna_options(&mut self, options: SennaParseOptions) {
      self.senna_options.set(options);
  }

  pub fn iter(& mut self) -> DocumentIterator {
    DocumentIterator {
      walker : Box::new(WalkDir::new(self.path.clone()).into_iter()),
      corpus : self
    }
  }

  pub fn load_doc(&self, path : String) -> Result<Document, XmlParseError> {
    Document::new(path, self)
  }
}

impl<'d> Document<'d> {
  pub fn new(filepath: String, owner: &'d Corpus) -> Result<Self, XmlParseError> {
    let dom = try!(owner.parser.parse_file(&filepath));
    let xpc = Context::new(&dom).unwrap();

    Ok(Document {
      path : filepath,
      dom : dom,
      corpus : owner,
      dnm : None,
      xpath_context: xpc,
    })
  }

  pub fn paragraph_iter(&mut self) -> ParagraphIterator {
    //let xpath_context = Context::new(&self.dom).unwrap();
    let paras = match self.xpath_context.evaluate("//*[contains(@class,'ltx_para')]") {
      Ok(xpath_result) => xpath_result.get_nodes_as_vec(),
      _ => Vec::new()
    };
    ParagraphIterator {
      walker : paras.into_iter(),
      document : self
    }
  }

  pub fn sentence_iter(&mut self) -> SentenceIterator {
    if self.dnm.is_none() {
      self.dnm = Some(DNM::new(self.dom.get_root_element().unwrap(), DNMParameters::llamapun_normalization()));
    }
    let tokenizer = &self.corpus.tokenizer;
    let sentences = tokenizer.sentences(self.dnm.as_ref().unwrap());
    SentenceIterator {
      walker : sentences.into_iter(),
      document: self,
    }
  }

  pub fn annotated_sentence_iter(&mut self) -> AnnotatedSentenceIterator {
    if self.dnm.is_none() {
      self.dnm = Some(DNM::new(self.dom.get_root_element().unwrap(), DNMParameters::llamapun_normalization()));
    }
    let sentences = self.xpath_context.evaluate("//*[contains(@class,'ltx_para')]//span[contains(@class,'sentence')]")
                                 .expect("Could not evalute sentence XPATH");
    AnnotatedSentenceIterator {
      pos: 0,
      sentences: sentences.get_nodes_as_vec(),
      document: self,
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
        if range.is_empty() {
          self.next()
        } else {
          let sentence = Sentence { range: range, document: self.document, senna_sentence : None, node: None };
          Some(sentence)
        }
      }
    }
  }
}


impl<'iter> Iterator for AnnotatedSentenceIterator<'iter> {
  type Item = Sentence<'iter>;
  fn next(&mut self) -> Option<Sentence<'iter>> {
    if self.sentences.len() > self.pos {
      self.pos = self.pos + 1;
      if self.sentences.len() > self.pos {
        let sentence_node = &self.sentences[self.pos];
        let sentence_id = sentence_node.get_property("id").expect("Sentence doesn't have id");
        let word_nodes = match self.document.xpath_context.evaluate(
                                    &format!("//span[@id='{}']//span[contains(@class,'word')]", sentence_id)) {
            Ok(result) => result.get_nodes_as_vec(),
            Err(_) => {
                writeln!(io::stderr(), "Warning: Found sentence without words (@id='{}')", sentence_id).unwrap();
                vec![]
            }
        };
        // get words of sentences in raw form
        let mut words_raw : Vec<(DNMRange, POS)> = Vec::new();
        for word_node in word_nodes {
            let pos_tag_string = word_node.get_property("pos").expect("Word doesn't have pos tag");
            let pos_tag_str : &str = &pos_tag_string;
            let pos_map = &self.document.corpus.senna.borrow().pos_map;
            let pos_tag = pos_map.get(pos_tag_str)
                                 .expect(&format!("Unknown pos tag: \"{}\"", pos_tag_str));
            let dnmrange = self.document.dnm.as_ref().unwrap().get_range_of_node(&word_node).unwrap();
            words_raw.push((dnmrange, *pos_tag));
        }
        // sort the words
        words_raw.sort_by(|&(ref a,_), &(ref b,_)| a.start.cmp(&b.start));
        let sentence_range = self.document.dnm.as_ref().unwrap().get_range_of_node(&sentence_node).unwrap();
        let mut ssent = SennaSentence::new(sentence_range.get_plaintext());
        ssent.set_psgstring(sentence_node.get_property("psg").expect("Sentence doesn't have psg"));
        let psg_map = &self.document.corpus.senna.borrow().psg_map;
        let psgroot = parse_psg(ssent.get_psgstring().unwrap().as_bytes(),
                                             &mut 0, &mut 0, psg_map);
        ssent.set_psgroot(psgroot);
        for i in 0..words_raw.len() {
            let mut word = SennaWord::new(words_raw[i].0.start - sentence_range.start,
                                     words_raw[i].0.end - sentence_range.start,
                                     words_raw[i].0.get_plaintext(), i as u32);
            word.set_pos(words_raw[i].1);
            ssent.push_word(word);
        }
        
        Some(Sentence {
            range: sentence_range.clone(),
            document: self.document,
            senna_sentence: Some(ssent),
            node: Some(sentence_node.clone()),
        })
      } else {
        None
      }
    } else {
      None
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
    if self.senna_sentence.is_none() {
        self.senna_sentence = 
            Some(self.document.corpus.senna.borrow_mut().parse((&self.range).get_plaintext(),
                                          self.document.corpus.senna_options.get()));
    }
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

