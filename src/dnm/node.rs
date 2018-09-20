use libxml::tree::Node;
use libxml::xpath::Context;

/// Map math nodes to their lexemes
pub fn lexematize_math(node: &Node, context: &mut Context) -> String {
  // We are going to descend down an assumed equation/formula/eqnarray, grabbing any x-llamapun
  // encoded lexemes we can find

  let annotations = context
    .findnodes(
      ".//*[local-name()='annotation' and @encoding='application/x-llamapun']",
      Some(node),
    ).unwrap();

  let lexemes: String = annotations
    .iter()
    .map(|anno| {
      anno
        .get_content()
        .split_whitespace()
        .map(|anno_word| {
          if anno_word.starts_with("NUM") {
            String::from("NUM")
          } else {
            anno_word
              .chars()
              .map(|c| match c {
                ':' | '-' => '_',
                '\n' => ' ',
                _ => c,
              }).collect()
          }
        }).collect::<Vec<String>>()
        .join(" ")
    }).collect::<Vec<String>>()
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
