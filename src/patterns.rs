use senna::pos::POS;
use senna::phrase::Phrase;
use senna::sentence::Sentence as SSentence;
use senna::sentence::{PSGNode, PSGPhrase};

use data::Sentence;

use std::io::{self, Write};
use std::cmp::Ordering;




/*
 * PSG HELPER FUNCTIONS
 */

fn get_psg_start(phrase: &PSGPhrase) -> usize {
    let first_child = &phrase.get_children()[0];   // every phrase has at least one child
    match first_child {
        &PSGNode::Leaf(pos) => pos,
        &PSGNode::Parent(ref p) => get_psg_start(p.as_ref())
    }
}

/// returns index of last word in phrase + 1
fn get_psg_end(phrase: &PSGPhrase) -> usize {
    let last_child = &phrase.get_children()[phrase.get_children().len() - 1];
    match last_child {
        &PSGNode::Leaf(pos) => pos+1,
        &PSGNode::Parent(ref p) => get_psg_end(p.as_ref())
    }
}

/// returns the highest `PSGPhrase` that starts with word `pos`
fn get_top_psg_of_word(sent: &SSentence, pos: usize) -> Option<PSGPhrase> {
    // let root_ = sent.get_psgroot();
    if sent.get_psgroot().is_none() {
        writeln!(&mut io::stderr(), "llamapun::patterns::get_top_psg_of_word: Warning: PSG not generated!").unwrap();
        return None;
    }
    match sent.get_psgroot().unwrap() {
        &PSGNode::Leaf(_) => {
            return None;
        }
        &PSGNode::Parent(ref p) => {
            let mut root = p.as_ref();
            loop {
                match pos.cmp(&get_psg_start(root)) {
                    Ordering::Less => {
                        let mut new_root: Option<&PSGPhrase> = None;
                        // find last child that starts a word with less than pos
                        for child in root.get_children() {
                            match child {
                                &PSGNode::Leaf(_) => {  }
                                &PSGNode::Parent(ref p) => {
                                    match pos.cmp(&get_psg_start(p.as_ref())) {
                                        Ordering::Less => { new_root = Some(p.as_ref()); }
                                        Ordering::Equal => { return Some(p.as_ref().clone()); }
                                        Ordering::Greater => { return None; }
                                    }
                                }
                            }
                        }
                        root = new_root.unwrap();
                    }
                    Ordering::Equal => {
                        return Some(root.clone());
                    }
                    Ordering::Greater => {
                        return None;
                    }
                }
            }
        }
    }
    unreachable!();
}

fn psg_get_top_left_child_phrase(pt: Phrase, psg: &PSGPhrase) -> Option<PSGPhrase> {
    if psg.get_label() == pt {
        Some(psg.clone())
    } else {
        let left_child = &psg.get_children()[0];  // at least one child
        match left_child {
            &PSGNode::Leaf(_) => None,
            &PSGNode::Parent(ref p) => psg_get_top_left_child_phrase(pt, p.as_ref()),
        }
    }
}



/*
 * public enums and struct
 */

#[derive(Clone)]
pub enum Pattern<'t, MarkerT, NoteT> where MarkerT: 't + Clone , NoteT: 't + Clone {
    W(&'t str),
    // Ws(Vec<&'t str>),
    WP(&'t str, Vec<POS>),
    // WsP(Vec<&'t str>, Vec<POS>),
    P(Vec<POS>),
    Phr0(Phrase),
    // PhrS(Phrase, &'t Pattern<'t, MarkerT, NoteT>),
    // PhrE(Phrase, &'t Pattern<'t, MarkerT, NoteT>),
    // PhrSE(Phrase, &'t Pattern<'t, MarkerT, NoteT>, &'t Pattern<'t, MarkerT, NoteT>),
    Marked(MarkerT, Vec<NoteT>, &'t Pattern<'t, MarkerT, NoteT>),
    Seq(Vec<&'t Pattern<'t, MarkerT, NoteT>>),
}

#[derive(Clone)]
pub struct Match<MarkerT: Clone, NoteT: Clone> {
    marks: Vec<Mark<MarkerT, NoteT>>,
    match_start: usize,
    match_end: usize,
}

#[derive(Clone)]
pub struct Mark<MarkerT, NoteT> where MarkerT: Clone, NoteT: Clone {
    offset_start: usize,  // in words
    offset_end: usize,
    marker: MarkerT,
    notes: Vec<NoteT>,
}



/*
 * Matcher implementation
 */

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
                    return Some((None, pos+1));
                } else {
                    return None;
                }
            }
            &Pattern::WP(s, ref p) => {
                let word = &sent.get_words()[pos];
                if word.get_string() == s && p.contains(&word.get_pos()) {
                    return Some((None, pos+1));
                } else {
                    return None;
                }
            }
            &Pattern::P(ref p) => {
                let word = &sent.get_words()[pos];
                if p.contains(&word.get_pos()) {
                    return Some((None, pos+1));
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
            &Pattern::Seq(ref seq) => {
                let mut cur_pos = pos;
                let mut new_marks : Box<Vec<Mark<MarkerT, NoteT>>> = Box::new(Vec::new());
                for pattern in seq {
                    let m = Pattern::rec_match(pattern, cur_pos, sent);
                    match m {
                        None => { return None; }
                        Some((marks, end)) => {
                            match marks {
                                None => { }
                                Some(v) => { (*new_marks).extend_from_slice(&v) }
                            };
                            cur_pos = end;
                        }
                    }
                }
                if new_marks.len() > 0 {
                    return Some((Some(new_marks), cur_pos));
                } else {
                    return Some((None, cur_pos));
                }
            }
            &Pattern::Phr0(pat) => {
                match get_top_psg_of_word(sent, pos) {
                    None => {
                        return None;
                    }
                    Some(ref r) => {
                        match psg_get_top_left_child_phrase(pat, r) {
                            None => {
                                return None;
                            }
                            Some(ref p) => {
                                return Some((None, get_psg_end(p)));
                            }
                        }
                    }
                }
            }
        }
    }
}


