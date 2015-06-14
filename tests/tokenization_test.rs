//! Tests for the LLaMaPUn tokenizer

extern crate llamapun;
extern crate rustlibxml;
extern crate libc;

use libc::c_void;
use llamapun::dnmlib::*;
use llamapun::tokenizer::*;
use rustlibxml::tree::*;
use std::collections::HashMap;

#[test]
/// Test sentence tokenization of a simple document
fn test_sentence_tokenization_simple() {
  let simple_text = "This note was written to clarify for myself and my colleagues certain properties \
   of Bernstein approximations that are useful in investigating copulas. We derive some of the basic properties \
   of the Bernstein approximation for functions of n variables and then show that the Bernstein approximation of \
   a copula is again a copula. Unorthodox beginnings of sentences can also occur. Deciphering Eqn. 1 is sometimes. difficult Prof. Automation, isn't it? \
   Our most significant result is a stochastic interpretation of the Bernstein \
   approximation of a copula. This interpretation was communicated to us by J. H. B. Kemperman in [2] for \
   2-copulas and we are not aware of its publication elsewhere.".to_string();
  let fake_ptr = 0 as *mut libc::c_void;
  let simple_dnm = DNM {
    plaintext : simple_text,
    parameters : DNMParameters::default(),
    root_node : XmlNodeRef {node_ptr : fake_ptr, node_is_inserted : true},
    node_map : HashMap::new()};

  let simple_tokenizer = Tokenizer::default();
  let ranges : Vec<DNMRange> = simple_tokenizer.sentences(&simple_dnm).unwrap();
  // // Debug:
  // for r in ranges.iter() {
  //   println!("Sentence: \n{}\n",r.get_plaintext().trim());
  // }
  assert_eq!(ranges.len(), 6);
}