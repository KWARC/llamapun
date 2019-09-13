//! Tests for the DNM library
use llamapun::data::Document;
use llamapun::dnm::*;
use llamapun::tokenizer::Tokenizer;
use std::fs;

#[test]
fn test_string_to_dnm() {
  let dnm_result = DNM::from_str("Hello world", None);
  assert!(dnm_result.is_ok());
  if let Ok((doc, dnm)) = dnm_result {
    let paras = Document::paragraph_nodes(&doc);
    assert_eq!(
      paras.len(),
      1,
      "one auto-wrapped <div class='ltx_para'> paragraph"
    );
    let simple_tokenizer = Tokenizer::default();
    let ranges: Vec<DNMRange> = simple_tokenizer.sentences(&dnm);
    assert_eq!(ranges.len(), 1, "one sentence in mock example");
  }
}

#[test]
fn real_ams_para_to_dnm() {
  let contents = fs::read_to_string("tests/resources/ams_para_definition.txt")
    .expect("Something went wrong reading the file");
  assert_eq!(
    contents.lines().collect::<Vec<&str>>().len(),
    3,
    "three pre-tokenized sentences in the example"
  );
  assert_eq!(
    contents.len(),
    1263,
    "we hand-counted the chars in this example."
  );
  // File successfully loaded, let's check the DNM wraps it correctly.
  let dnm_result = DNM::from_str(&contents, None);
  assert!(dnm_result.is_ok());
  if let Ok((doc, dnm)) = dnm_result {
    let paras = Document::paragraph_nodes(&doc);
    assert_eq!(
      paras.len(),
      1,
      "one auto-wrapped <div class='ltx_para'> paragraph"
    );
    let simple_tokenizer = Tokenizer::default();
    let ranges: Vec<DNMRange> = simple_tokenizer.sentences(&dnm);
    // dbg!(&ranges);
    assert_eq!(
      ranges.len(),
      1,
      "one sentence if read as a regular text string"
    );
  }

  // However, if we want to reinterpret this as a llamapun-normalized plaintext, we need to rebuild
  // some punctuation and add tags.
  let dnm_ams_result = DNM::from_ams_paragraph_str(&contents, None);
  assert!(dnm_ams_result.is_ok());
  if let Ok((doc, dnm)) = dnm_ams_result {
    let paras = Document::paragraph_nodes(&doc);
    assert_eq!(
      paras.len(),
      1,
      "one auto-wrapped <div class='ltx_para'> paragraph"
    );
    let simple_tokenizer = Tokenizer::default();
    let ranges: Vec<DNMRange> = simple_tokenizer.sentences(&dnm);
    // dbg!(&ranges);
    assert_eq!(
      ranges.len(),
      3,
      "three sentences if read as a pre-normalized llamapun paragraph."
    );
    assert_eq!(
      dnm.plaintext.matches("mathformula").count(),
      13,
      "We expect lexematized mathematics to be compacted to a mathformula token"
    );
  }
}
