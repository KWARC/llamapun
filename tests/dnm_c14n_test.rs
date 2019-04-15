//! Tests for the DNM canonicalization module
extern crate libxml;
extern crate llamapun;

use libxml::parser::Parser;
use libxml::xpath::Context;
use llamapun::dnm::*;
use std::collections::HashSet;

#[test]
fn test_c14n_basic() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file01.xml").unwrap();

  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(root, DNMParameters::default());
  let c14n = dnm.to_c14n_basic();
  assert!(!c14n.is_empty());
}

#[test]
fn test_c14n_basic_full() {
  let parser = Parser::default();
  let doc = parser
    .parse_file("tests/resources/1311.0066.xhtml")
    .unwrap();

  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(root, DNMParameters::default());
  let c14n = dnm.to_c14n_basic();
  assert!(!c14n.is_empty());
}

#[test]
fn test_c14n_math_hash() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/0903.1000.html").unwrap();

  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(root, DNMParameters::default());

  let xpath_context = Context::new(&doc).unwrap();
  let formulas = match xpath_context.evaluate("//*[contains(@class,'ltx_Math')]") {
    Ok(xpath_result) => xpath_result.get_readonly_nodes_as_vec(),
    _ => Vec::new(),
  };

  let mut formula_c14ns = HashSet::new();
  let mut formula_hashes = HashSet::new();

  for formula in formulas {
    let canonical_formula = dnm.node_c14n_basic(formula);
    assert!(!canonical_formula.is_empty());
    // println!("Formula: \n{}", canonical_formula);
    // insert formula in hash
    formula_c14ns.insert(canonical_formula);
    // compute md5 hash of formula, and insert it in hash
    let formula_hash = dnm.node_hash_basic(formula);
    // println!("MD5: {}", formula_hash);
    formula_hashes.insert(formula_hash);
  }
  assert_eq!(formula_c14ns.len(), formula_hashes.len());
}
