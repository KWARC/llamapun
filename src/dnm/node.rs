use lazy_static::lazy_static;
use libxml::readonly::RoNode;
use libxml::xpath::Context;
use regex::Regex;

lazy_static! {
 static ref FUSED_ENDS : Regex = Regex::new(r"^(.+)((?:OPFUNCTION|OPERATOR|UNKNOWN|ADDOP|RELOP|MULOP|ID|BIGOP|OVERACCENT|UNDERACCENT)[:]end)$").unwrap();
}

/// Map math nodes to their lexemes
pub fn lexematize_math(node: RoNode, context: &mut Context) -> String {
  // We are going to descend down an assumed equation/formula/eqnarray, grabbing any x-llamapun
  // encoded lexemes we can find

  let annotations = context
    .node_evaluate_readonly(
      ".//*[local-name()='annotation' and @encoding='application/x-llamapun']",
      node,
    )
    .unwrap()
    .get_readonly_nodes_as_vec();

  let lexemes: String = annotations
    .iter()
    .map(|anno| {
      let mut annotation_string = anno.get_content();
      // offer fix for latexml 0.8.4 serialization flaw in some cases (e.g. "POSTFIX:endID:end"
      // instead of "POSTFIX:end ID:end")
      // THERE is a list of complications on the left side as well, namely:
      // (OPFUNCTION|OPERATOR|UNKNOWN|ADDOP|RELOP|MULOP|ID|BIGOP|OVERACCENT|UNDERACCENT)_end
      // show as glued to ~approx random left-hand side content (e.g. OPFUNCTION_TrivOPFUNCTION_end)
      annotation_string = annotation_string
        .split(":end")
        .collect::<Vec<&str>>()
        .join(":end ");
      annotation_string = annotation_string
        .split(":start")
        .collect::<Vec<&str>>()
        .join(":start ");
      annotation_string
        .trim()
        .split_whitespace()
        .flat_map(|anno_word|
          if let Some(cap) = FUSED_ENDS.captures(anno_word) {
            let first = cap.get(1).map_or("", |w| w.as_str());
            let second = cap.get(2).map_or("", |w| w.as_str());
            vec![first, second]
          } else {
            vec![anno_word]
          })
        .map(|anno_word| {
          if anno_word.starts_with("NUM") {
            String::from("NUM")
          } else if anno_word.starts_with("ARRAY") {
            String::from("ARRAY")
          } else if anno_word.starts_with("ATOM") {
            String::from("ATOM")
          } else if anno_word.starts_with("SUPERSCRIPTOP") {
            String::from("SUPERSCRIPTOP")
          } else {
            anno_word
              .chars()
              .map(|c| match c {
                ':' | '-' => '_',
                '\n' => ' ',
                _ => c,
              })
              .collect()
          }
        })
        .filter(|x| !x.is_empty())
        .collect::<Vec<String>>()
        .join(" ")
    })
    .filter(|x| !x.is_empty())
    .collect::<Vec<String>>()
    .join(" ");
  if !lexemes.is_empty() {
    lexemes
  } else {
    // Fallback if no lexemes - a generic substitution
    // intended to be used in a lowercased context, where only significant lexemes are left
    // capitalized
    String::from("mathformula")
  }
}
