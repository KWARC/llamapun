//! Test that `rustsenna` is integrated properly

extern crate llamapun;
extern crate libxml;
extern crate rustsenna;

use libxml::parser::Parser;
use llamapun::dnm::{DNM, DNMParameters};
use llamapun::senna_adapter::{SennaAdapter, SennaSettings};
use rustsenna::pos::POS;


#[test]
fn test_senna_adapter() {
    let parser = Parser::default();
    let doc = parser.parse_file("tests/resources/1311.0066.xhtml").unwrap();
    let root = doc.get_root_element().unwrap();
    let dnm = DNM::new(root, DNMParameters::llamapun_normalization());
    let mut senna = SennaAdapter::new(
        SennaSettings {
            do_psg: false,   // psg takes a lot of time
            ..Default::default()
        });
    let sentences = senna.process_dnm(&dnm);

    assert!(sentences.len() > 5);
    let sentence = &sentences[5];
    assert_eq!(sentence.get_plaintext(), "We discuss functoriality and show that our Chow groups agree with the classical ones [11] for regular schemes.");
    let words = sentence.get_words();
    assert!(words.len() > 18);
    assert!(words.len() < 28);
    let word = &words[1];
    assert_eq!(word.get_plaintext(), "discuss");
    assert_eq!(word.get_pos(), POS::VBP);
}

