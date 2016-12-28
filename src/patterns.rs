use libxml::tree::*;

use libxml::parser::Parser;

use senna::pos::POS;
use senna::phrase::Phrase;
use senna::sentence::*;

use std::collections::HashMap;

use dnm::*;




#[derive(Clone)]
pub struct PatternMarker {
    pub name: String,
    pub tags: Vec<String>,
}



#[derive(Clone)]
pub struct MathMarker {
    pub node : Node,
    pub marker : PatternMarker,
}

#[derive(Clone)]
pub struct TextMarker<'t> {
    pub range : DNMRange<'t>,
    pub marker : PatternMarker,
}

#[derive(Clone)]
pub enum MarkerEnum<'t> {
    Text(TextMarker<'t>),
    Math(MathMarker),

}

#[derive(Clone)]
pub struct Match<'t> {
    pub marker : MarkerEnum<'t>,
    pub sub_matches : Vec<Match<'t>>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum MathChildrenMatchType {
    StartsWith,
    MatchesExactly,
    EndsWith,
    Arbitrary
}

impl MathChildrenMatchType {
    pub fn from_str(string : &str) -> Result<MathChildrenMatchType, String> {
        match string {
            "starts_with" => Ok(MathChildrenMatchType::StartsWith),
            "exact" => Ok(MathChildrenMatchType::MatchesExactly),
            "ends_with" => Ok(MathChildrenMatchType::EndsWith),
            "arbitrary" => Ok(MathChildrenMatchType::Arbitrary),
            other => Err(format!("Unknown match_type for match_children \"{}\"", other)),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum MathDescendantMatchType {
    First,
    AtLeastOne,
    Arbitrary
}

impl MathDescendantMatchType {
    pub fn from_str(string : &str) -> Result<MathDescendantMatchType, String> {
        match string {
            "first" => Ok(MathDescendantMatchType::First),
            "at_least_one" => Ok(MathDescendantMatchType::AtLeastOne),
            "arbitrary" => Ok(MathDescendantMatchType::Arbitrary),
            other => Err(format!("Unknon match_type for math_descendant \"{}\"", other)),
        }
    }
}


#[derive(Clone)]
pub enum MathPattern {   // matches always exactly one node!
    AnyMath,
    MathRef(usize),
    Marked(Box<MathPattern>, PatternMarker),
    MathOr(Vec<MathPattern>),
    MathNode(Option<String> /* node name */,
             Option<usize> /* m_text_ref */,
             Option<(Vec<MathPattern>, MathChildrenMatchType)> /*children */),
    MathDescendant(Box<MathPattern>, MathDescendantMatchType),
}

#[derive(Clone)]
pub enum MTextPattern {
    AnyMText,
    MTextOr(Vec<MTextPattern>),
    MTextLit(String),
    MTextNot(Box<MTextPattern>),
    MTextRef(usize),
}


#[derive(Clone)]
pub enum PosPattern {
    Pos(POS),
    PosNot(Box<PosPattern>),
    PosOr(Vec<PosPattern>),
    PosRef(usize),
}

#[derive(Clone)]
pub enum WordPattern {
    WordRef(usize),
    WordOr(Vec<WordPattern>),
    Word(String),
    WordPos(usize, Box<WordPattern>),
    MathWord(MathPattern),
    AnyWord,
    WordNot(Box<WordPattern>),
    Marked(Box<WordPattern>, PatternMarker),
}


#[derive(Clone, PartialEq, Eq)]
pub enum PhraseMatchType {
    Shortest,
    Longest
}

#[derive(Clone, PartialEq, Eq)]
pub enum SequenceContainment {
    LessOrEqual,
    Any,
}

#[derive(Clone, PartialEq, Eq)]
pub enum SequenceMatchType {
    First,
    AtLeastOne,
    Any,
    Longest,
}

#[derive(Clone)]
pub enum SequencePattern {
    SeqRef(usize),
    SeqFromWord(WordPattern),
    SeqOfSeq(Vec<SequencePattern>),
    Phrase(Phrase, PhraseMatchType,
           Option<(Box<SequencePattern>, SequenceContainment)>,
           Option<Box<SequencePattern>>),
    Marked(Box<SequencePattern>, PatternMarker),
    SeqOr(Vec<SequencePattern>, SequenceMatchType),
}


#[derive(Clone)]
pub struct MetaDescription {
    pub name: String,
    pub summary: String,
    pub author: String,
}


#[derive(Clone)]
pub struct WordRule {
    pub description: MetaDescription,
    pub pattern: WordPattern,
}

#[derive(Clone)]
pub struct PosRule {
    pub description: MetaDescription,
    pub pattern: PosPattern,
}

#[derive(Clone)]
pub struct MathRule {
    pub description: MetaDescription,
    pub pattern: MathPattern,
}

#[derive(Clone)]
pub struct MTextRule {
    pub description: MetaDescription,
    pub pattern: MTextPattern,
}

#[derive(Clone)]
pub struct SequenceRule {
    pub description: MetaDescription,
    pub pattern: SequencePattern,
}


struct PCtx<'t> {  // pattern loading context
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


pub struct PatternFile {
    pub description: MetaDescription,

    pub word_rules: Vec<WordRule>,
    pub pos_rules: Vec<PosRule>,
    pub math_rules: Vec<MathRule>,
    pub mtext_rules: Vec<MTextRule>,
    pub sequence_rules: Vec<SequenceRule>,

    pub word_rule_names: HashMap<String, usize>,
    pub pos_rule_names: HashMap<String, usize>,
    pub math_rule_names: HashMap<String, usize>,
    pub mtext_rule_names: HashMap<String, usize>,
    pub sequence_rule_names: HashMap<String, usize>,
}


fn is_comment_node(node : &Node) -> bool {
    node.get_type().unwrap() == NodeType::CommentNode
}

fn get_simple_node_content(node : &Node, trim: bool) -> Result<String, String> {
    let child = node.get_first_child();
    if child.is_none() {
        Ok(String::new())
    } else if !child.as_ref().unwrap().is_text_node() || child.as_ref().unwrap().get_next_sibling().is_some() {
        Err(format!("found unexpected nodes in node \"{}\"", node.get_name()))
    } else {
        Ok(if trim { child.as_ref().unwrap().get_content().trim().to_string() } else { child.as_ref().unwrap().get_content() } )
    }
}

fn get_non_text_children(node : &Node) -> Result<Vec<Node>, String> {
    let mut cur = node.get_first_child();
    let mut children : Vec<Node> = Vec::new();

    loop {
        if cur.is_none() {
            return Ok(children);
        }

        let cur_ = cur.unwrap();

        if cur_.is_text_node() {
            if !cur_.get_content().trim().is_empty() {
                return Err(format!("found unexpected text in \"{}\" node: \"{}\"",
                                   node.get_name(), cur_.get_content().trim()));
            }
        } else if !is_comment_node(&cur_) {
            children.push(cur_.clone());
        }
        cur = cur_.get_next_sibling();
    }
}

fn fast_get_non_text_children(node : &Node) -> Vec<Node> {
    let mut cur = node.get_first_child();
    let mut children : Vec<Node> = Vec::new();
    loop {
        if cur.is_none() { return children; }
        let cur_ = cur.unwrap();
        if !cur_.is_text_node() && !is_comment_node(&cur_) { children.push(cur_.clone()); }
        cur = cur_.get_next_sibling();
    }
}

fn get_only_child(node : &Node) -> Result<Node, String> {
    let children = try!(get_non_text_children(node));
    if children.len() < 1 {
        Err(format!("Expected child node in node \"{}\"", node.get_name()))
    } else if children.len() > 1 {
        Err(format!("Too many child nodes in node \"{}\"", node.get_name()))
    } else {
        Ok(children[0].clone())
    }
}

fn check_found_property_already(property: &Option<String>,
                                node_name: &str,
                                parent_name: &str) -> Result<(), String> {
    if property.is_some() {
        Err(format!("found multiple \"{}\" nodes in \"{}\" node", node_name, parent_name))
    } else {
        Ok(())
    }
}

fn assert_no_child(node : &Node) -> Result<(), String> {
    if try!(get_non_text_children(node)).is_empty() {
        Ok(())
    } else {
        Err(format!("Found unexpected child of node \"{}\"", node.get_name()))
    }
}

impl MetaDescription {
    fn load_from_node(node : &Node, name: String) -> Result<MetaDescription, String> {
        if node.get_name().ne("meta") {
            return Err(format!("expected meta node, found \"{}\"", &node.get_name()));
        }

        let mut summary : Option<String> = None;
        let mut author : Option<String> = None;

        for cur in &try!(get_non_text_children(node)) {
            match cur.get_name().as_ref() {
                "description" => {
                    try!(check_found_property_already(&summary, "description", "meta"));
                    summary = Some(try!(
                            get_simple_node_content(&cur, true)
                            .map_err(|e| format!("error in meta node:\n{}", e))));
                }
                "author" => {
                    try!(check_found_property_already(&author, "author", "meta"));
                    author = Some(try!(
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
            author: author.unwrap_or_else(|| String::new()),
        })
    }
}

fn require_node_property(node : &Node, property : &str) -> Result<String, String> {
    match node.get_property(property) {
        None => Err(format!("\"{}\" node misses \"{}\" property", node.get_name(), property)),
        Some(value) => Ok(value)
    }

}

impl PatternMarker {
    fn load_from_node(node : &Node) -> Result<PatternMarker, String> {
        let name = try!(require_node_property(node, "name"));
        let tags = match node.get_property("tags") {   // TODO: Add regex: [a-zA-Z0-9_]+(,[a-zA-Z0-9_]+)*
            None => Vec::new(),
            Some(value) => value.split(",").map(|s| s.trim().to_string()).collect(),
        };
        return Ok(PatternMarker { name : name, tags : tags });
    }
}

impl MTextPattern {
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

impl MathPattern {
    fn load_from_node(node : &Node, pctx : &mut PCtx) -> Result<MathPattern, String> {
        match node.get_name().as_ref() {
            "math_any" => {
                try!(assert_no_child(node));
                Ok(MathPattern::AnyMath)
            }
            "math_marker" => {
                Ok(MathPattern::Marked(Box::new(try!(MathPattern::load_from_node(&try!(get_only_child(node)), pctx))),
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

impl PosPattern {
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



impl WordPattern {
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
                let ref_str = try!(require_node_property(node, "ref"));
                Ok(WordPattern::WordPos(pctx.get_pos_rule(&ref_str),
                                        Box::new(try!(WordPattern::load_from_node(&try!(get_only_child(node)), pctx)))))

            }
            unknown => Err(format!("Expected word node, found \"{}\"", unknown))
        }
    }
}


fn get_sequence_containment(node : &Node) -> Result<SequenceContainment, String> {
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


impl SequencePattern {
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
                            match try!(get_simple_node_content(cur, true)).as_ref() {
                                "shortest" => {
                                    match_type = PhraseMatchType::Shortest;
                                }
                                "longest" => {
                                    match_type = PhraseMatchType::Longest;
                                }
                                unknown => {
                                    return Err(format!("Unknown match_type \"{}\"", unknown));
                                }
                            }
                        }
                        "starts_with_seq" => {
                            if start.is_some() {
                                return Err("Cannot have multipe start_with_seq nodes in a phrase node".to_string());
                            }
                            start = Some((Box::new(try!(SequencePattern::load_from_node(&try!(get_only_child(cur)), pctx))),
                                          try!(get_sequence_containment(cur))));
                        }
                        "ends_with_seq" => {
                            if end.is_some() {
                                return Err("Cannot have multipe end_with_seq nodes in a phrase node".to_string());
                            }
                            end = Some(Box::new(try!(SequencePattern::load_from_node(&try!(get_only_child(cur)), pctx))));
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
        meta_opt = Some(MetaDescription { name : name.clone(), summary : String::new(), author : String::new() });
    }
    if rule_opt.is_none() {
        Err(format!("{} \"{}\" has no content node", rule_type, &name))
    } else {
        Ok(rule_gen(rule_opt.unwrap(), meta_opt.unwrap()))
    }
}


impl WordRule {
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<WordRule, String> {
        Ok(try!(load_rule(WordPattern::load_from_node, node, pctx, "word_rule", WordRule::generate_rule)))
    }

    fn generate_rule(pattern : WordPattern, description : MetaDescription) -> WordRule {
        WordRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl PosRule {
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<PosRule, String> {
        Ok(try!(load_rule(PosPattern::load_from_node, node, pctx, "pos_rule", PosRule::generate_rule)))
    }

    fn generate_rule(pattern : PosPattern, description : MetaDescription) -> PosRule {
        PosRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl MathRule {
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<MathRule, String> {
        Ok(try!(load_rule(MathPattern::load_from_node, node, pctx, "math_rule", MathRule::generate_rule)))
    }

    fn generate_rule(pattern : MathPattern, description : MetaDescription) -> MathRule {
        MathRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl MTextRule {
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<MTextRule, String> {
        Ok(try!(load_rule(MTextPattern::load_from_node, node, pctx, "mtext_rule", MTextRule::generate_rule)))
    }

    fn generate_rule(pattern : MTextPattern, description : MetaDescription) -> MTextRule {
        MTextRule {
            description : description,
            pattern : pattern,
        }
    }
}

impl SequenceRule {
    fn load_from_node(node: &Node, pctx: &mut PCtx) -> Result<SequenceRule, String> {
        Ok(try!(load_rule(SequencePattern::load_from_node, node, pctx, "seq_rule", SequenceRule::generate_rule)))
    }

    fn generate_rule(pattern : SequencePattern, description : MetaDescription) -> SequenceRule {
        SequenceRule {
            description : description,
            pattern : pattern,
        }
    }
}


fn get_rule_position<RuleT>(rules : &mut Vec<Option<RuleT>>, map : &mut HashMap<String, usize>, rule_name : &str) -> usize {
    {
        let o = map.get(rule_name);
        if o.is_some() {
            return *o.unwrap();
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
    pub fn load(file_name : &str) -> Result<PatternFile, String> {
        let parser = Parser::default();
        let doc = try!(parser.parse_file(file_name).map_err(|_| format!("Failed to obtain DOM from \"{}\"", file_name)));
        let root_node = try!(doc.get_root_element().map_err(|_| format!("\"{}\" has no root node", file_name)));
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
            meta_opt = Some(MetaDescription { name : file_name.to_string(), summary : String::new(), author : String::new() });
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

    pub fn match_sentence<'t>(&self, sentence : &Sentence, range : &'t DNMRange, rule : &str)
                                                            -> Result<Vec<Match<'t>>, String> {
        if !range.dnm.parameters.support_back_mapping {
            return Err("DNM of sentence does not support back mapping".to_string());
        }

        let rule_pos = try!(self.sequence_rule_names.get(rule)
                            .ok_or(format!("Could not find sequence rule \"{}\"", rule)));
        let actual_rule = &self.sequence_rules[*rule_pos];
        let words = sentence.get_words();
        let psg = try!(sentence.get_psgroot().ok_or("PSG required for pattern matching".to_string()));
        let phraseTree = try!(PhraseTree::from_psg(psg).map_err(|_| "Invalid PSG: Contains only leaf".to_string()));

        let mut matches : Vec<Match<'t>> = Vec::new();

        for start_pos in 0..words.len() {
            let m = self.match_seq(&actual_rule.pattern, sentence, &phraseTree, range, start_pos);
            matches.extend_from_slice(&m._matches[..]);
        }

        Ok((matches))
    }

    fn match_seq<'t>(&self, rule : &SequencePattern, sentence : &Sentence, phraseTree : &PhraseTree,
                     range : &'t DNMRange, pos : usize) -> InternalSeqMatch<'t> {
        match rule {
            &SequencePattern::SeqRef(p) =>
                self.match_seq(&self.sequence_rules[p].pattern, sentence, phraseTree, range, pos),
            &SequencePattern::SeqFromWord(ref wp) => {
                if pos >= sentence.get_words().len() {
                    return InternalSeqMatch::no_match();
                }
                let m = self.match_word(&wp, &sentence.get_words()[pos], range);
                if !m.matched {
                    return InternalSeqMatch::no_match();
                }
                InternalSeqMatch {
                    _matches : m._matches,
                    start : pos,
                    end : pos + 1,
                    matched : true,
                }
            }
            &SequencePattern::SeqOfSeq(ref patterns) => {
                let mut matches : Vec<Match> = Vec::new();
                let mut cur_pos = pos;
                for p in patterns {
                    let m = self.match_seq(p, sentence, phraseTree, range, cur_pos);
                    if !m.matched { return InternalSeqMatch::no_match(); }
                    cur_pos = m.end;
                    matches.extend_from_slice(&m._matches[..]);
                }
                InternalSeqMatch {
                    _matches : matches,
                    start : pos,
                    end : cur_pos,
                    matched : true,
                }
            }
            &SequencePattern::Phrase(phrase, ref match_type, ref start_condition, ref end_condition) => {
                let mut phrase_ends = PatternFile::get_phrase_matches(phraseTree, pos, phrase);
                if match_type == &PhraseMatchType::Shortest {
                    phrase_ends.reverse();   // were sorted descendingly
                }

                let mut matches : Vec<Match> = Vec::new();

                for end_pos in &phrase_ends {
                    matches.clear();
                    if start_condition.is_some() {
                        // check start_condition
                        let &(box ref rule, ref containment) = start_condition.as_ref().unwrap();
                        let m = self.match_seq(rule, sentence, phraseTree, range, pos);
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
                            let m = self.match_seq(rule, sentence, phraseTree, range, end_start_pos);
                            if !m.matched {continue; }
                            matched = true;
                            matches.extend_from_slice(&m._matches[..]);
                        }
                        if !matched { continue; }
                    }
                    return InternalSeqMatch {
                        _matches : matches,
                        start : pos,
                        end : *end_pos,
                        matched : true,
                    };
                }

                // no phrase match satisfied all conditions:
                InternalSeqMatch::no_match()
            }
            &SequencePattern::Marked(box ref pattern, ref marker) => {
                let mut m = self.match_seq(pattern, sentence, phraseTree, range, pos);
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
                        start : pos,
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
                    let m = self.match_seq(p, sentence, phraseTree, range, pos);
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
                        start : pos,
                        end : longest,
                        matched : true,   // don't use `matched` (because SequenceMatchType::Any matches always)
                    }
                }
            }
        }
    }

    fn get_phrase_matches(root : &PhraseTree, start_pos : usize, target : Phrase) -> Vec<usize> {
        if root.start == start_pos {
            let mut v = if root.children.len() > 0 {
                    PatternFile::get_phrase_matches(&root.children[0], start_pos, target)
                } else { Vec::new() };
            if root.phrase == target {
                v.push(root.end + 1);
            }
            v
        } else if root.start < start_pos {
            for child in &root.children {
                if child.start <= start_pos && child.end >= start_pos {
                    return PatternFile::get_phrase_matches(child, start_pos, target);
                }
            }
            Vec::new()
        } else {
            Vec::new()
        }
    }

    fn match_word<'t>(&self, rule : &WordPattern, word : &Word, range : &'t DNMRange) -> InternalWordMatch<'t> {
        match rule {
            &WordPattern::WordRef(rule_pos) => self.match_word(&self.word_rules[rule_pos].pattern, word, range),
            &WordPattern::WordOr(ref word_patterns) => {
                for pattern in word_patterns {
                    let m = self.match_word(&pattern, word, range);
                    if m.matched {
                        return m;
                    }
                }
                InternalWordMatch { _matches : Vec::new(), matched : false }
            }
            &WordPattern::Word(ref word_str) => {
                if word_str == word.get_string() {
                    InternalWordMatch { _matches : Vec::new(), matched : true }
                } else {
                    InternalWordMatch { _matches : Vec::new(), matched : false }
                }
            }
            &WordPattern::WordPos(rule_pos, box ref word_pattern) => {
                if self.match_pos(&self.pos_rules[rule_pos].pattern, word.get_pos()) {
                    self.match_word(&word_pattern, word, range)
                } else {
                    InternalWordMatch { _matches : Vec::new(), matched : false }
                }
            }
            &WordPattern::MathWord(ref math_pattern) => {
                let node = range.dnm.offset_to_node[range.start + word.get_offset_start()].clone();
                if node.get_name() != "math" {
                    return InternalWordMatch { _matches : Vec::new(), matched : false };
                }
                let children = fast_get_non_text_children(&node);
                if children.len() == 0 {
                    return InternalWordMatch { _matches : Vec::new(), matched : false };
                }
                
                // TODO: Make the following code work in the general case!!!
                let mut m : InternalMathMatch;
                if children[0].get_name() == "semantics" {
                    let c = fast_get_non_text_children(&children[0]);
                    if c.len() == 0 {
                        return InternalWordMatch { _matches : Vec::new(), matched : false };
                    }
                    m = self.match_math(&math_pattern, &c[0]);
                } else {
                    m = self.match_math(&math_pattern, &children[0]);
                }
                if m.matched {
                    InternalWordMatch {
                        _matches : m._matches,
                        matched : true,
                    }
                } else {
                    InternalWordMatch {
                        _matches : Vec::new(),
                        matched : false,
                    }
                }

            }
            &WordPattern::AnyWord => InternalWordMatch { _matches : Vec::new(), matched : true },
            &WordPattern::WordNot(box ref p) => {
                let m = self.match_word(&p, word, range);
                if m.matched { m } else { InternalWordMatch { _matches : Vec::new(), matched : false } }
            }
            &WordPattern::Marked(box ref p, ref marker) => {
                let m = self.match_word(&p, word, range);
                if m.matched {
                    InternalWordMatch {
                        _matches : vec![ Match {
                            marker : MarkerEnum::Text(TextMarker {
                                range : range.get_subrange(word.get_offset_start(), word.get_offset_end()),
                                marker : marker.clone() }),
                            sub_matches : m._matches}],
                        matched : true,
                    }
                } else { InternalWordMatch { _matches : Vec::new(), matched : false } }
            }
        }
    }

    fn match_math<'t>(&self, rule : &MathPattern, node : &Node) -> InternalMathMatch<'t> {
        match rule {
            &MathPattern::AnyMath => InternalMathMatch { _matches : Vec::new(), matched : true, },
            &MathPattern::MathRef(o) => self.match_math(&self.math_rules[o].pattern, node),
            &MathPattern::Marked(box ref pattern, ref marker) => {
                let m = self.match_math(&pattern, node);
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
                    let m = self.match_math(pattern, node);
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
                    if !self.match_mtext(&self.mtext_rules[mtext.unwrap()].pattern, &content.unwrap()) {
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
                            let m = self.match_math(&child_rules[i], &c_nodes[i]);
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
                        if c_nodes.len() < child_rules.len() { break; }
                    }
                    return InternalMathMatch::no_match();
                }

                return InternalMathMatch { _matches : Vec::new(), matched : true };  // no child matches required
            }
            &MathPattern::MathDescendant(box ref pattern, ref match_type) => {
                let mut matches : Vec<Match> = Vec::new();
                let mut matched = false;
                let m = self.match_math(&pattern, node);
                if m.matched {
                    matched = true;
                    matches.extend_from_slice(&m._matches[..]);
                }
                for child in &fast_get_non_text_children(node) {
                    if matched && match_type == &MathDescendantMatchType::First {
                        return InternalMathMatch { _matches : matches, matched : true };
                    }
                    let m = self.match_math(&pattern, &child);
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

    fn match_mtext(&self, rule : &MTextPattern, string : &str) -> bool {
        match rule {
            &MTextPattern::AnyMText => true,
            &MTextPattern::MTextOr(ref ps) => ps.into_iter().any(|p| self.match_mtext(&p, string)),
            &MTextPattern::MTextLit(ref s) => s == string,
            &MTextPattern::MTextNot(box ref p) => !self.match_mtext(&p, string),
            &MTextPattern::MTextRef(o) => self.match_mtext(&self.mtext_rules[o].pattern, string),
        }
    }

    fn match_pos(&self, rule : &PosPattern, pos : POS) -> bool {
        match rule {
            &PosPattern::Pos(p) => p == pos,
            &PosPattern::PosNot(box ref r) => !self.match_pos(&r, pos),
            &PosPattern::PosOr(ref ps) => ps.into_iter().any(|p| self.match_pos(&p, pos)),
            &PosPattern::PosRef(o) => self.match_pos(&self.pos_rules[o].pattern, pos),
        }
    }

}

struct InternalMathMatch<'t> {
    _matches : Vec<Match<'t>>,
    matched : bool,
}

impl<'t> InternalMathMatch<'t> {
    fn no_match() -> InternalMathMatch<'t> {
        InternalMathMatch {
            _matches : Vec::new(),
            matched : false,
        }
    }
}

struct InternalSeqMatch<'t> {
    _matches : Vec<Match<'t>>,
    start : usize,
    end : usize,
    matched : bool,
}


impl<'t> InternalSeqMatch<'t> {
    fn no_match() -> InternalSeqMatch<'t> {
        InternalSeqMatch {
            _matches : Vec::new(),
            start : 0,
            end : 0,
            matched : false,
        }
    }
}


struct InternalWordMatch<'t> {
    _matches : Vec<Match<'t>>,
    matched : bool,
}



struct PhraseTree {
    phrase : Phrase,
    start : usize,
    end : usize,
    children : Vec<PhraseTree>,
}

impl PhraseTree {
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







