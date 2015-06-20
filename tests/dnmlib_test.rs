//! Tests for the DNM library

extern crate llamapun;
extern crate rustlibxml;
extern crate libc;

use llamapun::dnmlib::*;
// use libc::{c_void, c_int};
use rustlibxml::tree::XmlDoc;
use rustlibxml::xpath::{XmlXPathContext};
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
    let dnm = DNM::create_dnm(&doc.get_root_element().unwrap(),
                              DNMParameters {
                                  special_tag_name_options : options,
                                  normalize_white_spaces : true,
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
    let dnm = DNM::create_dnm(&doc.get_root_element().unwrap(),
                              DNMParameters {
                                  special_tag_name_options : options,
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
    assert_eq!(dnm.get_range_of_node(&node).unwrap().get_plaintext(), "Title");
    while node.get_name() != "h2" {
        match node.get_next_sibling() {
            Some(n) => node = n,
            None => assert!(false)
        }
    }
    //node was skipped in dnm generation
    assert_eq!(dnm.get_range_of_node(&node).unwrap().get_plaintext(), "");
    while node.get_name() != "a" {
        match node.get_next_sibling() {
            Some(n) => node = n,
            None => assert!(false)
        }
    }
    //node content should have been replaced by "[link]"
    assert_eq!(dnm.get_range_of_node(&node).unwrap().get_plaintext().trim(), "[link]");
}



#[test]
/// Test that the normalization according to class attributes works
fn test_plaintext_normalized_class_names() {
    let doc = XmlDoc::parse_file("tests/resources/file02.xml").unwrap();
    let mut options : HashMap<String, SpecialTagsOption> = HashMap::new();
    options.insert("normalized".to_string(),
                   SpecialTagsOption::Normalize("[NORMALIZED]".to_string()));
    let dnm = DNM::create_dnm(&doc.get_root_element().unwrap(),
                              DNMParameters {
                                  special_tag_class_options : options,
                                  normalize_white_spaces : true,
                                  ..Default::default()
                              });
    assert_eq!(dnm.plaintext.trim(), "[NORMALIZED] Else");
}

/*
    #[test]
    /// Test the default math normalization on some real math document
    fn test_default_math_normalization() {
        let doc = XmlDoc::parse_file("tests/resources/1311.0066.xhtml").unwrap();
        let dnm = DNM::create_dnm(&doc.get_root_element().unwrap(),
                                  DNMParameters::llamapun_normalization());
        assert_eq!(dnm.plaintext, "abc");
    }
*/

#[test]
/// test if parameter option `move_whitespaces_between_nodes` works
fn test_move_whitespaces_between_nodes() {
    let doc = XmlDoc::parse_file("tests/resources/file01.xml").unwrap();
    let dnm = DNM::create_dnm(&doc.get_root_element().unwrap(),
                              DNMParameters {
                                  move_whitespaces_between_nodes: true,
                                  normalize_white_spaces: true,
                                  ..Default::default() });
    let context = XmlXPathContext::new(&doc).unwrap();
    let result = context.evaluate("/html/body/h2").unwrap();
    assert_eq!(result.get_number_of_nodes(), 1);
    let node = &result.get_nodes_as_vec()[0];
    if let Some(node) = node.get_next_sibling() {
        let range = dnm.get_range_of_node(&node).unwrap();
        assert_eq!(range.get_plaintext(), "Some text");
    } else {
        assert!(false);   // node should have had a sibling
    }
}


#[test]
/// test unicode normalization
fn test_unicode_normalization() {
    let doc = XmlDoc::parse_file("tests/resources/file03.xml").unwrap();
    let dnm = DNM::create_dnm(&doc.get_root_element().unwrap(),
                              DNMParameters {
                                  normalize_unicode: true,
                                  ..Default::default() });
    let node = doc.get_root_element().unwrap();
    let dnmrange = dnm.get_range_of_node(&node).unwrap();
    assert_eq!(dnmrange.get_plaintext().trim(), "At houEUR...");
}
