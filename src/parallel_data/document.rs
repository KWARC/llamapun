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
  /// Associated function for `get_heading_nodes`
  fn heading_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    Document::xpath_nodes(doc,
      "//*[contains(@class,'ltx_title') and (local-name()='h2' or local-name()='h3' or local-name()='h4' or local-name()='h5' or local-name()='h6') and not(descendant::*[contains(@class,'ltx_ERROR')]) and not(preceding-sibling::*[contains(@class,'ltx_ERROR')])]",
    )
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
    Document::xpath_nodes(doc,
      "//*[local-name()='div' and contains(@class,'ltx_para') and not(descendant::*[contains(@class,'ltx_ERROR')]) and not(preceding-sibling::*[contains(@class,'ltx_ERROR')])]",
    )
  }

  /// Get an iterator over the paragraphs of the document
  pub fn paragraph_iter(&self) -> RoNodeIterator {
    RoNodeIterator {
      walker: Document::paragraph_nodes(&self.dom).into_iter(),
      document: self,
    }
  }

  fn abstract_p_node(doc: &XmlDoc) -> Option<RoNode> {
    Document::xpath_node(doc,
      "//*[local-name()='div' and contains(@class,'ltx_abstract') and not(descendant::*[contains(@class,'ltx_ERROR')])]/p[1]")
  }

  /// This is for special latexml markup for a <div class='ltx_acknowledgement'>text content</div>
  /// which remains undetected by the regular paragraph selectors
  fn acknowledgement_node(doc: &XmlDoc) -> Option<RoNode> {
    Document::xpath_node(doc,
      "//*[local-name()='div' and contains(@class,'ltx_acknowledgement') and not(descendant::*[contains(@class,'ltx_ERROR')])]/text()")
  }

  /// Get an iterator over the paragraphs of the document,
  /// AND notable additional paragraphs, such as abstracts
  pub fn extended_paragraph_iter(&self) -> RoNodeIterator {
    let mut paras = Vec::new();
    if let Some(anode) = Document::abstract_p_node(&self.dom) {
      paras.push(anode);
    }
    paras.extend(Document::paragraph_nodes(&self.dom));
    if let Some(anode) = Document::acknowledgement_node(&self.dom) {
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
  pub(crate) fn math_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    Document::xpath_nodes(&doc, "//*[local-name()='math']")
  }
  /// Obtain the <span[class=ltx_ref]> nodes of a libxml `Document`
  pub fn get_ref_nodes(&self) -> Vec<RoNode> {
    Document::ref_nodes(&self.dom)
  }
  /// Associated function for `get_ref_nodes`
  pub(crate) fn ref_nodes(doc: &XmlDoc) -> Vec<RoNode> {
    Document::xpath_nodes(&doc,
    "//*[(local-name()='span' or local-name()='a') and (contains(@class,'ltx_ref ') or @class='ltx_ref')]")
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

  /// Obtain the nodes associated with the xpath evaluation over the underlying libxml `Document`
  pub fn get_xpath_nodes(&self, xpath_str: &str) -> Vec<RoNode> {
    Document::xpath_nodes(&self.dom, xpath_str)
  }

  /// Associated function for `get_xpath_nodes`
  pub(crate) fn xpath_nodes(doc: &XmlDoc, xpath_str: &str) -> Vec<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate(xpath_str) {
      Ok(found_payload) => found_payload.get_readonly_nodes_as_vec(),
      _ => Vec::new(),
    }
  }

  /// Obtain the first node associated with the xpath evaluation over the underlying libxml `Document`
  pub fn get_xpath_node(&self, xpath_str: &str) -> Option<RoNode> {
    Document::xpath_node(&self.dom, xpath_str)
  }

  /// Associated function for `get_xpath_nodes`
  pub(crate) fn xpath_node(doc: &XmlDoc, xpath_str: &str) -> Option<RoNode> {
    let xpath_context = Context::new(doc).unwrap();
    match xpath_context.evaluate(xpath_str) {
      Ok(found_payload) => {
        let mut vec_nodes = found_payload.get_readonly_nodes_as_vec();
        if !vec_nodes.is_empty() {
          Some(vec_nodes.remove(0))
        } else {
          None
        }
      }
      _ => None,
    }
  }

  /// Get an iterator over a custom xpath selector over the document
  pub fn xpath_selector_iter(&self, xpath_str: &str) -> RoNodeIterator {
    RoNodeIterator {
      walker: Document::xpath_nodes(&self.dom, xpath_str).into_iter(),
      document: self,
    }
  }

  /// Associated function for `get_filtered_nodes`
  pub(crate) fn dfs_filter_nodes(node: RoNode, filter: &dyn Fn(&RoNode) -> bool) -> Vec<RoNode> {
    let mut found = Vec::new();
    if filter(&node) {
      found.push(node);
    }
    for child in node.get_child_nodes().into_iter() {
      found.extend(Document::dfs_filter_nodes(child, filter));
    }
    found
  }

  /// Get an iterator using a custom closure predicate filter over the document (depth-first descent)
  pub fn filter_iter(&self, filter: &dyn Fn(&RoNode) -> bool) -> RoNodeIterator {
    // TODO: Can this be lazy? Eager for now...
    RoNodeIterator {
      walker: Document::dfs_filter_nodes(self.dom.get_root_readonly().unwrap(), filter).into_iter(),
      document: self,
    }
  }
}
