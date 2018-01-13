//! Some utility functions. Mostly helper functions for dealing with the XML

use libxml::tree::*;


/*
 * XML HELPER FUNCTIONS
 */


/// checks whether a node is a comment node
pub fn is_comment_node(node : &Node) -> bool {
    node.get_type().unwrap() == NodeType::CommentNode
}

/// gets the text content of a node. Requires that only child of the node is a text node
pub fn get_simple_node_content(node : &Node, trim: bool) -> Result<String, String> {
    let child = node.get_first_child();
    if child.is_none() {
        Ok(String::new())
    } else if !child.as_ref().unwrap().is_text_node() || child.as_ref().unwrap().get_next_sibling().is_some() {
        Err(format!("found unexpected nodes in node \"{}\"", node.get_name()))
    } else {
        Ok(if trim { child.as_ref().unwrap().get_content().trim().to_string() } else { child.as_ref().unwrap().get_content() } )
    }
}

/// Returns a vector of the children, skipping text and comment nodes.
/// Requires that the text nodes in between contain only whitespaces.
pub fn get_non_text_children(node : &Node) -> Result<Vec<Node>, String> {
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

/// Returns a vector of the children, skipping comments and text nodes
pub fn fast_get_non_text_children(node : &Node) -> Vec<Node> {
    let mut cur = node.get_first_child();
    let mut children : Vec<Node> = Vec::new();
    loop {
        if cur.is_none() { return children; }
        let cur_ = cur.unwrap();
        if !cur_.is_text_node() && !is_comment_node(&cur_) { children.push(cur_.clone()); }
        cur = cur_.get_next_sibling();
    }
}

/// gets the non-text child of a node. Requires it to be the only child node apart from comments
/// and text nodes containing only whitespaces
pub fn get_only_child(node : &Node) -> Result<Node, String> {
    let children = try!(get_non_text_children(node));
    if children.len() < 1 {
        Err(format!("Expected child node in node \"{}\"", node.get_name()))
    } else if children.len() > 1 {
        Err(format!("Too many child nodes in node \"{}\"", node.get_name()))
    } else {
        Ok(children[0].clone())
    }
}

/// asserts that a node has no children (apart from comments and text nodes containing only
/// whitespaces)
pub fn assert_no_child(node : &Node) -> Result<(), String> {
    if try!(get_non_text_children(node)).is_empty() {
        Ok(())
    } else {
        Err(format!("Found unexpected child of node \"{}\"", node.get_name()))
    }
}

/// Gets a property from a node (or an `Err`, if it doesn't have the property)
pub fn require_node_property(node : &Node, property : &str) -> Result<String, String> {
    match node.get_property(property) {
        None => Err(format!("\"{}\" node misses \"{}\" property", node.get_name(), property)),
        Some(value) => Ok(value)
    }
}

/// generates a specific error message if `property.is_some()`
pub fn check_found_property_already(property: &Option<String>,
                                node_name: &str,
                                parent_name: &str) -> Result<(), String> {
    if property.is_some() {
        Err(format!("found multiple \"{}\" nodes in \"{}\" node", node_name, parent_name))
    } else {
        Ok(())
    }
}

