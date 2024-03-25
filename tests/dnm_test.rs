//! Tests for the DNM library
extern crate libc;
extern crate libxml;
extern crate llamapun;
extern crate rustmorpha;

use libxml::parser::Parser;
use libxml::xpath::Context;
use llamapun::dnm::*;
use std::collections::HashMap;

#[test]
fn test_plaintext_simple() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file01.xml").unwrap();
  let mut options: HashMap<String, SpecialTagsOption> = HashMap::new();
  options.insert("h1".to_string(), SpecialTagsOption::Enter); //actually default behaviour
  options.insert("h2".to_string(), SpecialTagsOption::Skip);
  options.insert(
    "a".to_string(),
    SpecialTagsOption::Normalize("[link]".to_string()),
  );
  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(
    root,
    DNMParameters {
      special_tag_name_options: options,
      normalize_white_spaces: true,
      ..Default::default()
    },
  );
  assert_eq!(
    dnm.plaintext.trim(),
    "Title Some text [link] and a bit more text."
  );
}

#[test]
fn test_non_normalized_unicode() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file05.xml").unwrap();
  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(
    root,
    DNMParameters {
      normalize_unicode: false,
      ..DNMParameters::default()
    },
  );
  let entire_range = dnm.get_range_of_node(root).unwrap();
  let trimmed = entire_range.trim();
  let unicode = trimmed.get_subrange(0, 7);
  assert_eq!(unicode.get_plaintext(), "Unicöde");
  let unicode2 = trimmed.get_subrange_from_byte_offsets(0, 7);
  assert_eq!(unicode2.get_plaintext(), "Unicöd"); // ö has two bytes in UTF-8
  let privet1 = trimmed.get_subrange(13, 19);
  assert_eq!(privet1.get_plaintext(), "привет");
  let privet2 = trimmed.get_subrange_from_byte_offsets(14, 26);
  assert_eq!(privet2.get_plaintext(), "привет");
  let privet3 = trimmed.get_subrange_from_byte_offsets(14, 25); // last byte of last char is cut off - should still work!
  assert_eq!(privet3.get_plaintext(), "привет");
}

#[test]
fn test_xml_node_to_plaintext() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file01.xml").unwrap();
  let mut options: HashMap<String, SpecialTagsOption> = HashMap::new();
  options.insert("h1".to_string(), SpecialTagsOption::Enter); //actually default behaviour
  options.insert("h2".to_string(), SpecialTagsOption::Skip);
  options.insert(
    "a".to_string(),
    SpecialTagsOption::Normalize("[link]".to_string()),
  );
  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(
    root,
    DNMParameters {
      special_tag_name_options: options,
      ..Default::default()
    },
  );
  let mut node = doc.get_root_readonly().unwrap();
  match node.get_first_child() {
    Some(n) => node = n,
    None => unreachable!(), //DOM generation failed
  }
  while node.get_name() != "body" {
    match node.get_next_sibling() {
      Some(n) => node = n,
      None => unreachable!(),
    }
  }
  node = node.get_first_child().unwrap();
  while node.get_name() != "h1" {
    match node.get_next_sibling() {
      Some(n) => node = n,
      None => unreachable!(),
    }
  }
  //Node content should have been processed
  assert_eq!(
    dnm.get_range_of_node(node).unwrap().get_plaintext(),
    "Title"
  );
  while node.get_name() != "h2" {
    match node.get_next_sibling() {
      Some(n) => node = n,
      None => unreachable!(),
    }
  }
  //node was skipped in dnm generation
  assert_eq!(dnm.get_range_of_node(node).unwrap().get_plaintext(), "");
  while node.get_name() != "a" {
    match node.get_next_sibling() {
      Some(n) => node = n,
      None => unreachable!(),
    }
  }
  //node content should have been replaced by "[link]"
  assert_eq!(
    dnm.get_range_of_node(node).unwrap().get_plaintext().trim(),
    "[link]"
  );
}

#[test]
fn test_back_mapping_simple() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file01.xml").unwrap();
  let root = doc.get_root_readonly().unwrap();
  let mut options: HashMap<String, SpecialTagsOption> = HashMap::new();
  options.insert(
    "a".to_string(),
    SpecialTagsOption::Normalize("[link]".to_string()),
  );
  let dnm = DNM::new(
    root,
    DNMParameters {
      special_tag_name_options: options,
      normalize_white_spaces: true,
      support_back_mapping: true,
      ..Default::default()
    },
  );

  let range = DNMRange {
    start: 32,
    end: 35,
    dnm: &dnm,
  };

  assert_eq!(range.get_plaintext(), "and");

  // test serialization
  let string = range.serialize();
  assert_eq!(
    string,
    "arange(string-index(//body[1]/text()[4],10),string-index(//body[1]/text()[4],13))"
  );

  // test deserialization
  let xpath_context = Context::new(&doc).unwrap();
  let range2 = DNMRange::deserialize(&string, &dnm, &xpath_context);
  assert_eq!(range2.get_plaintext(), "and");

  let range3 = DNMRange {
    start: 26,
    end: 30,
    dnm: &dnm,
  };

  assert_eq!(range3.get_plaintext(), "link");
  let string2 = range3.serialize();
  assert_eq!(string2, "arange(//body[1]/a[1],//body[1]/text()[4])");

  let range4 = DNMRange::deserialize(&string2, &dnm, &xpath_context);

  assert_eq!(range4.get_plaintext(), "[link]");
}

#[test]
fn test_plaintext_normalized_class_names() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file02.xml").unwrap();
  let mut options: HashMap<String, SpecialTagsOption> = HashMap::new();
  options.insert(
    "normalized".to_string(),
    SpecialTagsOption::Normalize("[NORMALIZED]".to_string()),
  );
  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(
    root,
    DNMParameters {
      special_tag_class_options: options,
      normalize_white_spaces: true,
      ..Default::default()
    },
  );
  assert_eq!(dnm.plaintext.trim(), "[NORMALIZED] Else");
}

#[test]
fn test_unicode_normalization() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file03.xml").unwrap();
  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(
    root,
    DNMParameters {
      normalize_unicode: true,
      ..DNMParameters::default()
    },
  );
  let node = doc.get_root_readonly().unwrap();
  let dnmrange = dnm.get_range_of_node(node).unwrap();
  assert_eq!(dnmrange.get_plaintext().trim(), "At houEUR...");
}

#[test]
fn test_morpha_stemming() {
  let parser = Parser::default();
  let doc = parser.parse_file("tests/resources/file04.xml").unwrap();
  let root = doc.get_root_readonly().unwrap();
  let dnm = DNM::new(
    root,
    DNMParameters {
      stem_words_once: true,
      support_back_mapping: false,
      ..Default::default()
    },
  );
  let node = doc.get_root_readonly().unwrap();
  let dnmrange = dnm.get_range_of_node(node).unwrap().trim();

  assert_eq!(
    dnmrange.get_plaintext().trim(),
    "here be one sentence with multiple word."
  );
  rustmorpha::close();
}
