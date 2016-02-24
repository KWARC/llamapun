extern crate llamapun;
extern crate senna;

use llamapun::data::{Corpus};
use llamapun::patterns::Pattern;

use senna::phrase::Phrase;
use senna::senna::SennaParseOptions;


#[test]
fn test_number_of_oneword_matches() {
    let corpus = Corpus::new(".".to_string());
    let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    let mut match_count = 0;
    let pattern : Pattern<&str, &str> = Pattern::W("interpretation");
    for mut sentence in document.sentence_iter() {
        let s = sentence.senna_parse();
        match_count += Pattern::match_sentence(&s, &pattern).len();
    }
    assert_eq!(match_count, 5);
}

#[test]
fn test_number_of_seq_matches() {
    let corpus = Corpus::new(".".to_string());
    let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    let mut match_count = 0;
    let p_certain = Pattern::W("certain");
    let p_properties = Pattern::W("properties");
    let pattern : Pattern<&str, &str> = Pattern::Seq(vec![p_certain,
                                                          p_properties]);
    for mut sentence in document.sentence_iter() {
        let s = sentence.senna_parse();
        match_count += Pattern::match_sentence(&s, &pattern).len();
    }
    assert_eq!(match_count, 1);
}


#[test]
fn test_simple_marked_phrase_match() {
    let mut corpus = Corpus::new(".".to_string());
    corpus.set_senna_options(SennaParseOptions {
        pos: true,
        psg: true,
    });
    let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    let mut match_count = 0;
    let p_let = Pattern::W("let");
    let p_mf = Pattern::W("MathFormula");
    let p_be = Pattern::W("be");
    let p_indep = Pattern::W("independent");
    let p_ind_phrs = Pattern::PhrS(Phrase::NP, false, &p_indep);
    let p_phr_marked = Pattern::Marked("mathobject", vec!["definiens"],
                                       &p_ind_phrs);
    let pattern : Pattern<&str, &str> = Pattern::Seq(
        vec![p_let, p_mf, p_be, p_phr_marked]);
    for mut sentence in document.sentence_iter() {
        let s = sentence.senna_parse();
        let matches = Pattern::match_sentence(&s, &pattern);
        for m in &matches {
            assert_eq!(m.marks.len(), 1);
            let mark = &m.marks[0];
            assert_eq!(mark.offset_start + 3, mark.offset_end);
            assert_eq!(mark.marker, "mathobject");
            assert!(mark.notes.contains(&"definiens"));
            assert_eq!(m.match_start + 6, m.match_end);
            assert_eq!(s.senna_sentence.as_ref().unwrap().get_words()[mark.offset_end-1].get_string(), "variables");
        }
        match_count += matches.len();
    }
    assert_eq!(match_count, 1);
}



