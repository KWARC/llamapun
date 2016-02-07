extern crate llamapun;

use llamapun::data::Corpus;

pub fn main() {
    let corpus = Corpus::new("tests/resources/".to_string());
    let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    for mut sentence in document.sentence_iter() {
        print!("\n --- New Sentence ---\n\n");
        for word in sentence.senna_iter() {
            print!("'{}'\t{}\n", word.range.get_plaintext(), word.pos.to_str());
        }
    }
}

