extern crate llamapun;
extern crate libxml;
extern crate senna;

use std::env;
use std::io::Write;
use std::collections::HashMap;

use libxml::xpath::Context;
use libxml::tree::Node;
use libxml::tree::Document as DOM;

use senna::sennapath::SENNA_PATH;
use senna::senna::{Senna, SennaParseOptions};

use llamapun::data::Corpus;
use llamapun::dnm::{DNM, DNMParameters, DNMRange};
use llamapun::tokenizer::Tokenizer;

fn get_parent_chain(from: &Node, until: &Node) -> Vec<Node> {
    let mut chain : Vec<Node> = Vec::new();
    let mut it = from.clone();
    while it != *until {
        chain.push(it.clone());
        it = it.get_parent().expect("Expected parent");
    }
    chain.push(it.clone());
    return chain;
}

fn is_child_of(child: &Node, parent: &Node, root: &Node) -> bool {
    let mut it = child.clone();
    loop {
        if it == *parent {
            return true;
        } else if it == *root {
            return false;
        } else {
            it = it.get_parent().expect("Expected parent");
        }
    }
}

fn get_plaintext(node: &Node) -> (String, Vec<usize>, Vec<Node>) {
    let mut plaintext = String::new();
    let mut offsets : Vec<usize> = Vec::new();
    let mut nodes : Vec<Node> = Vec::new();
    if node.is_text_node() {
        let content = node.get_content();
        for i in 0..content.len() {
            offsets.push(i);
            nodes.push(node.clone());
        }
        plaintext.push_str(&content);
    } else {
        let name = node.get_name();
        let classvals = node.get_class_names();
        if name == "math" || classvals.contains("ltx_equation") || classvals.contains("ltx_equationgroup") {
            plaintext.push_str("mathformula");
            for i in 0..11 {
                offsets.push(i);
                nodes.push(node.clone());
            }
        } else if name == "cite" {
            plaintext.push_str("citationelement");
            for i in 0..15 {
                offsets.push(i);
                nodes.push(node.clone());
            }
        } else if name == "table" {
            /* skip */
        } else {
            // recurse into children
            let mut child_opt = node.get_first_child();
            loop {
                match child_opt {
                    Some(child) => {
                        let (p, o, n) = get_plaintext(&child);
                        plaintext.push_str(&p);
                        // offsets.push_all(&o);
                        // nodes.push_all(&n);
                        offsets.extend(o.into_iter());
                        nodes.extend(n.into_iter());
                        child_opt = child.get_next_sibling();
                    },
                    None => break
                }
            }
        }
    }
    if plaintext.len() != offsets.len() || offsets.len() != nodes.len() {
        panic!("Lenghts don't match!!");
    }
    return (plaintext, offsets, nodes);
}

fn annotate(node : Node, root: &Node, range: &DNMRange, dnm: &DNM, dom: &DOM) -> bool {
    // find lowest parent of range

    let (_, offsets, nodes) = get_plaintext(root); // Need to recalculate it every round



    if range.start == range.end {
        return false;
    }

    if range.end > nodes.len(){
        writeln!(std::io::stderr(), "Warning: range.end > nodes.len() in annotate. Something went wrong here before").unwrap();
        return false;
    }

    let start_node = &nodes[range.start];
    let start_parents : Vec<Node> = get_parent_chain(start_node, root);
    let end_node = &nodes[range.end-1];
    let end_parents : Vec<Node> = get_parent_chain(end_node, root);


    let mut si = start_parents.len() - 1;
    let mut ei = end_parents.len() - 1;
    while si > 0 && ei > 0 && start_parents[si-1] == end_parents[ei-1] {
        si -= 1;
        ei -= 1;
    }

    let common_parent = &start_parents[si];

    if common_parent.is_text_node() {
        let before_start = fix_byte_index(range.start - offsets[range.start], dnm.plaintext.clone());
        let before_end   = fix_byte_index(range.start, dnm.plaintext.clone());
        let core_end     = fix_byte_index(range.end, dnm.plaintext.clone());
        let before = Node::new_text_node(&dom, &dnm.plaintext[before_start..before_end]).unwrap();
        let core = Node::new_text_node(&dom, &dnm.plaintext[before_end..core_end]).unwrap();
        let mut textend = range.start+1;
        while textend < offsets.len() && offsets[textend] > 0 {
            textend += 1;
        }

        textend = fix_byte_index(textend, dnm.plaintext.clone());

        let after = Node::new_text_node(&dom, &dnm.plaintext[core_end..textend]).unwrap();

        common_parent.add_prev_sibling(node.clone()).unwrap();
        node.add_prev_sibling(before).unwrap();
        let break_ = Node::new("BREAK", None, &dom).unwrap();
        common_parent.add_prev_sibling(break_.clone()).unwrap();
        break_.add_prev_sibling(after).unwrap();
        break_.unlink();
        break_.free();
        common_parent.unlink();
        common_parent.free();
        node.add_child(&core).unwrap();
    } else if common_parent == start_node && common_parent == end_node {
        if offsets[range.start] == 0 && (range.end == offsets.len() || offsets[range.end] == 0) {
            common_parent.add_prev_sibling(node.clone()).unwrap();
            common_parent.unlink();
            node.add_child(common_parent).unwrap();
        } else {
            writeln!(std::io::stderr(), "Warning: Couldn't split for annotation (doesn't match token boundaries): \"{}\"",
            range.get_plaintext()).unwrap();
        }
    } else {
        // make sure splitting is possible
        let mut act_start = start_parents[si - 1].clone();
        if !(range.start == 0 ||
             act_start.is_text_node() ||
             !is_child_of(&nodes[range.start-1], &act_start, &root)) {

            // let act_end = &end_parents[ei - 1];
            // writeln!(std::io::stderr(), "NAMES: \"{}\" > \"{}\" | \"{}\"", dom.node_to_string(&common_parent), dom.node_to_string(&act_start), dom.node_to_string(&act_end)).unwrap();
            writeln!(std::io::stderr(), "Warning: Couldn't split for tokenization (at beginning): \"{}\"",
            range.get_plaintext()).unwrap();
            node.free();
            return false;
        }
        let mut act_end = end_parents[ei - 1].clone();
        if !(range.end == dnm.plaintext.len()-1 ||
             act_end.is_text_node() ||
             !is_child_of(&nodes[range.end+1], &act_end, &root)) {
            writeln!(std::io::stderr(), "Warning: Couldn't split for tokenization (at end): \"{}\"",
            range.get_plaintext()).unwrap();
            node.free();
            return false;
        }

        // split text nodes
        if act_start.is_text_node() && offsets[range.start] != 0 {  // have to split act_start
            let before_start = fix_byte_index(range.start - offsets[range.start], dnm.plaintext.clone());
            let before_end = fix_byte_index(range.start, dnm.plaintext.clone());

            let before = Node::new_text_node(&dom, &dnm.plaintext[before_start..before_end]).unwrap();
            let mut textend = range.start+1;
            while offsets[textend] > 0 {  // can't run to end of array, because in that case we'd have a text node as common parent (checked for before)
                textend += 1;
            }

            textend = fix_byte_index(textend, dnm.plaintext.clone());

            let after = Node::new_text_node(&dom, &dnm.plaintext[before_end..textend]).unwrap();
            let break_ = Node::new("BREAK", None, &dom).unwrap();  // make sure text nodes don't get merged into act_start
            act_start.add_prev_sibling(break_.clone()).unwrap();
            break_.add_prev_sibling(before).unwrap();
            let tmp = break_.add_prev_sibling(after).unwrap();
            act_start.unlink();
            act_start.free();
            break_.unlink();
            break_.free();
            act_start = tmp;
        }
        if act_end.is_text_node() && range.end < dnm.plaintext.len() - 1 &&
            offsets[range.end+1] != 0 {
                let before_start = fix_byte_index(range.end - offsets[range.end], dnm.plaintext.clone());
                let before_end = fix_byte_index(range.end, dnm.plaintext.clone());
                let before = Node::new_text_node(&dom, &dnm.plaintext[before_start..before_end]).unwrap();
                let mut textend = range.end+1;
                while textend < offsets.len() && offsets[textend] > 0 {
                    textend += 1;
                }

                textend = fix_byte_index(textend, dnm.plaintext.clone());
                let after = Node::new_text_node(&dom, &dnm.plaintext[before_end..textend]).unwrap();
                let stop = Node::new("STOP", None, &dom).unwrap();
                act_end.add_prev_sibling(stop.clone()).unwrap();
                let tmp = stop.add_prev_sibling(before).unwrap();

                stop.add_prev_sibling(after).unwrap();
                act_end.unlink();
                act_end.free();
                stop.unlink();
                stop.free();
                act_end = tmp;
            }

        act_start.add_prev_sibling(node.clone()).unwrap();
        while act_start != act_end {    // iterate with act_start to act_end and move everything inside node
            let tmp = act_start.get_next_sibling().unwrap();
            act_start.unlink();
            node.add_child(&act_start).unwrap();
            act_start = tmp;
        }
        act_end.unlink();
        node.add_child(&act_end).unwrap();
    }

    return true;
}

/* Takes a possibly invalid byte index to plaintext and returns a close and valid byte index. A byte index is invalid iff it does
   not point to a char boundary */
fn fix_byte_index(mut byte_index : usize, plaintext : String) -> usize {
   // for (byte, char) in plaintext.clone().char_indices() {
   //     println!("Indices {} {} {}, Goal {}", byte, char, plaintext.is_char_boundary(byte), byte_index);
   // }

    if !plaintext.is_char_boundary(byte_index){
        if byte_index > plaintext.len() {
            // plaintext.len() is always a char boundary
            byte_index = plaintext.len();
        }else {
            while byte_index < plaintext.as_bytes().len() && !plaintext.is_char_boundary(byte_index){
                // increase to the next char boundary
                byte_index += 1;
            }
        }
    }
   // println!("byte_index {}", byte_index);
    byte_index
}


fn add_ids_to_math(root: &Node, id: &str) {
    let mut c : Option<Node>;
    let mut tmp : Option<Node> = root.get_first_child();
    let mut idcounter = 0;
    loop {
        c = tmp;
        match &c {
            &Some(ref child) => {
                let childid = format!("{}.{}", id, idcounter);
                idcounter += 1;
                child.remove_property_with_name("id");
                child.add_property("id", &childid);
                add_ids_to_math(&child, &childid);
                tmp = child.get_next_sibling();
            },
            &None => break,
        }
    }
}


pub fn main() {
    let args : Vec<_> = env::args().collect();
    let corpus_path : &str = if args.len() > 1 { &args[1] } else { "tests/resources/" };
    println!("Loading corpus from \"{}\"", corpus_path);

    let mut senna = Senna::new(SENNA_PATH.to_owned());
    let tokenizer = Tokenizer::default();

    let mut sentence_id_counter = 0usize;

    let mut corpus = Corpus::new(corpus_path.to_owned());
    for document in corpus.iter() {
        if args.len() <= 1 {
            return;
        }

        // let document = corpus.load_doc(args[1].clone()).unwrap();
        if true {
            println!("Processing \"{}\"", &document.path);
            let dom = document.dom;
            let xpath_context = Context::new(&dom).unwrap();


            // Remove ltx_p nodes
            match xpath_context.evaluate("//*[contains(@class,'ltx_para')]//p[contains(@class,'ltx_p')]") {
                Ok(result) => {
                    for ltx_p in result.get_nodes_as_vec() {
                        // move children out
                        loop {
                            match ltx_p.get_first_child() {
                                None => { break; },   // Done
                                Some(child) => {
                                    child.unlink();
                                    ltx_p.add_prev_sibling(child).unwrap();
                                }
                            }
                        }
                        ltx_p.unlink();
                        ltx_p.free();
                    }
                },
                Err(_) => {
                    writeln!(std::io::stderr(), "Warning: Didn't remove any //*[contains(@class,'ltx_para')]//p[contains(@class,'ltx_p')]").unwrap();
                }
            }


            let paras = match xpath_context.evaluate("//*[contains(@class,'ltx_para')]") {
                Ok(result) => result.get_nodes_as_vec(),
                Err(_) => {
                    writeln!(std::io::stderr(), "Warning: No paragraphs found").unwrap();
                    vec![]
                }
            };

            for para in paras {
                let (plaintext, _, _) = get_plaintext(&para);
                // Need to create DNM for sentence tokenizer
                let dnm = DNM {
                    plaintext: plaintext,
                    parameters: DNMParameters::default(),
                    root_node: para.clone(),
                    node_map: HashMap::new(),
                };
                let sentences = tokenizer.sentences(&dnm);
                for sentence in sentences {
                    let pt = sentence.get_plaintext().replace("MathFormula", "mathformula");
                    //let senna_parse = senna.parse(&pt, SennaParseOptions { pos: true, psg: true,});
                    let snode = Node::new("span", None, &dom).unwrap();
                    snode.add_property("class", "sentence");
                    // Colors help to easily see missing annotations
                    snode.add_property("style", "color:darkgreen");
                    snode.add_property("id", &format!("sentence.{}", sentence_id_counter));
                    sentence_id_counter += 1;
                    //snode.add_property("psg", senna_parse.get_psgstring().unwrap());

                    annotate(snode, &para, &sentence, &dnm, &dom);

                    let mut word_id_counter = 0usize;

                    for word in tokenizer.words(&sentence) {
                        //for word in senna_parse.get_words() {
                        let wnode = Node::new("span", None, &dom).unwrap();
                        wnode.add_property("class", "word");
                        // wnode.add_property("style", "color:blue");
                        wnode.add_property("id", &format!("word.{}.{}", sentence_id_counter, word_id_counter));
                        word_id_counter += 1;
                        //    wnode.add_property("pos", word.get_pos().to_str());
                        //let word_range = sentence.get_subrange(word.get_offset_start(), word.get_offset_end());
                      //  let word_range = sentence.get_subrange(word.start, word.end);
//                        println!("word {} word.start {} word.end {}, sentence.start {}, sentence.end {}", word.get_plaintext(), word.start, word.end,
  //                          sentence.start, sentence.end);
                        annotate(wnode, &para, &word, &dnm, &dom);
                    }
                }
            }

            // add ids into mathml
            let mathtags = match xpath_context.evaluate("//math") {
                Ok(result) => result.get_nodes_as_vec(),
                Err(_) => {
                    writeln!(std::io::stderr(), "Warning: No paragraphs found").unwrap();
                    vec![]
                }
            };

            let mut mathidcounter = 0;
            for mathtag in mathtags {
                /* let id = match mathtag.get_property("id") {
           None => {
               let newid = format!("math.{}", mathidcounter);
               mathtag.add_property("id", newid);
               newid
           }
           Some(i) => &i
       };
       */
                mathtag.remove_property_with_name("id");
                let newid = format!("math.{}", mathidcounter);
                mathidcounter += 1;
                mathtag.add_property("id", &newid);
                add_ids_to_math(&mathtag, &newid);
            }

            dom.save_file(if args.len() > 2 { &args[2] } else {
                println!("Saving at /tmp/out.html");
                "/tmp/out.html"
            }).unwrap();
        }
    }
}

