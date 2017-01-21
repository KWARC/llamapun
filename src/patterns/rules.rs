//! The definitions of patterns and rules, and the code for loading them

use libxml::tree::*;
use libxml::parser::Parser;

use senna::pos::POS;
use senna::phrase::Phrase;

use dnm::*;

use std::collections::HashMap;

use patterns::utils::*;


/*
 * MARKERS
 */

/// The marker used for marking patterns.
/// If a match was found, this marker will be returned (along with the marked range).
#[derive(Clone)]
pub struct PatternMarker {
    /// name of the marker
    pub name: String,
    /// tags of the marker
    pub tags: Vec<String>,
}

impl PatternMarker {
    /// loads a `PatternMarker` from a node
    fn load_from_node(node : &Node) -> Result<PatternMarker, String> {
        let name = try!(require_node_property(node, "name"));
        let tags = match node.get_property("tags") {   // TODO: Add regex: [a-zA-Z0-9_]+(,[a-zA-Z0-9_]+)*
            None => Vec::new(),
            Some(value) => value.split(",").map(|s| s.trim().to_string()).collect(),
        };
        return Ok(PatternMarker { name : name, tags : tags });
    }
}


/// A marked math node
#[derive(Clone)]
pub struct MathMarker {
    /// the marked math node
    pub node : Node,
    /// the marker
    pub marker : PatternMarker,
}

/// A marked text range
#[derive(Clone)]
pub struct TextMarker<'t> {
    /// the marked range
    pub range : DNMRange<'t>,
    /// the marker
    pub marker : PatternMarker,
}


/// Any marked result
#[derive(Clone)]
pub enum MarkerEnum<'t> {
    /// a marked text range
    Text(TextMarker<'t>),
    /// a marked math node
    Math(MathMarker),
}


/*
 * MATCH TYPES FOR PATTERNS
 */


/// Describes the ways of matching the children of a math node
#[derive(Clone, PartialEq, Eq)]
pub enum MathChildrenMatchType {
    /// The patterns for the children have to match at the beginning,
    /// i.e. the first children
    StartsWith,
    /// The patterns for the children have to match all the children
    MatchesExactly,
    /// The patterns for the children have to match the last children
    EndsWith,
    /// Any kind of matching is acceptable, but only the first match will be used.
    Arbitrary
}

impl MathChildrenMatchType {
    /// loads a `MathChildrenMatchType` from a string
    fn from_str(string : &str) -> Result<MathChildrenMatchType, String> {
        match string {
            "starts_with" => Ok(MathChildrenMatchType::StartsWith),
            "exact" => Ok(MathChildrenMatchType::MatchesExactly),
            "ends_with" => Ok(MathChildrenMatchType::EndsWith),
            "arbitrary" => Ok(MathChildrenMatchType::Arbitrary),
            other => Err(format!("Unknown match_type for match_children \"{}\"", other)),
        }
    }
}


/// The ways of matching the descendants of a math node
#[derive(Clone, PartialEq, Eq)]
pub enum MathDescendantMatchType {
    /// Take the first match only (DFS)
    First,
    /// Require at least one matching descending
    AtLeastOne,
    /// Take all matching descendants
    Arbitrary
}

impl MathDescendantMatchType {
    /// loads a `MathDescendantMatchType` from a string
    fn from_str(string : &str) -> Result<MathDescendantMatchType, String> {
        match string {
            "first" => Ok(MathDescendantMatchType::First),
            "at_least_one" => Ok(MathDescendantMatchType::AtLeastOne),
            "arbitrary" => Ok(MathDescendantMatchType::Arbitrary),
            other => Err(format!("Unknon match_type for math_descendant \"{}\"", other)),
        }
    }
}


/// Describes how a phrase should be matched
#[derive(Clone, PartialEq, Eq)]
pub enum PhraseMatchType {
    /// Take the shortest phrase fulfilling the requirements
    Shortest,
    /// Take the longest phrase fulfilling the requirements
    Longest
}

impl PhraseMatchType {
    /// loads a `PhraseMatchType` from a string
    fn from_str(string : &str) -> Result<PhraseMatchType, String> {
        match string {
            "shortest" => {
                Ok(PhraseMatchType::Shortest)
            }
            "longest" => {
                Ok(PhraseMatchType::Longest)
            }
            unknown => {
                Err(format!("Unknown match_type \"{}\"", unknown))
            }
        }
    }
}

/// Describes how a sequence should be contained in another sequence
#[derive(Clone, PartialEq, Eq)]
pub enum SequenceContainment {
    /// The sequence should not be longer
    LessOrEqual,
    /// Any containment is acceptable (including if the sequence is not contained, i.e. longer)
    Any,
}

impl SequenceContainment {
    /// loads a `SequenceContainment` from a string
    fn from_node(node : &Node) -> Result<SequenceContainment, String> {
        match node.get_property("containment") {
            None => Ok(SequenceContainment::Any),
            Some(ref x) => {
                match x.as_ref() {
                    "lessorequal" => {
                        Ok(SequenceContainment::LessOrEqual)
                    }
                    "any" => {
                        Ok(SequenceContainment::Any)
                    }
                    unknown => {
                        Err(format!("Unknown containment \"{}\"", unknown))
                    }
                }
            }
        }
    }
}


/// Describes how the sequences in a `seq_or` listing should be matched
#[derive(Clone, PartialEq, Eq)]
pub enum SequenceMatchType {
    /// Take only the first sequence that matches
    First,
    /// Try to match all and require at least one to match
    AtLeastOne,
    /// Try to match all
    Any,
    /// Take only the longest matching sequence
    Longest,
}



/*
 * PATTERNS
 */


/// A pattern for matching math nodes
/// Currently, only nodes, not ranges of nodes can be matched
#[derive(Clone)]
pub enum MathPattern {
    /// Matches any math node
    AnyMath,
    /// Reference to another math node
    MathRef(usize),
    /// A `MathPattern` with a marker
    Marked(Box<MathPattern>, PatternMarker),
    /// Matches the first matching pattern in the vector
    MathOr(Vec<MathPattern>),
    /// Pattern to match a math node with optional restrictions on the
    /// * Name of the node (e.g. `mi`)
    /// * Text content (if it has only one text node as a child), by passing a reference to an
    /// `MTextPattern`
    /// * The children it has
    MathNode(Option<String> /* node name */,
             Option<usize> /* m_text_ref */,
             Option<(Vec<MathPattern>, MathChildrenMatchType)> /*children */),
    /// Pattern to match descendants in a DFS-like way
    MathDescendant(Box<MathPattern>, MathDescendantMatchType),
}



/// A pattern for the text (symbols) contained in math nodes
#[derive(Clone)]
pub enum MTextPattern {
    /// Matches any symbol
    AnyMText,
    /// Matches if any of the `MTextPattern`s in the vector match
    MTextOr(Vec<MTextPattern>),
    /// Matches, if a symbol matches the string
    MTextLit(String),
    /// Matches, if the referenced pattern does not match
    MTextNot(Box<MTextPattern>),
    /// Reference to another `MTextPattern`
    MTextRef(usize),
}


/// Pattern for matching POS tags
#[derive(Clone)]
pub enum PosPattern {
    /// Matches a particular POS tag
    Pos(POS),
    /// Matches if the referenced pattern does not match
    PosNot(Box<PosPattern>),
    /// Matches, if any of the patterns in the vector match
    PosOr(Vec<PosPattern>),
    /// Matches a referenced `PosPattern`
    PosRef(usize),
}


/// Pattern for matching words
#[derive(Clone)]
pub enum WordPattern {
    /// Matches a referenced `WordPattern`
    WordRef(usize),
    /// Matches, if one of the patterns in the vector match
    WordOr(Vec<WordPattern>),
    /// Matches, if a word corresponds to the string
    Word(String),
    /// Restricts the word by a `PosPattern`
    WordPos(PosPattern, Box<WordPattern>),
    /// Matches, if the word corresponds to a math node matching the `MathPattern`
    MathWord(MathPattern),
    /// Matches any word
    AnyWord,
    /// Matches any word not matching the referenced `WordPattern`
    WordNot(Box<WordPattern>),
    /// A `WordPattern` with a marker
    Marked(Box<WordPattern>, PatternMarker),
}

/// Pattern for matching sequences of words
#[derive(Clone)]
pub enum SequencePattern {
    /// reference to another sequence
    SeqRef(usize),
    /// Transforms a word into a sequence (of length 1 of course)
    SeqFromWord(WordPattern),
    /// A sequence of sequences
    SeqOfSeq(Vec<SequencePattern>),
    /// Matches a phrase, with optional restrictions on the beginning and end of the sequence
    Phrase(Phrase, PhraseMatchType,
           Option<(Box<SequencePattern>, SequenceContainment)>,
           Option<Box<SequencePattern>>),
    /// A `SequencePattern` with a marker
    Marked(Box<SequencePattern>, PatternMarker),
    /// Matches the `SequencePattern`s in the vector following the `SequenceMatchType`
    SeqOr(Vec<SequencePattern>, SequenceMatchType),
}



/*
 * RULES
 */


/// A meta description of a rule/file
#[derive(Clone)]
pub struct MetaDescription {
    /// name of the rule/file
    name: String,
    /// summary about the purpose/functioning of the rule/file
    summary: String,
}


impl MetaDescription {
    /// Loads a `MetaDescription` from a `<meta>` node
    fn load_from_node(node : &Node, name: String) -> Result<MetaDescription, String> {
        if node.get_name().ne("meta") {
            return Err(format!("expected meta node, found \"{}\"", &node.get_name()));
        }

        let mut summary : Option<String> = None;

        for cur in &try!(get_non_text_children(node)) {
            match cur.get_name().as_ref() {
                "description" => {
                    try!(check_found_property_already(&summary, "description", "meta"));
                    summary = Some(try!(
                            get_simple_node_content(&cur, true)
                            .map_err(|e| format!("error in meta node:\n{}", e))));
                }
                &_ => {
                    return Err(format!("unexpected node in meta node: \"{}\"", cur.get_name()));
                }
            }
        }

        Ok(MetaDescription {
            name: name,
            summary: summary.unwrap_or_else(|| String::new()),
        })
    }
}



/// A rule for matching words
#[derive(Clone)]
pub struct WordRule {
    /// description of the rule
    pub description: MetaDescription,
    /// the pattern
    pub pattern: WordPattern,
}

/// A rule for matching POS tags
#[derive(Clone)]
pub struct PosRule {
    /// description of the rule
    pub description: MetaDescription,
    /// the pattern
    pub pattern: PosPattern,
}

#[derive(Clone)]
pub struct MathRule {
    /// description of the rule
    pub description: MetaDescription,
    /// the pattern
    pub pattern: MathPattern,
}

#[derive(Clone)]
pub struct MTextRule {
    /// description of the rule
    pub description: MetaDescription,
    /// the pattern
    pub pattern: MTextPattern,
}

#[derive(Clone)]
pub struct SequenceRule {
    /// description of the rule
    pub description: MetaDescription,
    /// the pattern
    pub pattern: SequencePattern,
}

/// Pattern Loading Context
/// (internal data structure for loading the patterns from a file)
/// it keep tracks of loaded and referenced rules
struct PCtx<'t> {
    pos_map : HashMap<&'t str, POS>,
    phrase_map : HashMap<&'t str, Phrase>,
    word_rules : Vec<Option<WordRule>>,
    seq_rules : Vec<Option<SequenceRule>>,
    math_rules : Vec<Option<MathRule>>,
    mtext_rules : Vec<Option<MTextRule>>,
    pos_rules : Vec<Option<PosRule>>,
    word_name_map : HashMap<String, usize>,
    seq_name_map : HashMap<String, usize>,
    math_name_map : HashMap<String, usize>,
    mtext_name_map : HashMap<String, usize>,
    pos_name_map : HashMap<String, usize>,
}


/// Contains rules loaded from a pattern file
pub struct PatternFile {
    /// description of the file
    pub description: MetaDescription,

    /// the word rules
    pub word_rules: Vec<WordRule>,
    /// the POS rules
    pub pos_rules: Vec<PosRule>,
    /// the math rules
    pub math_rules: Vec<MathRule>,
    /// the mtext rules (math symbols)
    pub mtext_rules: Vec<MTextRule>,
    /// the sequence rules
    pub sequence_rules: Vec<SequenceRule>,

    /// matches names of word rules to their offsets
    pub word_rule_names: HashMap<String, usize>,
    /// matches names of POS rules to their offsets
    pub pos_rule_names: HashMap<String, usize>,
    /// matches names of math rules to their offsets
    pub math_rule_names: HashMap<String, usize>,
    /// matches names of mtext rules to their offsets
    pub mtext_rule_names: HashMap<String, usize>,
    /// matches names of sequence rules to their offsets
    pub sequence_rule_names: HashMap<String, usize>,
}



/*
 * IMPLEMENTATIONS FOR PATTERN LOADING
 */

impl MathPattern {
    /// loads a `MathPattern` from a node
    fn load_from_node(node : &Node, pctx : &mut PCtx) -> Result<MathPattern, String> {
        match node.get_name().as_ref() {
            "math_any" => {
                try!(assert_no_child(node));
                Ok(MathPattern::AnyMath)
            }
            "math_marker" => {
                Ok(MathPattern::Marked(
                        Box::new(try!(MathPattern::load_from_node(&try!(get_only_child(node)), pctx))),
                        try!(PatternMarker::load_from_node(node))))
            }
            "math_ref" => {
                try!(assert_no_child(node));
                let ref_str = try!(require_node_property(node, "ref"));
                Ok(MathPattern::MathRef(pctx.get_math_rule(&ref_str)))
            }
            "math_or" => {
                let mut options : Vec<MathPattern> = Vec::new();
                for cur in &try!(get_non_text_children(node)) {
                    options.push(try!(MathPattern::load_from_node(cur, pctx)));
                }
                Ok(MathPattern::MathOr(options))
            }
            "math_node" => {
                let node_name : Option<String> = node.get_property("name");
                let mut mtextref : Option<usize> = None;
                let mut children : Option<(Vec<MathPattern>, MathChildrenMatchType)> = None;
                for cur in &try!(get_non_text_children(node)) {
                    match cur.get_name().as_ref() {
                        "math_children" => {
                            if children.is_some() {
                                return Err("\"math_node\" had multiple children \"math_children\"".to_string());
                            }
                            let match_type = try!(MathChildrenMatchType::from_str(
                                                     &try!(require_node_property(cur, "match_type"))));
                            let mut child_nodes : Vec<MathPattern> = Vec::new();
                            for cur_cur in &try!(get_non_text_children(cur)) {
                                child_nodes.push(try!(MathPattern::load_from_node(cur_cur, pctx)));
                            }
                            if child_nodes.len() == 0 {
                                return Err("\"math_children\" is emty".to_string()); // would cause problems later
                            }
                            children = Some((child_nodes, match_type));
                        }
                        "mtext_ref" => {
                            if mtextref.is_some() {
                                return Err("\"math_node\" had multiple children \"mtext_ref\"".to_string());
                            }
                            try!(assert_no_child(cur));
                            let ref_str = try!(require_node_property(cur, "ref"));
                            mtextref = Some(pctx.get_mtext_rule(&ref_str));
                        }
                        other => {
                            return Err(format!("Expected \"mtext_ref\" or \"math_children\", but found \"{}\"",
                                               other));
                        }
                    }
                }
                Ok(MathPattern::MathNode(node_name, mtextref, children))
            }
            "math_descendant" => {
                let match_type = try!(MathDescendantMatchType::from_str(
                        &try!(require_node_property(node, "match_type"))));
                let child = Box::new(try!(MathPattern::load_from_node(&try!(get_only_child(node)), pctx)));
                Ok(MathPattern::MathDescendant(child, match_type))
                
            }
            unknown => Err(format!("Expected math node, found \"{}\"", unknown))
        }
    }
}


impl SequencePattern {
    /// loads a `SequencePattern` from a node
    fn load_from_node(node : &Node, pctx : &mut PCtx) -> Result<SequencePattern, String> {
        match node.get_name().as_ref() {
            "seq_ref" => {
                try!(assert_no_child(node));
                let ref_str = try!(require_node_property(node, "ref"));
                Ok(SequencePattern::SeqRef(pctx.get_sequence_rule(&ref_str)))
            }
            "seq_word" => {
                Ok(SequencePattern::SeqFromWord(try!(WordPattern::load_from_node(&try!(get_only_child(node)), pctx))))
            }
            "seq_seq" => {
                let mut elements : Vec<SequencePattern> = Vec::new();
                for cur in &try!(get_non_text_children(node)) {
                    elements.push(try!(SequencePattern::load_from_node(cur, pctx)));
                }
                Ok(SequencePattern::SeqOfSeq(elements))
            }
            "phrase" => {
                let tag_str : &str = &try!(require_node_property(node, "tag"));
                let mut match_type : PhraseMatchType = PhraseMatchType::Longest;  // TODO: Is this a good default?
                let mut start : Option<(Box<SequencePattern>, SequenceContainment)> = None;
                let mut end : Option<Box<SequencePattern>> = None;

                for cur in &try!(get_non_text_children(node)) {
                    match cur.get_name().as_ref() {
                        "match_type" => {
                            match_type = try!(PhraseMatchType::from_str(&try!(get_simple_node_content(cur, true))));
                        }
                        "starts_with_seq" => {
                            if start.is_some() {
                                return Err("Cannot have multipe start_with_seq nodes in a phrase node".to_string());
                            }
                            start = Some((Box::new(try!(SequencePattern::load_from_node(
                                                &try!(get_only_child(cur)),
                                                pctx))),
                                          try!(SequenceContainment::from_node(cur))));
                        }
                        "ends_with_seq" => {
                            if end.is_some() {
                                return Err("Cannot have multipe end_with_seq nodes in a phrase node".to_string());
                            }
                            end = Some(Box::new(try!(SequencePattern::load_from_node(
                                            &try!(get_only_child(cur)),
                                            pctx))));
                        }
                        unknown => {
                            return Err(format!("Unexpected node \"{}\" in phrase node", unknown));
                        }
                    }
                }
                let tag_opt = pctx.phrase_map.get(&tag_str);
                if tag_opt.is_none() {
                    return Err(format!("unknow Phrase type \"{}\"" ,tag_str));
                }
                Ok(SequencePattern::Phrase(tag_opt.unwrap().clone(), match_type, start, end))
            }
            "seq_marker" => {
                Ok(SequencePattern::Marked(Box::new(try!(SequencePattern::load_from_node(&try!(get_only_child(node)), pctx))),
                                           try!(PatternMarker::load_from_node(node))))
            }
            "seq_or" => {
                let match_type = match node.get_property("match_type") {
                    None => SequenceMatchType::First,
                    Some(ref m) => match m.as_ref() {
                        "first" => SequenceMatchType::First,
                        "atleastone" => SequenceMatchType::AtLeastOne,
                        "any" => SequenceMatchType::Any,
                        "longest" => SequenceMatchType::Longest,
                        unknown => { return Err(format!("Unknown sequence match_type \"{}\"", unknown)) }
                    }
                };
                let mut elements : Vec<SequencePattern> = Vec::new();
                for cur in &try!(get_non_text_children(node)) {
                    elements.push(try!(SequencePattern::load_from_node(cur, pctx)));
                }
                Ok(SequencePattern::SeqOr(elements, match_type))

            }
            unknown => Err(format!("Expected sequence node, found \"{}\"", unknown))
        }
    }
}



impl WordPattern {
    /// loads a `WordPattern` from a node
    fn load_from_node(node : &Node, pctx : &mut PCtx) -> Result<WordPattern, String> {
        match node.get_name().as_ref() {
            "word_ref" => {
                try!(assert_no_child(node));
                let ref_str = try!(require_node_property(node, "ref"));
                Ok(WordPattern::WordRef(pctx.get_word_rule(&ref_str)))
            }
            "word_or" => {
                let mut options : Vec<WordPattern> = Vec::new();
                for cur in &try!(get_non_text_children(node)) {
                    options.push(try!(WordPattern::load_from_node(cur, pctx)));
                }
                Ok(WordPattern::WordOr(options))
            }
            "word" => {
                Ok(WordPattern::Word(try!(get_simple_node_content(node, true))))
            }
            "word_math" => {
                Ok(WordPattern::MathWord(try!(MathPattern::load_from_node(&try!(get_only_child(node)),pctx))))
            }
            "word_any" => {
                Ok(WordPattern::AnyWord)
            }
            "word_not" => {
                Ok(WordPattern::WordNot(Box::new(try!(
                                WordPattern::load_from_node(&try!(get_only_child(node)), pctx)
                                ))))
            }
            "word_marker" => {
                Ok(WordPattern::Marked(Box::new(try!(WordPattern::load_from_node(&try!(get_only_child(node)), pctx))),
                                try!(PatternMarker::load_from_node(node))))
            }
            "word_pos" => {
                let mut word_pattern = None;
                let mut pos_pattern = None;

                for cur in &try!(get_non_text_children(node)) {
                    match cur.get_name().as_ref() {
                        "pos" => {
                            if pos_pattern.is_some() {
                                return Err("Cannot have multiple 'pos' nodes in a 'word_pos' node".to_string());
                            }
                            pos_pattern = Some(try!(PosPattern::load_from_node(cur, pctx)));
                        }
                        _ => {
                            if word_pattern.is_some() {
                                return Err("Cannot have multiple word pattern in a 'word_pos' node".to_string());
                            }
                            word_pattern = Some(try!(WordPattern::load_from_node(cur, pctx)));
                        }
                    }
                }

                if !word_pattern.is_some() {
                    return Err("'word_pos' node does not contain a word pattern".to_string());
                }

                if !pos_pattern.is_some() {
                    return Err("'word_pos' node does not contain a 'pos' node".to_string());
                }

                Ok(WordPattern::WordPos(pos_pattern.unwrap(), Box::new(word_pattern.unwrap())))
            }
            unknown => Err(format!("Expected word node, found \"{}\"", unknown))
        }
    }
}


impl PosPattern {
    /// loads a `PosPattern` from a node
    fn load_from_node(node : &Node, pctx : &mut PCtx) -> Result<PosPattern, String> {
        match node.get_name().as_ref() {
            "pos" => {
                try!(assert_no_child(node));
                let pos_str : &str = &try!(require_node_property(node, "tag"));
                match pctx.pos_map.get(pos_str) {
                    None => Err(format!("unknown POS tag \"{}\"", pos_str)),
                    Some(pos) => Ok(PosPattern::Pos(*pos))
                }
            }
            "pos_not" => {
                Ok(PosPattern::PosNot(Box::new(try!(
                                PosPattern::load_from_node(&try!(get_only_child(node)), pctx)
                                ))))
            }
            "pos_ref" => {
                try!(assert_no_child(node));
                let ref_str = try!(require_node_property(node, "ref"));
                Ok(PosPattern::PosRef(pctx.get_pos_rule(&ref_str)))
            }
            "pos_or" => {
                let mut options : Vec<PosPattern> = Vec::new();
                for cur in &try!(get_non_text_children(node)) {
                    options.push(try!(PosPattern::load_from_node(cur, pctx)));
                }
                Ok(PosPattern::PosOr(options))
            }
            unknown => Err(format!("Expected pos node, found \"{}\"", unknown))
        }
    }
}


impl MTextPattern {
    /// loads a `MTextPattern` from a node
    fn load_from_node(node : &Node, pctx : &mut PCtx) -> Result<MTextPattern, String> {
        match node.get_name().as_ref() {
            "mtext_any" => {
                try!(assert_no_child(node));
                Ok(MTextPattern::AnyMText)
            }
            "mtext_or" => {
                let mut options : Vec<MTextPattern> = Vec::new();
                for cur in &try!(get_non_text_children(node)) {
                    options.push(try!(MTextPattern::load_from_node(cur, pctx)));
                }
                Ok(MTextPattern::MTextOr(options))
            }
            "mtext_lit" => {
                try!(assert_no_child(node));
                let lit = try!(require_node_property(node, "str"));
                Ok(MTextPattern::MTextLit(lit))
            }
            "mtext_not" => {
                Ok(MTextPattern::MTextNot(Box::new(try!(MTextPattern::load_from_node(&try!(get_only_child(node)),
                                                                                     pctx)))))
            }
            "mtext_ref" => {
                try!(assert_no_child(node));
                let ref_str = try!(require_node_property(node, "ref"));
                Ok(MTextPattern::MTextRef(pctx.get_mtext_rule(&ref_str)))
            }
            unknown => Err(format!("Expected mtext node, found \"{}\"", unknown))
        }
    }
}

/*
 * IMPLEMENTATIONS FOR LOADING RULES
 */

/// Helper function: loads a rule
fn load_rule<PatternT, RuleT> (load_f: fn(&Node, &mut PCtx) -> Result<PatternT, String>,
                               node : &Node, pctx : &mut PCtx, rule_type : &str,
                               rule_gen : fn (PatternT, MetaDescription) -> RuleT) -> Result<RuleT, String> {
    let name = try!(require_node_property(node, "name"));
    let mut rule_opt : Option<PatternT> = None;
    let mut meta_opt : Option<MetaDescription> = None;
    for cur in &try!(get_non_text_children(node)) {
        match cur.get_name().as_ref() {
            "meta" => {
                if meta_opt.is_some() {
                    return Err(format!("{} \"{}\" has multiple meta nodes", rule_type,  &name));
                }
                meta_opt = Some(try!(MetaDescription::load_from_node(cur, name.clone())
                                     .map_err(|e| format!("error when loading meta node in word_rule \"{}\":\n{}",
                                                          &name, e))));
            }
            x => {
                if rule_opt.is_some() {
                    return Err(format!("Unexpected node \"{}\" in {} \"{}\"", x, rule_type, &name));
                }
                rule_opt = Some(try!(load_f(cur, pctx)
                                     .map_err(|e| format!("error when loading content node in {} \"{}\":\n{}",
                                                          rule_type, &name, e))));
            }
        }
    }
    if meta_opt.is_none() {
        meta_opt = Some(MetaDescription { name : name.clone(), summary : String::new() });
    }
    if rule_opt.is_none() {
        Err(format!("{} \"{}\" has no content node", rule_type, &name))
    } else {
        Ok(rule_gen(rule_opt.unwrap(), meta_opt.unwrap()))
    }
}



impl WordRule {
    /// loads a `WordRule` from a node
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<WordRule, String> {
        Ok(try!(load_rule(WordPattern::load_from_node, node, pctx, "word_rule", WordRule::generate_rule)))
    }

    /// creates a new rule from a pattern and a description
    fn generate_rule(pattern : WordPattern, description : MetaDescription) -> WordRule {
        WordRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl PosRule {
    /// loads a `PosRule` from a node
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<PosRule, String> {
        Ok(try!(load_rule(PosPattern::load_from_node, node, pctx, "pos_rule", PosRule::generate_rule)))
    }

    /// creates a new rule from a pattern and a description
    fn generate_rule(pattern : PosPattern, description : MetaDescription) -> PosRule {
        PosRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl MathRule {
    /// loads a `MathRule` from a node
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<MathRule, String> {
        Ok(try!(load_rule(MathPattern::load_from_node, node, pctx, "math_rule", MathRule::generate_rule)))
    }

    /// creates a new rule from a pattern and a description
    fn generate_rule(pattern : MathPattern, description : MetaDescription) -> MathRule {
        MathRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl MTextRule {
    /// loads a `MTextRule` from a node
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<MTextRule, String> {
        Ok(try!(load_rule(MTextPattern::load_from_node, node, pctx, "mtext_rule", MTextRule::generate_rule)))
    }

    /// creates a new rule from a pattern and a description
    fn generate_rule(pattern : MTextPattern, description : MetaDescription) -> MTextRule {
        MTextRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl SequenceRule {
    /// loads a `SequenceRule` from a node
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<SequenceRule, String> {
        Ok(try!(load_rule(SequencePattern::load_from_node, node, pctx, "seq_rule", SequenceRule::generate_rule)))
    }

    /// creates a new rule from a pattern and a description
    fn generate_rule(pattern : SequencePattern, description : MetaDescription) -> SequenceRule {
        SequenceRule {
            description : description,
            pattern : pattern,
        }
    }
}


/// gets the position (offset) for a rule. Allocates a new position, if rule doesn't have an offset
/// yet
fn get_rule_position<RuleT>(rules : &mut Vec<Option<RuleT>>, map : &mut HashMap<String, usize>, rule_name : &str) -> usize {
    {
        if let Some(position) = map.get(rule_name) {
            return *position
        }
    }
    let pos = rules.len();
    rules.push(None);
    map.insert(rule_name.to_string(), pos);
    pos
}


impl<'t> PCtx<'t> {
    fn new() -> PCtx<'t> {
        PCtx {
            pos_map : POS::generate_str_to_pos_map(),
            phrase_map : Phrase::generate_str_to_phrase_map(),
            word_rules : Vec::new(),
            seq_rules : Vec::new(),
            math_rules : Vec::new(),
            mtext_rules : Vec::new(),
            pos_rules : Vec::new(),
            word_name_map : HashMap::new(),
            seq_name_map : HashMap::new(),
            math_name_map : HashMap::new(),
            mtext_name_map : HashMap::new(),
            pos_name_map : HashMap::new(),
        }
    }

    fn get_math_rule(&mut self, rule_name : &str) -> usize {
        println!("Get math rule \"{}\"", rule_name);
        get_rule_position(&mut self.math_rules, &mut self.math_name_map, rule_name)
    }

    fn get_mtext_rule(&mut self, rule_name : &str) -> usize {
        println!("Get mtext rule \"{}\"", rule_name);
        get_rule_position(&mut self.mtext_rules, &mut self.mtext_name_map, rule_name)
    }
    
    fn get_pos_rule(&mut self, rule_name : &str) -> usize {
        println!("Get pos rule \"{}\"", rule_name);
        get_rule_position(&mut self.pos_rules, &mut self.pos_name_map, rule_name)
    }

    fn get_word_rule(&mut self, rule_name : &str) -> usize {
        println!("Get word rule \"{}\"", rule_name);
        get_rule_position(&mut self.word_rules, &mut self.word_name_map, rule_name)
    }

    fn get_sequence_rule(&mut self, rule_name : &str) -> usize {
        println!("Get seq rule \"{}\"", rule_name);
        get_rule_position(&mut self.seq_rules, &mut self.seq_name_map, rule_name)
    }

    fn add_math_rule(&mut self, node : &Node) -> Result<(), String> {
        let rule = try!(MathRule::load_from_node(node, self));
        let pos = self.get_math_rule(&rule.description.name);
        if self.math_rules[pos].is_some() {
            return Err(format!("Conflict: Multiple definitions of math_rule \"{}\"", &rule.description.name));
        }
        self.math_rules[pos] = Some(rule);
        Ok(())
    }

    fn add_mtext_rule(&mut self, node : &Node) -> Result<(), String> {
        let rule = try!(MTextRule::load_from_node(node, self));
        let pos = self.get_mtext_rule(&rule.description.name);
        if self.mtext_rules[pos].is_some() {
            return Err(format!("Conflict: Multiple definitions of mtext_rule \"{}\"", &rule.description.name));
        }
        self.mtext_rules[pos] = Some(rule);
        Ok(())
    }

    fn add_word_rule(&mut self, node : &Node) -> Result<(), String> {
        let rule = try!(WordRule::load_from_node(node, self));
        let pos = self.get_word_rule(&rule.description.name);
        if self.word_rules[pos].is_some() {
            return Err(format!("Conflict: Multiple definitions of word_rule \"{}\"", &rule.description.name));
        }
        self.word_rules[pos] = Some(rule);
        Ok(())
    }

    fn add_pos_rule(&mut self, node : &Node) -> Result<(), String> {
        let rule = try!(PosRule::load_from_node(node, self));
        let pos = self.get_pos_rule(&rule.description.name);
        if self.pos_rules[pos].is_some() {
            return Err(format!("Conflict: Multiple definitions of pos_rule \"{}\"", &rule.description.name));
        }
        self.pos_rules[pos] = Some(rule);
        Ok(())
    }

    fn add_sequence_rule(&mut self, node : &Node) -> Result<(), String> {
        let rule = try!(SequenceRule::load_from_node(node, self));
        let pos = self.get_sequence_rule(&rule.description.name);
        if self.seq_rules[pos].is_some() {
            return Err(format!("Conflict: Multiple definitions of seq_rule \"{}\"", &rule.description.name));
        }
        self.seq_rules[pos] = Some(rule);
        Ok(())
    }

    /// Verifies that all referenced rules have a definition
    fn verify(&self) -> Result<(), String> {
        for (name, pos) in &self.pos_name_map {
            if self.pos_rules[*pos].is_none() {
                return Err(format!("Couldn't find definition for pos_rule \"{}\"", name));
            }
        }
        for (name, pos) in &self.math_name_map {
            if self.math_rules[*pos].is_none() {
                return Err(format!("Couldn't find definition for math_rule \"{}\"", name));
            }
        }
        for (name, pos) in &self.mtext_name_map {
            if self.mtext_rules[*pos].is_none() {
                return Err(format!("Couldn't find definition for mtext_rule \"{}\"", name));
            }
        }
        for (name, pos) in &self.seq_name_map {
            if self.seq_rules[*pos].is_none() {
                return Err(format!("Couldn't find definition for seq_rule \"{}\"", name));
            }
        }
        for (name, pos) in &self.word_name_map {
            if self.word_rules[*pos].is_none() {
                return Err(format!("Couldn't find definition for word_rule \"{}\"", name));
            }
        }
        Ok(())
    }
}


impl PatternFile {
    /// loads a pattern file
    pub fn load(file_name : &str) -> Result<PatternFile, String> {
        let parser = Parser::default();
        let doc = try!(parser.parse_file(file_name).map_err(|_| format!("Failed to obtain DOM from \"{}\"", file_name)));
        // let root_node = try!(doc.get_root_element().map_err(|_| format!("\"{}\" has no root node", file_name)));
        let root_node = doc.get_root_element(); // try!(doc.get_root_element().map_err(|_| format!("\"{}\" has no root node", file_name)));
        let mut meta_opt : Option<MetaDescription> = None;
        let mut pctx = PCtx::new();

        let err_map = |e| format!("error when loading pattern file \"{}\":\n{}", file_name, e);

        for cur in &try!(get_non_text_children(&root_node)) {
            match cur.get_name().as_ref() {
                "meta" => {
                    if meta_opt.is_some() {
                        return Err("pattern_file has multiple meta nodes".to_string()).map_err(&err_map);
                    }
                    meta_opt = Some(try!(MetaDescription::load_from_node(cur, file_name.to_string())
                                         .map_err(&err_map)));
                }
                "pos_rule" => {
                    try!(pctx.add_pos_rule(cur).map_err(&err_map));
                }
                "math_rule" => {
                    try!(pctx.add_math_rule(cur).map_err(&err_map));
                }
                "mtext_rule" => {
                    try!(pctx.add_mtext_rule(cur).map_err(&err_map));
                }
                "word_rule" => {
                    try!(pctx.add_word_rule(cur).map_err(&err_map));
                }
                "seq_rule" => {
                    try!(pctx.add_sequence_rule(cur).map_err(&err_map));
                }
                x => {
                    return Err(format!("Unexpected node \"{}\" in pattern_file", x)).map_err(&err_map);
                }
            }
        }
        if meta_opt.is_none() {
            meta_opt = Some(MetaDescription { name : file_name.to_string(), summary : String::new() });
        }

        try!(pctx.verify().map_err(&err_map));

        return Ok(PatternFile {
            description : meta_opt.unwrap(),

            word_rules : pctx.word_rules.iter().map(|o| o.as_ref().unwrap().clone()).collect(),
            pos_rules : pctx.pos_rules.iter().map(|o| o.as_ref().unwrap().clone()).collect(),
            sequence_rules : pctx.seq_rules.iter().map(|o| o.as_ref().unwrap().clone()).collect(),
            math_rules : pctx.math_rules.iter().map(|o| o.as_ref().unwrap().clone()).collect(),
            mtext_rules : pctx.mtext_rules.iter().map(|o| o.as_ref().unwrap().clone()).collect(),

            word_rule_names : pctx.word_name_map,
            pos_rule_names : pctx.pos_name_map,
            sequence_rule_names : pctx.seq_name_map,
            math_rule_names : pctx.math_name_map,
            mtext_rule_names : pctx.mtext_name_map,
        });

    }

}

