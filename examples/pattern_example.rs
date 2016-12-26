extern crate llamapun;
extern crate senna;

use llamapun::patterns::PatternFile;
use llamapun::data::Corpus;
use senna::senna::SennaParseOptions;



pub fn main() {
    let pattern_file_result = PatternFile::load("examples/declaration_pattern.xml");
    let pattern_file = match pattern_file_result {
        Err(x) => panic!(x),
        Ok(x) => x,
    };

    let mut corpus = Corpus::new("tests/resources/".to_string());
    corpus.senna_options = std::cell::Cell::new( SennaParseOptions { pos : true, psg : true } );
    corpus.dnm_parameters.support_back_mapping = true;

    let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    for mut sentence in document.sentence_iter() {
        let sentence_2 = sentence.senna_parse();
        let matches = pattern_file.match_sentence(&sentence_2.senna_sentence.as_ref().unwrap(),
                                                       &sentence_2.range, "declaration").unwrap();
        if matches.len() > 0 {
            println!("Found match: \"{}\"", sentence_2.range.get_plaintext());
        }
    }
}
