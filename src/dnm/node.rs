use lazy_static::lazy_static;
use libxml::readonly::RoNode;
use libxml::xpath::Context;
use regex::Regex;
lazy_static! {
  static ref LEXEME_END_MARKER : Regex = Regex::new(r"((?:OPFUNCTION|OPERATOR|OPEN|CLOSE|UNKNOWN|RELOP|ADDOP|MULOP|ARROW|BIGOP|BINOP|ID|OVERACCENT|UNDERACCENT):end)").unwrap();
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
      annotation_string = LEXEME_END_MARKER
        .replace_all(&annotation_string, " $1 ")
        .split(":end")
        .collect::<Vec<&str>>()
        .join(":end ");
      annotation_string = annotation_string
        .split(":start")
        .collect::<Vec<&str>>()
        .join(":start ");
      annotation_string
        .split_whitespace()
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
        .collect::<Vec<String>>()
        .join(" ")
    })
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
