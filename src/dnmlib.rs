//! The `dnmlib` can be used for easier switching between the DOM
//! (Document Object Model) representation and the plain text representation,
//! which is needed for most NLP tools.

extern crate libc;
extern crate unidecode;
extern crate rustmorpha;

use rustlibxml::tree::*;
use std::collections::HashMap;
use std::mem;
use unidecode::unidecode;
use std::io::Write;

/// Print error message to stderr
/// from http://stackoverflow.com/questions/27588416/how-to-send-output-to-stderr
macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


/// Specifies how to deal with a certain tag
pub enum SpecialTagsOption {
    /// Recurse into tag (default behaviour)
    Enter,
    /// Normalize tag, replacing it by some token
    Normalize(String),
    /// Normalize tag, obtain replacement string by function call
    //FunctionNormalize(|&XmlNodeRef| -> String),
    FunctionNormalize(fn(&XmlNodeRef) -> String),
    /// Skip tag
    Skip,
}


/// Paremeters for the DNM generation
pub struct DNMParameters {
    /// How to deal with special tags (e.g. `<math>` tags)
    pub special_tag_name_options: HashMap<String, SpecialTagsOption>,
    /// How to deal with tags with special class names (e.g. ltx_note_mark)
    /// *Remark*: If both a tag name and a tag class match, the tag name rule
    /// will be applied.
    pub special_tag_class_options: HashMap<String, SpecialTagsOption>,
    /// merge sequences of whitespaces into a single ' '.
    /// *Doesn't affect tokens*
    pub normalize_white_spaces: bool,
    /// put spaces before and after tokens
    pub wrap_tokens: bool,
    /// if there is a trailing white space in a tag, don't make it part
    /// of that tag. Requires `normalize_white_spaces` to be set.
    pub move_whitespaces_between_nodes: bool,
    /// Replace unicode characters by the ascii code representation
    pub normalize_unicode: bool,
    /// Apply the morpha stemmer once to the text nodes
    pub stem_words_once: bool,
    /// Apply the morpha stemmer to the text nodes
    /// as often as it changes something
    pub stem_words_full: bool,
    /// Move to lowercase (remark: The stemmer does automatically)
    pub convert_to_lowercase: bool,
}

impl Default for DNMParameters {
    /// Don't do anything fancy and specific by default
    fn default() -> DNMParameters {
        DNMParameters {
            special_tag_name_options : HashMap::new(),
            special_tag_class_options : HashMap::new(),
            normalize_white_spaces: true,
            wrap_tokens: false,
            move_whitespaces_between_nodes: false,
            normalize_unicode: false,
            stem_words_once: false,
            stem_words_full: false,
            convert_to_lowercase: false
        }
    }
}

impl DNMParameters {
    /// Normalize in a reasonable way for our math documents
    pub fn llamapun_normalization() -> DNMParameters {
        let mut name_options = HashMap::new();
        name_options.insert("math".to_string(), SpecialTagsOption::Normalize("MathFormula".to_string()));
        name_options.insert("cite".to_string(), SpecialTagsOption::Normalize("CiteExpression".to_string()));
        name_options.insert("table".to_string(), SpecialTagsOption::Skip);
        name_options.insert("head".to_string(), SpecialTagsOption::Skip);
        let mut class_options = HashMap::new();
        class_options.insert("ltx_equation".to_string(), SpecialTagsOption::Normalize("MathFormula".to_string()));
        class_options.insert("ltx_equationgroup".to_string(), SpecialTagsOption::Normalize("MathFormula".to_string()));
        class_options.insert("ltx_note_mark".to_string(), SpecialTagsOption::Skip);
        class_options.insert("ltx_note_outer".to_string(), SpecialTagsOption::Skip);
        class_options.insert("ltx_bibliography".to_string(), SpecialTagsOption::Skip);

        DNMParameters {
            special_tag_name_options : name_options,
            special_tag_class_options : class_options,
            normalize_white_spaces : true,
            wrap_tokens : false,
            move_whitespaces_between_nodes: true,
            normalize_unicode: true,
            ..Default::default()
        }
    }
}

/// For some reason `libc::c_void` isn't hashable and cannot be made hashable
fn node_to_hashable(node : &XmlNodeRef) -> usize {
    unsafe { mem::transmute::<*mut libc::c_void, usize>(node.node_ptr) }
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
    //pub node_map : HashMap<XmlNodeRef, (usize, usize)>,
    //pub node_map : HashMap<libc::c_void, (usize, usize)>,
    pub node_map : HashMap<usize, (usize, usize)>,
}


/// Some temporary data for the parser
struct TmpParseData {
    /// plaintext is currently terminated by some whitespace
    had_whitespace : bool,
}

/// The heart of the dnm generation...
fn recursive_dnm_generation(dnm: &mut DNM, root: &XmlNodeRef,
                            tmp: &mut TmpParseData) {
    let mut offset_start = dnm.plaintext.len();
    let mut still_in_leading_whitespaces = true;

    if root.is_text_node() {  //CASE: WE HAVE A TEXT NODE
        //possibly normalize unicode
        let mut content = if dnm.parameters.normalize_unicode {
            unidecode(&root.get_content()) } else { root.get_content() };
        if dnm.parameters.stem_words_once {
            content = rustmorpha::stem(&content);
        }
        if dnm.parameters.stem_words_full {
            content = rustmorpha::full_stem(&content);
        }

        if dnm.parameters.convert_to_lowercase {
            content = content.to_lowercase();
        }

        //if the option is set, reduce sequences of white spaces to single spaces
        if dnm.parameters.normalize_white_spaces {
            for c in content.to_string().chars() {
                if c.is_whitespace() {
                    if tmp.had_whitespace { continue; }
                    dnm.plaintext.push(' ');
                    tmp.had_whitespace = true;
                    if dnm.parameters.move_whitespaces_between_nodes {
                        if still_in_leading_whitespaces {
                            offset_start += 1;
                        }
                    }
                }
                else {
                    dnm.plaintext.push(c);
                    tmp.had_whitespace = false;
                    still_in_leading_whitespaces = false;
                }
            }
        } else {
            dnm.plaintext.push_str(&content);
        }
        dnm.node_map.insert(node_to_hashable(root), (offset_start,
            if dnm.parameters.move_whitespaces_between_nodes && dnm.plaintext.len() > offset_start && tmp.had_whitespace {
                dnm.plaintext.len() - 1  //don't put trailing white space into node
            } else { dnm.plaintext.len() }));
        return;

    }

    //CASE: WE DON'T HAVE A TEXT NODE
    {   //need nested scope because of borrowing issues
    let name : String = root.get_name();
    let mut rules = Vec::new();
    rules.push(dnm.parameters.special_tag_name_options.get(&name));
    for classname in root.get_class_names() {
        rules.push(dnm.parameters.special_tag_class_options.get(&classname));
    }
    for rule in rules {  //iterate over applying rules
        match rule {
            Some(&SpecialTagsOption::Enter) => {
                break;
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
                dnm.node_map.insert(node_to_hashable(root),
                                    (offset_start,
                    if dnm.parameters.move_whitespaces_between_nodes && dnm.plaintext.len() > offset_start && tmp.had_whitespace {
                        dnm.plaintext.len() - 1    //don't put trailing white space into node
                    } else { dnm.plaintext.len() }));
                return;
            }
            Some(&SpecialTagsOption::FunctionNormalize(f)) => {
                if dnm.parameters.wrap_tokens {
                    if !tmp.had_whitespace ||
                       !dnm.parameters.normalize_white_spaces {
                        dnm.plaintext.push(' ');
                    }
                    dnm.plaintext.push_str(&f(&root));
                    dnm.plaintext.push(' ');
                    tmp.had_whitespace = true;
                } else {
                    dnm.plaintext.push_str(&f(&root));
                    //Return value of f is not considered a white space
                    tmp.had_whitespace = false;
                }
                dnm.node_map.insert(node_to_hashable(root),
                                    (offset_start,
                    if dnm.parameters.move_whitespaces_between_nodes && dnm.plaintext.len() > offset_start && tmp.had_whitespace {
                        dnm.plaintext.len() - 1    //don't put trailing white space into node
                    } else { dnm.plaintext.len() }));
                return;
            }
            Some(&SpecialTagsOption::Skip) => {
                dnm.node_map.insert(node_to_hashable(root),
                                            (offset_start,
                    if dnm.parameters.move_whitespaces_between_nodes && dnm.plaintext.len() > offset_start && tmp.had_whitespace {
                        dnm.plaintext.len() - 1    //don't put trailing white space into node
                    } else { dnm.plaintext.len() }));
                return;
            }
            None => {
                continue;
            }
        }
    }
    } //needed nested scope because of borrowing issues


    // Recurse into children
    let mut child_option = root.get_first_child();
    loop {
        match child_option {
            Some(child) => {
                recursive_dnm_generation(dnm, &child, tmp);
                child_option = child.get_next_sibling();
            }
            None => break,
        }
    }

    dnm.node_map.insert(node_to_hashable(root), (offset_start, 
        if dnm.parameters.move_whitespaces_between_nodes && dnm.plaintext.len() > offset_start && tmp.had_whitespace {
            dnm.plaintext.len() - 1    //don't put trailing white space into node
        } else { dnm.plaintext.len()}));
}



/// Very often we'll talk about substrings of the plaintext - words, sentences,
/// etc. A `DNMRange` stores start and end point of such a substring and has
/// a reference to the `DNM`.
pub struct DNMRange <'a> {
    pub start : usize,
    pub end : usize,
    pub dnm : &'a DNM,
}

impl <'a> DNMRange <'a> {
    /// Get the plaintext substring corresponding to the range
    pub fn get_plaintext(&self) -> String {
        //self.dnm.plaintext.slice_chars(self.start, self.end).to_owned()
        (&self.dnm.plaintext)[self.start..self.end].to_owned()
    }
    /// Get the plaintext without trailing white spaces
    pub fn get_plaintext_truncated(&self) -> String {
        //self.dnm.plaintext.slice_chars(self.start, self.end).trim_right().to_owned()
        (&self.dnm.plaintext)[self.start..self.end].trim_right().to_owned()
    }

    pub fn trim(&self) -> DNMRange <'a> {
      let mut trimmed_start = self.start;
      let mut trimmed_end = self.end;
      let range_text = self.get_plaintext();

      for c in range_text.chars() {
        if c.is_whitespace() {
          trimmed_start += 1; }
        else {
          break; }}
      for c in range_text.chars().rev() {
        if c.is_whitespace() {
          trimmed_end -= 1; }
        else {
          break; }}
      DNMRange {start : trimmed_start, end: trimmed_end, dnm: self.dnm}
    }
}

impl <'a> Clone for DNMRange <'a> {
    fn clone(&self) -> DNMRange <'a> {
        DNMRange {start : self.start, end: self.end, dnm: self.dnm}
    }
}

/// Prints warnings, if the parameter settings don't make sense.
/// Doesn't check for every possible stupidity
fn check_dnm_parameters(parameters: &DNMParameters) {
    if parameters.stem_words_once && parameters.stem_words_full {
        println_stderr!("llamapun::dnmlib: Parameter options stem_words_once\
and stem_words_full are both set");
    }
    if !parameters.normalize_white_spaces && parameters.move_whitespaces_between_nodes {
        println_stderr!("llamapun::dnmlib: Parameter option\
move_whitespaces_between_nodes only works in combination with normalize_white_spaces\n\
Consider using DNMRange::trim instead");
    }
    if !parameters.normalize_white_spaces && parameters.move_whitespaces_between_nodes {
        println_stderr!("llamapun::dnmlib: Parameter option\
move_whitespaces_between_nodes only works in combination with normalize_white_spaces\n\
Consider using DNMRange::trim instead");
    }
    if (parameters.stem_words_once || parameters.stem_words_full)
        && parameters.convert_to_lowercase {
        println_stderr!("llamapun::dnmlib: Parameter option convert_to_lowercase\
is redundant, because stemming converts to lowercase already");
    }
}


impl DNM {
    /// Creates a `DNM` for `root`
    pub fn create_dnm(root: &XmlNodeRef, parameters: DNMParameters) -> DNM {
        check_dnm_parameters(&parameters);
        let mut dnm = DNM {
            plaintext : String::new(),
            parameters : parameters,
            root_node : XmlNodeRef {node_ptr : root.node_ptr,
                                    node_is_inserted : true},
            node_map : HashMap::new(),
        };

        let mut tmp = TmpParseData {
            had_whitespace : true,  //no need for leading whitespaces
        };

        recursive_dnm_generation(&mut dnm, &root, &mut tmp);

        dnm
    }

    /// Get the plaintext range of a node
    pub fn get_range_of_node (self : &DNM, node: &XmlNodeRef)
                                    -> Result<DNMRange, ()> {
        match self.node_map.get(&node_to_hashable(&node)) {
            Some(&(start, end)) => Ok(DNMRange
                                     {start:start, end:end, dnm: self}),
            None => Err(()),
        }
    }
}

