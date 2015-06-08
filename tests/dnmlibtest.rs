//! Tests for the DNM library

extern crate llamapun;
extern crate rustlibxml;

use llamapun::dnmlib::*;
use rustlibxml::tree::XmlDoc;
use std::collections::HashMap;


#[test]
/// Test the plaintext generation for a simple file
fn test_plaintext_simple() {
    let doc = XmlDoc::parse_file("tests/resources/file01.xml").unwrap();
    let mut options : HashMap<String, SpecialTagsOption> = HashMap::new();
    options.insert("h1".to_string(),
                   SpecialTagsOption::Enter);  //actually default behaviour 
    options.insert("h2".to_string(), SpecialTagsOption::Skip);
    options.insert("a".to_string(),
                   SpecialTagsOption::Normalize("[link]".to_string()));
    let dnm = DNM::create_dnm(doc.get_root_element().unwrap(),
                              DNMParameters {
                                  special_tags_options : options,
                                  ..Default::default()
                              });
    assert_eq!(dnm.plaintext.trim(),
               "Title Some text [link] and a bit more text.");
}




#[test]
/// Test the xmlNode -> plaintext mapping
fn test_xml_node_to_plaintext() {
    let doc = XmlDoc::parse_file("tests/resources/file01.xml").unwrap();
    let mut options : HashMap<String, SpecialTagsOption> = HashMap::new();
    options.insert("h1".to_string(),
                   SpecialTagsOption::Enter);  //actually default behaviour 
    options.insert("h2".to_string(), SpecialTagsOption::Skip);
    options.insert("a".to_string(),
                   SpecialTagsOption::Normalize("[link]".to_string()));
    let dnm = DNM::create_dnm(doc.get_root_element().unwrap(),
                              DNMParameters {
                                  special_tags_options : options,
                                  ..Default::default()
                              });
    let mut node = doc.get_root_element().unwrap();
    match node.get_first_child() {
        Some(n) => node = n,
        None => assert!(false)  //DOM generation failed
    }
    while node.get_name() != "body" {
        match node.get_next_sibling() {
            Some(n) => node = n,
            None => assert!(false)
        }
    }
    node = node.get_first_child().unwrap();
    while node.get_name() != "h1" {
        match node.get_next_sibling() {
            Some(n) => node = n,
            None => assert!(false)
        }
    }
    //Node content should have been processed
    assert_eq!(dnm.get_range_of_node(node).unwrap().get_plaintext(), "Title");
    while node.get_name() != "h2" {
        match node.get_next_sibling() {
            Some(n) => node = n,
            None => assert!(false)
        }
    }
    //node was skipped in dnm generation
    assert_eq!(dnm.get_range_of_node(node).unwrap().get_plaintext(), "");
    while node.get_name() != "a" {
        match node.get_next_sibling() {
            Some(n) => node = n,
            None => assert!(false)
        }
    }
    //node content should have been replaced by "[link]"
    assert_eq!(dnm.get_range_of_node(node).unwrap().get_plaintext().trim(), "[link]");
}
