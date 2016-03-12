extern crate llamapun;
extern crate senna;
extern crate libxml;

use std::io::Write;

use llamapun::dnm::{DNM, DNMParameters, DNMRange};
use llamapun::data::{Corpus, Sentence};
use llamapun::patterns::Pattern as P;

use senna::phrase::Phrase;
use senna::senna::SennaParseOptions;
use senna::pos::POS;

use senna::sentence::Sentence as SSentence;
use senna::sentence::Word as SWord;
use libxml::xpath::Context;


fn print_range(sentence: &SSentence, from: usize, to: usize) {
    for i in from..to {
        print!("{} ", sentence.get_words()[i].get_string());
    }
}


pub fn main() {
    let mut corpus = Corpus::new("tests/resources/".to_string());
    /* corpus.set_senna_options(SennaParseOptions {
        pos: true,
        psg: true,
    }); */
    let pos_map = POS::generate_str_to_pos_map();
    let psg_map = senna::phrase::Phrase::generate_str_to_phrase_map();

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
     * Load Sentences from Corpus
     */

    let mut document = corpus.load_doc("/tmp/test.html".to_string()).unwrap();
    // let mut document = corpus.load_doc("tests/resources/1311.0066.xhtml".to_string()).unwrap();
    let xpath_context = Context::new(&document.dom).unwrap();
    let dnm = DNM::new(document.dom.get_root_element().expect("Document is empty"),
                       DNMParameters::llamapun_normalization());
    let sentences = match xpath_context.evaluate("//*[contains(@class,'ltx_para')]//span[contains(@class,'sentence')]") {
        Ok(result) => result.get_nodes_as_vec(),
        Err(_) => {
            // writeln!(std::io::stderr(), "Error: No sentences found").unwrap();
            vec![]
        }
    };
    if sentences.len() == 0 {
        writeln!(std::io::stderr(), "Warning: No sentences found (maybe document is not annotated)").unwrap();
    }

    for sentence_node in sentences {
        let sentence_id = sentence_node.get_property("id").expect("Sentence doesn't have id");
        let word_nodes = match xpath_context.evaluate(
                                    &format!("//span[@id='{}']//span[contains(@class,'word')]", sentence_id)) {
            Ok(result) => result.get_nodes_as_vec(),
            Err(_) => {
                writeln!(std::io::stderr(), "Warning: Found sentence without words (@id='{}')", sentence_id).unwrap();
                vec![]
            }
        };
        // get words of sentences in raw form
        let mut words_raw : Vec<(DNMRange, POS)> = Vec::new();
        for word_node in word_nodes {
            let pos_tag_string = word_node.get_property("pos").expect("Word doesn't have pos tag");
            let pos_tag_str : &str = &pos_tag_string;
            let pos_tag = pos_map.get(pos_tag_str)
                                 .expect(&format!("Unknown pos tag: \"{}\"", pos_tag_str));
            let dnmrange = dnm.get_range_of_node(&word_node).unwrap();
            words_raw.push((dnmrange, *pos_tag));
        }
        // sort the words
        words_raw.sort_by(|&(ref a,_), &(ref b,_)| a.start.cmp(&b.start));
        let sentence_range = dnm.get_range_of_node(&sentence_node).unwrap();
        let mut ssent = SSentence::new(sentence_range.get_plaintext());
        ssent.set_psgstring(sentence_node.get_property("psg").expect("Sentence doesn't have psg"));
        let psgroot = senna::util::parse_psg(ssent.get_psgstring().unwrap().as_bytes(),
                                             &mut 0, &mut 0, &psg_map);
        ssent.set_psgroot(psgroot);
        for i in 0..words_raw.len() {
            let mut word = SWord::new(words_raw[i].0.start - sentence_range.start,
                                     words_raw[i].0.end - sentence_range.start,
                                     words_raw[i].0.get_plaintext(), i as u32);
            word.set_pos(words_raw[i].1);
            ssent.push_word(word);
        }
        
        let s = Sentence {
            range: sentence_range.clone(),
            document: &document,
            senna_sentence: Some(ssent),
        };
        let ss = s.senna_sentence.as_ref().unwrap();

        /*
         * Apply Pattern
         */
        let matches = P::match_sentence(&s, &p_let_pattern);

        for match_ in &matches {
            println!("\n\n=========================");
            print!("Sentence: {}\n", ss.get_string());
            print!("Match: ");
            print_range(ss, match_.match_start, match_.match_end);
            print!("\n\n");
            for mark in &match_.marks {
                print!("Mark: ");
                print_range(ss, mark.offset_start, mark.offset_end);
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


    // let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();
    /* let mut document = corpus.load_doc("tests/resources/1311.0066.xhtml".to_string()).unwrap();
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
    */
}


