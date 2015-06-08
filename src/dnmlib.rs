//! The `dnmlib` can be used for easier switching between the DOM
//! (Document Object Model) representation and the plain text representation,
//! which is needed for most NLP tools.

use rustlibxml::tree::*;
use std::collections::HashMap;

/// Specifies how to deal with a certain tag
pub enum SpecialTagsOption {
    /// Recurse into tag (default behaviour)
    Enter,
    /// Normalize tag, replacing it by some token
    Normalize(String),
    /// Skip tag
    Skip,
}


/// Paremeters for the DNM generation
pub struct DNMParameters {
    /// How to deal with special tags (e.g. `<math>` tags)
    pub special_tags_options: HashMap<String, SpecialTagsOption>,
    /// merge sequences of whitespaces into a single ' '.
    /// *Doesn't affect tokens*
    pub normalize_white_spaces: bool,
    /// put spaces before and after tokens
    pub wrap_tokens: bool,
}

impl Default for DNMParameters {
    fn default() -> DNMParameters {
        DNMParameters {
            special_tags_options : HashMap::new(),
            normalize_white_spaces: true,
            wrap_tokens: true,
        }
    }
}

/// The `DNM` is essentially a wrapper around the plain text representation
/// of the document, which facilitates mapping plaintext pieces to the DOM.
/// This breaks, if the DOM is changed after the DNM generation!
pub struct DNM {
    /// The plaintext
    pub plaintext : String,
    /// The options for generation
    pub parameters : DNMParameters,
    /// The root node of the underlying xml tree
    pub root_node : XmlNodeRef,
    /// Maps nodes to plaintext offsets
    pub node_map : HashMap<XmlNodeRef, (usize, usize)>,
    //pub node_map : HashMap<libc::c_void, (usize, usize)>,
}

/// Some temporary data for the parser
struct TmpParseData {
    /// plaintext is currently terminated by some whitespace
    had_whitespace : bool,
}

fn recursive_dnm_generation(dnm: &mut DNM, root: XmlNodeRef,
                            tmp: &mut TmpParseData) {
    let offset_start = dnm.plaintext.len();

    if root.is_text_node() {
        if dnm.parameters.normalize_white_spaces {
            for c in root.get_content().chars() {
                if c.is_whitespace() {
                    if tmp.had_whitespace { continue; }
                    dnm.plaintext.push(' ');
                    tmp.had_whitespace = true;
                }
                else {
                    dnm.plaintext.push(c);
                    tmp.had_whitespace = false;
                }
            }
        } else {
            dnm.plaintext.push_str(&root.get_content());
        }
        dnm.node_map.insert(root, (offset_start, dnm.plaintext.len()));
        return;
    }
    let name : String = root.get_name();
    {
        let rule = dnm.parameters.special_tags_options.get(&name);
        match rule {
            Some(&SpecialTagsOption::Enter) => {

            }
            Some(&SpecialTagsOption::Normalize(ref token)) => {
                if dnm.parameters.wrap_tokens {
                    if !tmp.had_whitespace ||
                       !dnm.parameters.normalize_white_spaces {
                        dnm.plaintext.push(' ');
                    }
                    dnm.plaintext.push_str(&token);
                    dnm.plaintext.push(' ');
                    tmp.had_whitespace = true;
                } else {
                    dnm.plaintext.push_str(&token);
                    //tokens are considered non-whitespace
                    tmp.had_whitespace = false;
                }
                dnm.node_map.insert(root,
                                    (offset_start, dnm.plaintext.len()));
                return;
            }
            Some(&SpecialTagsOption::Skip) => {
                dnm.node_map.insert(root,
                                    (offset_start, dnm.plaintext.len()));
                return;
            }
            None => {

            }
        }
    }

    let mut child_option = root.get_first_child();
    loop {
        match child_option {
            Some(child) => {
                recursive_dnm_generation(dnm, child, tmp);
                child_option = child.get_next_sibling();
            }
            None => break,
        }
    }
    dnm.node_map.insert(root, (offset_start, dnm.plaintext.len()));
}

/// Very often we'll talk about substrings of the plaintext - words, sentences,
/// etc. A `DNMRange` stores start and end point of such a substring and has
/// a reference to the `DNM`.
pub struct DNMRange <'a> {
    start : usize,
    end : usize,
    dnm : &'a DNM,
}

impl <'a> DNMRange <'a> {
    /// Get the plaintext substring corresponding to the range
    pub fn get_plaintext(&self) -> String {
        self.dnm.plaintext.slice_chars(self.start, self.end).to_owned()
    }
}


impl DNM {
    /// Creates a `DNM` for `root`
    pub fn create_dnm(root: XmlNodeRef, parameters: DNMParameters) -> DNM {
        let mut dnm = DNM {
            plaintext : String::new(),
            parameters : parameters,
            root_node : root,
            node_map : HashMap::new(),
        };

        let mut tmp = TmpParseData {
            had_whitespace : true,  //no need for leading whitespaces
        };

        recursive_dnm_generation(&mut dnm, root, &mut tmp);

        dnm
    }

    /// Get the plaintext range of a node
    pub fn get_range_of_node (self : &DNM, node: XmlNodeRef)
                                    -> Result<DNMRange, ()> {
        match self.node_map.get(&node) {
            Some(&(start, end)) => Ok(DNMRange
                                     {start:start, end:end, dnm: self}),
            None => Err(()),
        }
    }
}

