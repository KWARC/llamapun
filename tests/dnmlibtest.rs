//! Tests for the DNM library

extern crate llamapun;
extern crate rustlibxml;

use llamapun::dnmlib::*;
use rustlibxml::tree::XmlDoc;
use std::collections::HashMap;


#[test]
/// Test the plaintext generation for a simple file
fn test_plaintext() {
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
               "Title Some text [link] and a bit more text.".to_string());
}
