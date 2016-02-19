

use senna::pos::POS;
use senna::phrase::Phrase;
use data::Sentence;

use senna::sentence::Sentence as SSentence;
use senna::sentence::{PSGNode, PSGPhrase};


use std::io::{self, Write};


pub enum Pattern<'t, MarkerT, NoteT> where MarkerT: 't + Clone , NoteT: 't + Clone {
    W(&'t str),
    // Ws(Vec<&'t str>),
    WP(&'t str, Vec<POS>),
    // WsP(Vec<&'t str>, Vec<POS>),
    P(Vec<POS>),
    // Phr0(Phrase),
    // PhrS(Phrase, &'t Pattern<'t, MarkerT, NoteT>),
    // PhrE(Phrase, &'t Pattern<'t, MarkerT, NoteT>),
    // PhrSE(Phrase, &'t Pattern<'t, MarkerT, NoteT>, &'t Pattern<'t, MarkerT, NoteT>),
    Marked(MarkerT, Vec<NoteT>, &'t Pattern<'t, MarkerT, NoteT>),
}

pub struct Match<MarkerT: Clone, NoteT: Clone> {
    marks: Vec<Mark<MarkerT, NoteT>>,
    match_start: usize,
    match_end: usize,
}

pub struct Mark<MarkerT, NoteT> where MarkerT: Clone, NoteT: Clone {
    offset_start: usize,  // in words
    offset_end: usize,
    marker: MarkerT,
    notes: Vec<NoteT>,
}


impl <'t, MarkerT: Clone, NoteT: Clone> Pattern<'t, MarkerT, NoteT> {
    pub fn match_sentence<'a>(sentence: &'a mut Sentence<'a>, pattern: &Pattern<'t, MarkerT, NoteT>)
                    -> Vec<Match<MarkerT, NoteT>> {
        let mut matches : Vec<Match<MarkerT, NoteT>> = Vec::new();
        let s = sentence.senna_parse();
        let sensent = &s.senna_sentence;

        for i in 0..sensent.as_ref().unwrap().get_words().len() {
            match Pattern::rec_match(pattern, i, sensent.as_ref().unwrap()) {
                None => { }
                Some((m, end)) => {
                    matches.push(
                        Match {
                            marks: match m {
                                None => vec![],
                                Some(boxed) => *boxed,
                            },
                            match_start: i,
                            match_end: end,
                        });
                }
            }
        }
        return matches;
    }

    fn rec_match(pattern: &Pattern<'t, MarkerT, NoteT>, pos: usize, sent: &SSentence)
            -> Option<(Option<Box<Vec<Mark<MarkerT, NoteT>>>>, usize)> {   //(markers, endpos)
        if pos > sent.get_words().len() {
            return None;
        }
        match pattern {
            &Pattern::W(s) => {
                let word = &sent.get_words()[pos];
                if word.get_string() == s {
                    return Some((None, pos));
                } else {
                    return None;
                }
            }
            &Pattern::WP(s, ref p) => {
                let word = &sent.get_words()[pos];
                if word.get_string() == s && p.contains(&word.get_pos()) {
                    return Some((None, pos));
                } else {
                    return None;
                }
            }
            &Pattern::P(ref p) => {
                let word = &sent.get_words()[pos];
                if p.contains(&word.get_pos()) {
                    return Some((None, pos));
                } else {
                    return None;
                }
            }
            &Pattern::Marked(ref marker, ref notes, ref pattern) => {
                let m = Pattern::rec_match(pattern, pos, sent);
                match m {
                    None => { return None; }
                    Some((marks, end)) => {
                        let nm : Mark <MarkerT, NoteT> = Mark {
                                    offset_start: pos,
                                    offset_end: end,
                                    marker: marker.clone(),
                                    notes: notes.clone(),
                                };
                        let ms = match marks {
                            None => { Box::new(vec![nm])   }
                            Some(mut v) => { (*v).push(nm); v }
                        };

                        return Some((Some(ms), end));
                    }
                }
            }

    // Marked(MarkerT, Vec<NoteT>, &'t Pattern<'t, MarkerT, NoteT>),
/* pub struct Match<MarkerT, NoteT> {
    offset_start: usize,
    offset_end: usize,
    marker: MarkerT,
    notes: Vec<NoteT>,
} */

        }
    }
}


