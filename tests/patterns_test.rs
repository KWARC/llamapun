extern crate llamapun;

use llamapun::data::{Corpus};
use llamapun::patterns::Pattern;


#[test]
fn test_number_of_oneword_matches() {
    let corpus = Corpus::new(".".to_string());
    let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    let mut match_count = 0;
    let pattern : Pattern<&str, &str> = Pattern::W("interpretation");
    for mut sentence in document.sentence_iter() {
        match_count += Pattern::match_sentence(&mut sentence, &pattern).len();
    }
    assert_eq!(match_count, 5);
}

