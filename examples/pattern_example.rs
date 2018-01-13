extern crate llamapun;
extern crate senna;
extern crate libxml;

use llamapun::patterns::*;
use llamapun::data::Corpus;
use senna::senna::SennaParseOptions;
use llamapun::dnm::*;
use libxml::xpath::Context;
use libxml::tree::*;
use std::rc::Rc;
use std::collections::HashMap;


/// turns a marker into a readable string representation
fn get_pattern_marker_string(marker : &PatternMarker) -> String {
    let mut result = String::new();
    result.push_str(&marker.name);
    result.push_str(" {");
    let mut first = true;
    for tag in &marker.tags {
        if !first {
            result.push_str(", ");
        }
        first = false;
        result.push('\'');
        result.push_str(tag);
        result.push('\'');
    }
    result.push('}');
    result
}


/// turns a math node into a readable string representation
fn math_node_to_string(node : &Node) -> String {
    let mut s = String::new();
    math_node_to_string_actual(node, &mut s);
    s
}

/// helper function
fn math_node_to_string_actual(node : &Node, mut string : &mut String) {
    match node.get_name().as_ref() {
        "semantics" => math_node_to_string_children(node, &mut string),
        "annotation" => { },
        "annotation-xml" => { },
        "text" => {
            if node.is_text_node() {
                string.push_str(&node.get_content());
            }
        }
        default => {
            string.push('<');
            string.push_str(default);
            string.push('>');
            math_node_to_string_children(node, &mut string);
            string.push('<');
            string.push('/');
            string.push_str(default);
            string.push('>');
        }
    }
}

/// helper function
fn math_node_to_string_children(node : &Node, mut string : &mut String) {
    let mut cur = node.get_first_child();
    loop {
        if cur.is_none() { break; }
        math_node_to_string_actual(cur.as_ref().unwrap(), &mut string);
        cur = cur.unwrap().get_next_sibling();
    }
}


/// prints a marker in a human readable way
fn print_marker(marker : &MarkerEnum, alt_dnm : &DNM, xpath_context : &Context) {
    match marker {
        &MarkerEnum::Text(ref text_marker) => {
            println!("<h5>TextMarker</h5> \"{}\" \n <br /><br /><p>{}</p>", &get_pattern_marker_string(&text_marker.marker),
                     DNMRange::deserialize(
                         &text_marker.range.serialize(),
                         &alt_dnm,
                         xpath_context)
                     .get_plaintext());
        }
        &MarkerEnum::Math(ref math_marker) => {
            println!("<h5>MathMarker</h5> \"{}\"\n <br /><br /> <p>{}</p>", &get_pattern_marker_string(&math_marker.marker),
                     &math_node_to_string(&math_marker.node));
        }
    }
}

/// gets a DNM that is more readable for printing
fn get_alternative_dnm(root: &Node) -> DNM {
    let mut name_options = HashMap::new();
    name_options.insert("math".to_string(),
                        SpecialTagsOption::FunctionNormalize(Rc::new(math_node_to_string)));
    name_options.insert("cite".to_string(),
                        SpecialTagsOption::Normalize("CitationElement".to_string()));
    name_options.insert("table".to_string(), SpecialTagsOption::Skip);
    name_options.insert("head".to_string(), SpecialTagsOption::Skip);

    let mut class_options = HashMap::new();
    class_options.insert("ltx_equation".to_string(),
                         SpecialTagsOption::FunctionNormalize(Rc::new(math_node_to_string)));
    class_options.insert("ltx_equationgroup".to_string(),
                         SpecialTagsOption::FunctionNormalize(Rc::new(math_node_to_string)));
    class_options.insert("ltx_note_mark".to_string(), SpecialTagsOption::Skip);
    class_options.insert("ltx_note_outer".to_string(), SpecialTagsOption::Skip);
    class_options.insert("ltx_bibliography".to_string(), SpecialTagsOption::Skip);

    let parameters = DNMParameters {
        special_tag_name_options: name_options,
        special_tag_class_options: class_options,
        normalize_white_spaces: true,
        wrap_tokens: false,
        normalize_unicode: false,
        ..Default::default()
    };

    DNM::new(root.clone(), parameters)
}

pub fn main() {
    let pattern_file_result = PatternFile::load("examples/declaration_pattern.xml");
    let pattern_file = match pattern_file_result {
        Err(x) => panic!(x),
        Ok(x) => x,
    };

    let mut corpus = Corpus::new("tests/resources/".to_string());
    corpus.senna_options = std::cell::Cell::new( SennaParseOptions { pos : true, psg : true } );
    corpus.dnm_parameters.support_back_mapping = true;

    let mut document = corpus.load_doc("tests/resources/1311.0066.xhtml".to_string()).unwrap();
    // let mut document = corpus.load_doc("tests/resources/0903.1000.html".to_string()).unwrap();


    // get a more readable DNM for printing
    let alt_dnm = get_alternative_dnm(&document.dom.get_root_element());
    println!("<?xml version=\"1.0\" encoding=\"utf-8\"?><html><head><meta http-equiv=\"Content-Type\" content=\"application/xhtml+xml; charset=UTF-8\"/></head>
             <body>");

    for mut sentence in document.sentence_iter() {
        let sentence_2 = sentence.senna_parse();
        let matches = match_sentence(&pattern_file, &sentence_2.senna_sentence.as_ref().unwrap(),
                                     &sentence_2.range, "declaration").unwrap();
        if matches.len() > 0 {
            let xpath_context = Context::new(&sentence_2.document.dom).unwrap();
            println!("<hr />");
            println!("<h4>Sentence</h4>\n<p>{}</p>",
                     DNMRange::deserialize(
                         &sentence_2.range.serialize(),
                         &alt_dnm,
                         &xpath_context)
                     .get_plaintext());
            for m in &matches {
                for m2 in &m.get_marker_list() {
                    print_marker(m2, &alt_dnm, &xpath_context);
                }
            }
        }
    }
    
    println!("</body></html>");
}
