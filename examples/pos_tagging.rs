extern crate llamapun;

use llamapun::data::Corpus;

pub fn main() {
    let corpus = Corpus::new("tests/resources/".to_string());
    let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    for mut sentence in document.sentence_iter() {
        println!("\n --- New Sentence ---\n");
        for word in sentence.senna_iter() {
            println!("'{}'\t{}", word.range.get_plaintext(), word.pos.to_str());
        }
    }
}

