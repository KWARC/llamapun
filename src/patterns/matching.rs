//! The code for the actual matching

use libxml::tree::*;

use patterns::rules::*;
use patterns::utils::*;

use dnm::*;

use senna::pos::POS;
use senna::phrase::Phrase;
use senna::sentence::*;


/*
 * MATCH TYPES
 */

/// A `Match`. Note that matches are represented in a tree structure, following the structure of
/// the patterns
#[derive(Clone)]
pub struct Match<'t> {
    /// The marker associated with this match
    pub marker : MarkerEnum<'t>,
    /// The sub-matches
    pub sub_matches : Vec<Match<'t>>,
}

impl<'t> Match<'t> {
    /// returns a list of all markers
    pub fn get_marker_list(&'t self) -> Vec<MarkerEnum<'t>> {
        let mut markers = vec![self.marker.clone()];
        for m in &self.sub_matches {
            markers.extend_from_slice(&m.get_marker_list()[..]);
        }
        markers
    }
}

/// used internally for matching of math nodes
struct InternalMathMatch<'t> {
    _matches : Vec<Match<'t>>,
    matched : bool,
}

impl<'t> InternalMathMatch<'t> {
    /// indicates that no match was found
    fn no_match() -> InternalMathMatch<'t> {
        InternalMathMatch {
            _matches : Vec::new(),
            matched : false,
        }
    }
}

/// used internally for matching sequences
struct InternalSeqMatch<'t> {
    _matches : Vec<Match<'t>>,
    // start : usize,
    end : usize,
    matched : bool,
}


impl<'t> InternalSeqMatch<'t> {
    /// indicates that no match was found
    fn no_match() -> InternalSeqMatch<'t> {
        InternalSeqMatch {
            _matches : Vec::new(),
            // start : 0,
            end : 0,
            matched : false,
        }
    }
}


/// used internally for matching words
struct InternalWordMatch<'t> {
    _matches : Vec<Match<'t>>,
    matched : bool,
}

impl<'t> InternalWordMatch<'t> {
    /// indicates that no match was found
    fn no_match() -> InternalWordMatch<'t> {
        InternalWordMatch {
            _matches : Vec::new(),
            matched : false,
        }
    }
}


/*
 * PHRASE MATCHING UTILS
 */

/// internal representation of the phrase tree for faster processing
struct PhraseTree {
    phrase : Phrase,
    start : usize,
    end : usize,
    children : Vec<PhraseTree>,
}

impl PhraseTree {
    /// load the `PhraseTree` from senna's representation
    fn from_psg(root : &PSGNode) -> Result<PhraseTree, usize> {
        match root {
            &PSGNode::Leaf(pos) => Err(pos),
            &PSGNode::Parent(box ref content) => {
                let mut child_trees : Vec<PhraseTree> = Vec::new();
                let mut start : Option<usize> = None;
                let mut end = 0;
                for child in content.get_children() {
                    match PhraseTree::from_psg(&child) {
                        Err(p) => {
                            if start.is_none() {
                                start = Some(p);
                            }
                            end = p;
                        }
                        Ok(t) => {
                            if start.is_none() {
                                start = Some(t.start);
                            }
                            end = t.end;
                            child_trees.push(t);
                        }

                    }
                }
                Ok(PhraseTree {
                    phrase : content.get_label(),
                    start : start.unwrap(),
                    end : end,
                    children : child_trees,
                })
            }
        }
    }
}

fn get_phrase_matches(root : &PhraseTree, start_pos : usize, target : Phrase) -> Vec<usize> {
    if root.start == start_pos {
        let mut v = if root.children.len() > 0 {
            get_phrase_matches(&root.children[0], start_pos, target)
        } else { Vec::new() };
        if root.phrase == target {
            v.push(root.end + 1);
        }
        v
    } else if root.start < start_pos {
        for child in &root.children {
            if child.start <= start_pos && child.end >= start_pos {
                return get_phrase_matches(child, start_pos, target);
            }
        }
        Vec::new()
    } else {
        Vec::new()
    }
}



/*
 * MATCHING FUNCTIONS
 */


/// returns the matches in a sentence
pub fn match_sentence<'t>(pf : &PatternFile, sentence : &Sentence, range : &'t DNMRange, rule : &str)
-> Result<Vec<Match<'t>>, String> {
    /* if !range.dnm.parameters.support_back_mapping {
       return Err("DNM of sentence does not support back mapping".to_string());
       } */

    let rule_pos = try!(pf.sequence_rule_names.get(rule)
                        .ok_or(format!("Could not find sequence rule \"{}\"", rule)));
    let actual_rule = &pf.sequence_rules[*rule_pos];
    let words = sentence.get_words();
    let psg = try!(sentence.get_psgroot().ok_or("PSG required for pattern matching".to_string()));
    let phrase_tree = try!(PhraseTree::from_psg(psg).map_err(|_| "Invalid PSG: Contains only leaf".to_string()));

    let mut matches : Vec<Match<'t>> = Vec::new();

    for start_pos in 0..words.len() {
        let m = match_seq(pf, &actual_rule.pattern, sentence, &phrase_tree, range, start_pos);
        matches.extend_from_slice(&m._matches[..]);
    }

    Ok((matches))
}


fn match_seq<'t>(pf : &PatternFile, rule : &SequencePattern, sentence : &Sentence, phrase_tree : &PhraseTree,
                 range : &'t DNMRange, pos : usize) -> InternalSeqMatch<'t> {
    match rule {
        &SequencePattern::SeqRef(p) =>
            match_seq(pf, &pf.sequence_rules[p].pattern, sentence, phrase_tree, range, pos),
            &SequencePattern::SeqFromWord(ref wp) => {
                if pos >= sentence.get_words().len() {
                    return InternalSeqMatch::no_match();
                }
                let m = match_word(pf, &wp, &sentence.get_words()[pos], range);
                if !m.matched {
                    return InternalSeqMatch::no_match();
                }
                InternalSeqMatch {
                    _matches : m._matches,
                    // start : pos,
                    end : pos + 1,
                    matched : true,
                }
            }
        &SequencePattern::SeqOfSeq(ref patterns) => {
            let mut matches : Vec<Match> = Vec::new();
            let mut cur_pos = pos;
            for p in patterns {
                let m = match_seq(pf, p, sentence, phrase_tree, range, cur_pos);
                if !m.matched { return InternalSeqMatch::no_match(); }
                cur_pos = m.end;
                matches.extend_from_slice(&m._matches[..]);
            }
            InternalSeqMatch {
                _matches : matches,
                // start : pos,
                end : cur_pos,
                matched : true,
            }
        }
        &SequencePattern::Phrase(phrase, ref match_type, ref start_condition, ref end_condition) => {
            let mut phrase_ends = get_phrase_matches(phrase_tree, pos, phrase);
            if match_type == &PhraseMatchType::Shortest {
                phrase_ends.reverse();   // were sorted descendingly
            }

            let mut matches : Vec<Match> = Vec::new();

            for end_pos in &phrase_ends {
                matches.clear();
                if start_condition.is_some() {
                    // check start_condition
                    let &(box ref rule, ref containment) = start_condition.as_ref().unwrap();
                    let m = match_seq(pf, rule, sentence, phrase_tree, range, pos);
                    if !m.matched { continue; }
                    if containment == &SequenceContainment::LessOrEqual && m.end > *end_pos {
                        continue; 
                    }
                    matches.extend_from_slice(&m._matches[..]);
                }
                if end_condition.is_some() {
                    let mut matched = false;
                    for end_start_pos in pos .. *end_pos {
                        let &box ref rule = end_condition.as_ref().unwrap();
                        let m = match_seq(pf, rule, sentence, phrase_tree, range, end_start_pos);
                        if !m.matched {continue; }
                        matched = true;
                        matches.extend_from_slice(&m._matches[..]);
                    }
                    if !matched { continue; }
                }
                return InternalSeqMatch {
                    _matches : matches,
                    // start : pos,
                    end : *end_pos,
                    matched : true,
                };
            }

            // no phrase match satisfied all conditions:
            InternalSeqMatch::no_match()
        }
        &SequencePattern::Marked(box ref pattern, ref marker) => {
            let m = match_seq(pf, pattern, sentence, phrase_tree, range, pos);
            if m.matched {
                InternalSeqMatch {
                    _matches : vec![Match {
                        marker : MarkerEnum::Text(TextMarker {
                            marker : marker.clone(),
                            range : range.get_subrange(sentence.get_words()[pos].get_offset_start(),
                            sentence.get_words()[m.end-1].get_offset_end()),
                        }),
                        sub_matches : m._matches
                    }],
                    // start : pos,
                    end : m.end,
                    matched : true
                }
            } else  {
                InternalSeqMatch::no_match()
            }
        }
        &SequencePattern::SeqOr(ref patterns, ref match_type) => {
            let mut matches : Vec<Match> = Vec::new();
            let mut matched = false;
            let mut longest = pos;
            for p in patterns {
                let m = match_seq(pf, p, sentence, phrase_tree, range, pos);
                if m.matched {
                    matched = true;
                    match match_type {
                        &SequenceMatchType::First => {
                            matches = m._matches;
                            if m.end > longest { longest = m.end; }
                            break;
                        }
                        &SequenceMatchType::AtLeastOne | &SequenceMatchType::Any => {
                            if m.end > longest { longest = m.end; }
                            matches.extend_from_slice(&m._matches);
                        }
                        &SequenceMatchType::Longest => {
                            if m.end > longest {
                                longest = m.end;
                                matches = m._matches;
                            }
                        }
                    }
                }
            }
            if !matched && match_type != &SequenceMatchType::Any {
                InternalSeqMatch::no_match()
            } else {
                InternalSeqMatch {
                    _matches : matches,
                    // start : pos,
                    end : longest,
                    matched : true,   // don't use `matched` (because SequenceMatchType::Any matches always)
                }
            }
        }
    }
}

fn match_word<'t>(pf : &PatternFile, rule : &WordPattern, word : &Word, range : &'t DNMRange) -> InternalWordMatch<'t> {
    match rule {
        &WordPattern::WordRef(rule_pos) => match_word(pf, &pf.word_rules[rule_pos].pattern, word, range),
        &WordPattern::WordOr(ref word_patterns) => {
            for pattern in word_patterns {
                let m = match_word(pf, &pattern, word, range);
                if m.matched {
                    return m;
                }
            }
            InternalWordMatch::no_match()
        }
        &WordPattern::Word(ref word_str) => {
            if word_str == word.get_string() {
                InternalWordMatch { _matches : Vec::new(), matched : true }
            } else {
                InternalWordMatch::no_match()
            }
        }
        &WordPattern::WordPos(ref pos_pattern, box ref word_pattern) => {
            if match_pos(pf, &pos_pattern, word.get_pos()) {
                match_word(pf, &word_pattern, word, range)
            } else {
                InternalWordMatch::no_match()
            }
        }
        &WordPattern::MathWord(ref math_pattern) => {
            let node = range.dnm.back_map[range.start + word.get_offset_start()].0.clone();
            if node.get_name() != "math" {
                return InternalWordMatch::no_match();
            }
            let children = fast_get_non_text_children(&node);
            if children.len() == 0 {
                return InternalWordMatch::no_match();
            }

            // TODO: Make the following code work in the general case!!!
            let m : InternalMathMatch;
            if children[0].get_name() == "semantics" {
                let c = fast_get_non_text_children(&children[0]);
                if c.len() == 0 {
                    return InternalWordMatch::no_match()
                }
                m = match_math(pf, &math_pattern, &c[0]);
            } else {
                m = match_math(pf, &math_pattern, &children[0]);
            }
            if m.matched {
                InternalWordMatch {
                    _matches : m._matches,
                    matched : true,
                }
            } else {
                InternalWordMatch::no_match()
            }

        }
        &WordPattern::AnyWord => InternalWordMatch { _matches : Vec::new(), matched : true },
        &WordPattern::WordNot(box ref p) => {
            let m = match_word(pf, &p, word, range);
            if m.matched { m } else { InternalWordMatch::no_match() }
        }
        &WordPattern::Marked(box ref p, ref marker) => {
            let m = match_word(pf, &p, word, range);
            if m.matched {
                InternalWordMatch {
                    _matches : vec![ Match {
                        marker : MarkerEnum::Text(TextMarker {
                            range : range.get_subrange(word.get_offset_start(), word.get_offset_end()),
                            marker : marker.clone() }),
                            sub_matches : m._matches}],
                            matched : true,
                }
            } else { InternalWordMatch::no_match() }
        }
    }
}

fn match_math<'t>(pf : &PatternFile, rule : &MathPattern, node : &Node) -> InternalMathMatch<'t> {
    match rule {
        &MathPattern::AnyMath => InternalMathMatch { _matches : Vec::new(), matched : true, },
        &MathPattern::MathRef(o) => match_math(pf, &pf.math_rules[o].pattern, node),
        &MathPattern::Marked(box ref pattern, ref marker) => {
            let m = match_math(pf, &pattern, node);
            if m.matched {
                InternalMathMatch {
                    _matches : vec![ Match {
                        marker : MarkerEnum::Math(MathMarker {
                            node : node.clone(),
                            marker : marker.clone() }),
                            sub_matches : m._matches}],
                            matched : true,
                }
            } else { InternalMathMatch::no_match() }
        }
        &MathPattern::MathOr(ref patterns) => {
            for pattern in patterns {
                let m = match_math(pf, pattern, node);
                if m.matched { return m; }
            }
            InternalMathMatch::no_match()
        }
        &MathPattern::MathNode(ref name, ref mtext, ref children) => {
            // Here we will use that each MathPattern matches exactly one node for optimization
            // purposes

            if name.is_some() && name.as_ref().unwrap() != &node.get_name() {
                return InternalMathMatch::no_match();
            }

            if mtext.is_some() {
                let content = get_simple_node_content(node, false);
                if content.is_err() {
                    return InternalMathMatch::no_match();
                }
                if !match_mtext(pf, &pf.mtext_rules[mtext.unwrap()].pattern, &content.unwrap()) {
                    return InternalMathMatch::no_match();
                }
            }

            if children.is_some() {
                let &(ref child_rules, ref match_type) = children.as_ref().unwrap();
                let c_nodes = fast_get_non_text_children(node);

                if c_nodes.len() < child_rules.len() {
                    return InternalMathMatch::no_match();
                }

                let mut start_pos = 0usize;
                if match_type == &MathChildrenMatchType::MatchesExactly &&
                    c_nodes.len() != child_rules.len() {
                        return InternalMathMatch::no_match();
                    }
                if match_type == &MathChildrenMatchType::EndsWith {
                    start_pos = c_nodes.len() - child_rules.len();
                }

                loop {
                    // try to match all children
                    let mut matches : Vec<Match> = Vec::new();
                    let mut matched = true;
                    for i in 0..child_rules.len() {
                        let m = match_math(pf, &child_rules[i], &c_nodes[start_pos + i]);
                        if m.matched {
                            matches.extend_from_slice(&m._matches[..]);
                        } else {
                            matched = false;
                            break;
                        }
                    }
                    if matched {
                        return InternalMathMatch { _matches : matches, matched : true };
                    }
                    if match_type != &MathChildrenMatchType::Arbitrary { break; }
                    start_pos += 1;
                    if c_nodes.len() < child_rules.len() + start_pos { break; }
                }
                return InternalMathMatch::no_match();
            }

            return InternalMathMatch { _matches : Vec::new(), matched : true };  // no child matches required
        }
        &MathPattern::MathDescendant(box ref pattern, ref match_type) => {
            let mut matches : Vec<Match> = Vec::new();
            let mut matched = false;
            let m = match_math(pf, &pattern, node);
            if m.matched {
                matched = true;
                matches.extend_from_slice(&m._matches[..]);
            }
            for child in &fast_get_non_text_children(node) {
                if matched && match_type == &MathDescendantMatchType::First {
                    return InternalMathMatch { _matches : matches, matched : true };
                }
                let m = match_math(pf, &pattern, &child);
                if m.matched {
                    matched = true;
                    matches.extend_from_slice(&m._matches[..]);
                }
            }

            if matched || match_type == &MathDescendantMatchType::Arbitrary {
                InternalMathMatch { _matches : matches, matched : true }
            } else {
                InternalMathMatch::no_match()
            }
        }
    }
}

fn match_mtext(pf : &PatternFile, rule : &MTextPattern, string : &str) -> bool {
    match rule {
        &MTextPattern::AnyMText => true,
        &MTextPattern::MTextOr(ref ps) => ps.into_iter().any(|p| match_mtext(pf, &p, string)),
        &MTextPattern::MTextLit(ref s) => s == string,
        &MTextPattern::MTextNot(box ref p) => !match_mtext(pf, &p, string),
        &MTextPattern::MTextRef(o) => match_mtext(pf, &pf.mtext_rules[o].pattern, string),
    }
}

fn match_pos(pf : &PatternFile, rule : &PosPattern, pos : POS) -> bool {
    match rule {
        &PosPattern::Pos(p) => p == pos,
        &PosPattern::PosNot(box ref r) => !match_pos(pf, &r, pos),
        &PosPattern::PosOr(ref ps) => ps.into_iter().any(|p| match_pos(pf, &p, pos)),
        &PosPattern::PosRef(o) => match_pos(pf, &pf.pos_rules[o].pattern, pos),
    }
}


