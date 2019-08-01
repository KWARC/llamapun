use libxml::parser::XmlParseError;
use libxml::readonly::RoNode;
use libxml::tree::Document as XmlDoc;
use libxml::xpath::Context;

use super::corpus::Corpus;
use super::{DNMRangeIterator, RoNodeIterator};
use crate::dnm::DNM;

/// One of our math documents, thread-friendly
pub struct Document<'d> {
  /// The DOM of the document
  pub dom: XmlDoc,
  /// The file path of the document
  pub path: String,
  /// A reference to the corpus containing this document
  pub corpus: &'d Corpus,
  /// If it exists, the DNM corresponding to this document
  pub dnm: Option<DNM>,
}

impl<'d> Document<'d> {
  /// Load a new document
  pub fn new(filepath: String, corpus: &'d Corpus) -> Result<Self, XmlParseError> {
    let dom = if filepath.ends_with(".xhtml") {
      corpus.xml_parser.parse_file(&filepath)?
    } else {
      corpus.html_parser.parse_file(&filepath)?
    };

    Ok(Document {
      path: filepath,
      dom,
      corpus,
      dnm: None,
    })
  }

  /// Obtain the problem-free logical headings of a libxml `Document`
  pub fn get_heading_nodes(&self) -> Vec<RoNode> {
    Document::heading_nodes(&self.dom)
  }
  /// Associated function for `get_paragraph_nodes`
  fn heading_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate(
      "//*[contains(@class,'ltx_title') and (local-name()='h2' or local-name()='h3' or local-name()='h4' or local-name()='h5' or local-name()='h6') and not(descendant::*[contains(@class,'ltx_ERROR')]) and not(preceding-sibling::*[contains(@class,'ltx_ERROR')])]",
    ) {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }
  /// Get an iterator over the headings of the document
  pub fn heading_iter(&self) -> RoNodeIterator {
    RoNodeIterator {
      walker: Document::heading_nodes(&self.dom).into_iter(),
      document: self,
    }
  }

  /// Obtain the problem-free logical paragraphs of a libxml `Document`
  pub fn get_paragraph_nodes(&self) -> Vec<RoNode> {
    Document::paragraph_nodes(&self.dom)
  }

  /// Associated function for `get_paragraph_nodes`
  fn paragraph_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate(
      "//*[local-name()='div' and contains(@class,'ltx_para') and not(descendant::*[contains(@class,'ltx_ERROR')]) and not(preceding-sibling::*[contains(@class,'ltx_ERROR')])]",
    ) {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }

  /// Get an iterator over the paragraphs of the document
  pub fn paragraph_iter(&self) -> RoNodeIterator {
    RoNodeIterator {
      walker: Document::paragraph_nodes(&self.dom).into_iter(),
      document: self,
    }
  }

  fn abstract_p_node(doc: &XmlDoc) -> Option<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate(
      "//*[local-name()='div' and contains(@class,'ltx_abstract') and not(descendant::*[contains(@class,'ltx_ERROR')])]/p[1]",
    ) {
      Ok(found_payload) => {
        let mut abs = found_payload.get_readonly_nodes_as_vec();
        if !abs.is_empty() {
          Some(abs.remove(0))
        } else {
          None
        }
      },
      _ => None,
    }
  }

  /// Get an iterator over the paragraphs of the document, AND notable additional paragraphs, such as abstracts
  pub fn extended_paragraph_iter(&self) -> RoNodeIterator {
    let mut paras = Document::paragraph_nodes(&self.dom);
    if let Some(anode) = Document::abstract_p_node(&self.dom) {
      paras.push(anode);
    }
    RoNodeIterator {
      walker: paras.into_iter(),
      document: self,
    }
  }

  /// Obtain the MathML <math> nodes of a libxml `Document`
  pub fn get_math_nodes(&self) -> Vec<RoNode> {
    Document::math_nodes(&self.dom)
  }

  /// Associated function for `get_math_nodes`
  fn math_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate("//*[local-name()='math']") {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }
  /// Obtain the <span[class=ltx_ref]> nodes of a libxml `Document`
  pub fn get_ref_nodes(&self) -> Vec<RoNode> {
    Document::ref_nodes(&self.dom)
  }
  /// Associated function for `get_ref_nodes`
  fn ref_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate("//*[(local-name()='span' or local-name()='a') and (contains(@class,'ltx_ref ') or @class='ltx_ref')]") {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }

  /// Get an iterator over the sentences of the document
  pub fn sentence_iter(&mut self) -> DNMRangeIterator {
    if self.dnm.is_none() {
      if let Some(root) = self.dom.get_root_readonly() {
        self.dnm = Some(DNM::new(root, self.corpus.dnm_parameters.clone()));
      }
    }
    let tokenizer = &self.corpus.tokenizer;
    let sentences = tokenizer.sentences(self.dnm.as_ref().unwrap());
    DNMRangeIterator {
      walker: sentences.into_iter(),
      document: self,
    }
  }
}
