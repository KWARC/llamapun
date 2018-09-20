use libxml::tree::Node;
// use libxml::xpath::Context;

/// Map math nodes to their lexemes
pub fn lexematize_math(node: &Node) -> String {
  // We are going to descend down an assumed equation/formula/eqnarray, grabbing any x-llamapun
  // encoded lexemes we can find

  // PROBLEM: Invoking findnodes so often is staggeringly slow on the test suite. 10x slow down!!!
  // let annotations = context.findnodes(
  //   "//*[local-name()='annotation' and @encoding='application/x-llamapun']",
  //   Some(node),
  // );

  // math -> semantics -> annotation[application/x-llamapun]

  // table -> tbody -> tr -> td -> math ... ugh

  // dashes and colons -> to underscores in lexemes ...

  println!(
    "lexematize on: {}:{}",
    node.get_name(),
    node.get_attribute("class").unwrap_or_default()
  );
  // Fallback if no lexemes - a generic substitution
  String::from("MathFormula")
}
