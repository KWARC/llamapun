extern crate llamapun;
extern crate senna;

use llamapun::data::Corpus;
use llamapun::patterns::Pattern as P;

use senna::phrase::Phrase;
use senna::senna::SennaParseOptions;

use senna::sentence::Sentence as SSentence;


fn print_range(sentence: &SSentence, from: usize, to: usize) {
    for i in from..to {
        print!("{} ", sentence.get_words()[i].get_string());
    }
}


pub fn main() {
    let mut corpus = Corpus::new("tests/resources/".to_string());
    corpus.set_senna_options(SennaParseOptions {
        pos: true,
        psg: true,
    });

    /*
     * Create pattern
     */

    let p_indefinite_article = P::Or(vec![P::W("a"), P::W("an"), P::W("any"), P::W("some"), P::W("every")]);
    let p_mathformula = P::W("MathFormula");
    let p_mf_marked = P::Marked("definiendum", vec![], &p_mathformula);

    let p_short_dfs = P::PhrS(Phrase::NP, false, &p_indefinite_article);
    let p_short_dfs_marked = P::Marked("definiens", vec!["with article", "short"], &p_short_dfs);

    // let p_quantifier_existential = P::Seq(vec![P::W("there"), P::Or(vec![P::W("is"), P::W("exists")])]);
    // let p_quantifier_universal = P::W("for");
                                           
    let p_let_pattern = P::Seq(vec![P::W("let"), p_mf_marked.clone(), P::W("be"), p_short_dfs_marked.clone()]);
                                       

    /*
     * Use pattern
     */


    // let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    let mut document = corpus.load_doc("tests/resources/1311.0066.xhtml".to_string()).unwrap();
    for mut sentence in document.sentence_iter() {
        let s = sentence.senna_parse();
        let matches = P::match_sentence(&s, &p_let_pattern);
        let ssent = s.senna_sentence.as_ref().unwrap();
        for match_ in &matches {
            println!("\n\n=========================");
            print!("Sentence: {}\n", ssent.get_string());
            print!("Match: ");
            print_range(ssent, match_.match_start, match_.match_end);
            print!("\n\n");
            for mark in &match_.marks {
                print!("Mark: ");
                print_range(ssent, mark.offset_start, mark.offset_end);
                print!("\nType: {}\n", mark.marker);
                if mark.notes.len() > 0 {
                    print!("Notes: ");
                    for note in &mark.notes {
                        print!("{}, ", note);
                    }
                    print!("\n");
                }
                print!("\n\n\n");
            }
        }
    }
}


