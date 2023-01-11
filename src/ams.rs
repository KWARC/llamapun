//! Representation, normalization and utilities for working with AMS markup in LaTeX-derived
//! scientific documents

use crate::data::Document;
use crate::util::data_helpers;
use libxml::tree::Document as XmlDoc;
use libxml::xpath::Context;
use regex::Regex;
use std::fmt;

/// Checks a llamapun `Document` for 'ltx_theorem' AMS markup
pub fn has_markup(doc: &Document) -> bool { has_markup_xmldoc(&doc.dom) }

/// Checks a libxml document for `ltx_theorem` AMS markup
pub fn has_markup_xmldoc(dom: &XmlDoc) -> bool {
  let xpath_context = Context::new(dom).unwrap();
  match xpath_context.evaluate("//*[local-name()='div' and contains(@class,'ltx_theorem')][1]") {
    Ok(found_payload) => !found_payload.get_nodes_as_vec().is_empty(),
    _ => false,
  }
}

/// Semantically fixed structural environments in scientific documents, to collect as
/// add-on to the AMS markup
///
/// Note we are explicitly ignoring some of the very high-frequency environments, as they are not
/// rich on textual content. Namely: references, appendix, pacs, subject; Which are rich in metadata
/// and semi-structured content (figures, tables).
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StructuralEnv {
  Abstract,
  Acknowledgement,
  // Algorithm, // avoid due to noise+confusion with AMS
  Analysis,
  Application,
  Assumption,
  Background,
  Case,
  Caption,
  Claim,
  Conclusion,
  Condition,
  // An unproven proposition (includes, "Hypothesis")
  Conjecture,
  Contribution,
  Corollary,
  Data,
  Dataset,
  Definition,
  Demonstration,
  Description,
  Discussion,
  Example,
  Experiment,
  Fact,
  FutureWork,
  Implementation,
  Introduction,
  // Keywords,
  Lemma,
  Methods,
  Model,
  Motivation,
  Notation,
  Observation,
  Other,
  Preliminaries,
  /// A task to be solved (sometimes with solution following), includes "Exercise"
  Problem,
  Proof,
  Property,
  Proposition,
  Question,
  RelatedWork,
  Remark,
  Result,
  Simulation,
  Step,
  Summary,
  Theorem,
  Theory,
}

impl From<&str> for StructuralEnv {
  fn from(heading: &str) -> StructuralEnv {
    use StructuralEnv::*;
    let normalized_heading = data_helpers::normalize_heading_title(&heading.to_lowercase());
    match normalized_heading.as_str() {
      "abstract" => Abstract,
      "acknowledgement" => Acknowledgement,
      // "algorithm" => Algorithm,
      "analysis" => Analysis,
      "application" => Application,
      "assumption" => Assumption,
      "background" => Background,
      "case" => Case,
      "caption" => Caption,
      "claim" => Claim,
      "conclusion" => Conclusion,
      "condition" => Condition,
      "conjecture" => Conjecture,
      "contribution" => Contribution,
      "corollary" => Corollary,
      "data" => Data,
      "dataset" => Dataset,
      "definition" => Definition,
      "demonstration" => Demonstration,
      "description" => Description,
      "discussion" => Discussion,
      "example" => Example,
      "experiment" => Experiment,
      "fact" => Fact,
      "future work" => FutureWork,
      "implementation" => Implementation,
      "introduction" => Introduction,
      // "keywords" => Keywords,
      "lemma" => Lemma,
      "methods" => Methods,
      "model" => Model,
      "motivation" => Motivation,
      "notation" => Notation,
      "observation" => Observation,
      "preliminaries" => Preliminaries,
      "problem" => Problem,
      "proof" => Proof,
      "property" => Property,
      "proposition" => Proposition,
      "question" => Question,
      "related work" => RelatedWork,
      "remark" => Remark,
      "result" => Result,
      "simulation" => Simulation,
      "step" => Step,
      "summary" => Summary,
      "theorem" => Theorem,
      "theory" => Theory,
      _ => Other,
    }
  }
}

impl fmt::Display for StructuralEnv {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    use StructuralEnv::*;
    let val = match self {
      Abstract => "abstract",
      Acknowledgement => "acknowledgement",
      // Algorithm => "algorithm",
      Analysis => "analysis",
      Application => "application",
      Assumption => "assumption",
      Background => "background",
      Case => "case",
      Claim => "claim",
      Caption => "caption",
      Conclusion => "conclusion",
      Condition => "condition",
      Conjecture => "conjecture",
      Contribution => "contribution",
      Corollary => "corollary",
      Data => "data",
      Dataset => "dataset",
      Definition => "definition",
      Demonstration => "demonstration",
      Description => "description",
      Discussion => "discussion",
      Example => "example",
      Experiment => "experiment",
      Fact => "fact",
      FutureWork => "future work",
      Implementation => "implementation",
      Introduction => "introduction",
      // Keywords => "keywords",
      Lemma => "lemma",
      Methods => "methods",
      Model => "model",
      Motivation => "motivation",
      Notation => "notation",
      Observation => "observation",
      Preliminaries => "preliminaries",
      Problem => "problem",
      Proof => "proof",
      Property => "property",
      Proposition => "proposition",
      Question => "question",
      RelatedWork => "related work",
      Remark => "remark",
      Result => "result",
      Simulation => "simulation",
      Step => "step",
      Summary => "summary",
      Theorem => "theorem",
      Theory => "theory",
      Other => "other",
    };
    fmt.write_str(val)
  }
}

/// Author-annotated \newthorem{} environments using the amsthm.sty mechanism
/// which are then transported in the HTML representation as `ltx_theorem_<env>`
///
/// We shortlist 23 of the >20,000 variety of values present in the arXiv corpus
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AmsEnv {
  /// typically co-author support for a proof/paper (also "thanks")
  Acknowledgement,
  /// usually defines a computer science algorithm (also "heuristic"; arXiv data is bad quality,
  /// would not recommend using)
  Algorithm,
  /// To be analyzed (?)
  Answer,
  /// To be analyzed (?)
  Affirmation,
  /// assumption/axiom/assertion/prior -- should they be included in propositions? Are they
  /// separable?
  Assumption,
  /// To be analyzed (?)
  Bound,
  /// usually an actual Figure or Table captions realized via AMS (strange data, would not
  /// recommend using)
  Caption,
  /// A case in a multi-step proof / description / exposition
  Case,
  /// To be analyzed (?)
  Claim,
  /// To be analyzed (?)
  Comment,
  /// To be analyzed (?)
  Conclusion,
  /// Potentially a constraint on a proof
  Condition,
  /// An unproven statement/theorem (includes "conjecture", "ansatz", "guess", "hypothesis")
  Conjecture,
  /// To be analyzed (?)
  Constraint,
  /// To be analyzed (?)
  Convention,
  /// A direct-to-derive consequence of a prior proposition
  Corollary,
  /// To be analyzed (?)
  Criterion,
  /// Unlike notations, introduces new conceptual mathematical objects
  Definition,
  /// To be analyzed (?)
  Demonstration,
  /// To be analyzed (?)
  Discussion,
  /// Demonstration of a definition, notation etc (also "experiment")
  Example,
  /// To be analyzed (?)
  Experiment,
  /// To be analyzed (?)
  Expansion,
  /// To be analyzed (?)
  Expectation,
  /// To be analyzed (?)
  Explanation,
  /// To be analyzed (?)
  Fact,
  /// To be analyzed (?)
  Hint,
  /// To be analyzed (?)
  Issue,
  /// To be analyzed (?)
  Keywords,
  /// A smaller sub-theorem to a main theorem
  Lemma,
  /// Introduces a new syntactic rule, usually for convenience / brevity
  Notation,
  /// To be analyzed (?)
  Note,
  /// To be analyzed (?)
  Notice,
  /// To be analyzed (?)
  Observation,
  /// A named paragraph, without a clear standalone function
  Paragraph,
  /// To be analyzed (?)
  Principle,
  /// A task to be solved (sometimes with solution following), includes "Exercise"
  Problem,
  /// Proves a prior theorem/lemma
  Proof,
  /// A provably true/false statement. Is this a synonym to theorem in arXiv?
  Proposition,
  /// (sometimes) initial goal of inquiry (also "puzzle", "query")
  Question,
  /// A comment that is an aside to the main line of reasoning
  Remark,
  /// Summarizes paper's experimental deliverables
  Result,
  /// To be analyzed (?)
  Rule,
  /// To be analyzed (?)
  Solution,
  /// A part of a proof, or demonstration/layout
  Step,
  /// To be analyzed (?)
  Summary,
  /// A main proposition to be proven in the document
  Theorem,
  /// Anything else that was marked up with AMS, but doesn't fit this scheme
  Other,
}

impl fmt::Display for AmsEnv {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    use AmsEnv::*;
    let val = match self {
      Acknowledgement => "acknowledgement",
      Affirmation => "affirmation",
      Algorithm => "algorithm",
      Answer => "answer",
      Assumption => "assumption",
      Bound => "bound",
      Caption => "caption",
      Case => "case",
      Claim => "claim",
      Comment => "comment",
      Conclusion => "conclusion",
      Condition => "condition",
      Conjecture => "conjecture",
      Constraint => "constraint",
      Convention => "convention",
      Corollary => "corollary",
      Criterion => "criterion",
      Definition => "definition",
      Demonstration => "demonstration",
      Discussion => "discussion",
      Example => "example",
      Expansion => "expansion",
      Expectation => "expectation",
      Experiment => "experiment",
      Explanation => "explanation",
      Fact => "fact",
      Hint => "hint",
      Issue => "issue",
      Keywords => "keywords",
      Lemma => "lemma",
      Notation => "notation",
      Note => "note",
      Notice => "notice",
      Observation => "observation",
      Other => "other",
      Paragraph => "paragraph",
      Principle => "principle",
      Problem => "problem",
      Proof => "proof",
      Proposition => "proposition",
      Question => "question",
      Remark => "remark",
      Result => "result",
      Rule => "rule",
      Solution => "solution",
      Step => "step",
      Summary => "summary",
      Theorem => "theorem",
    };
    fmt.write_str(val)
  }
}

/// Maps a latexml-produced HTML class, such as "ltx_theorem ltx_theorem_lemma" to an `AmsEnv` enum
pub fn class_to_env(class: &str) -> Option<AmsEnv> {
  if class.is_empty() {
    None
  } else if !class.contains("ltx_theorem") {
    if class == "ltx_proof" {
      Some(AmsEnv::Proof)
    } else {
      None
    }
  } else {
    lazy_static! {
      static ref AMS_ENV: Regex = Regex::new(r"ltx_theorem_(\w+)").unwrap();
    }
    let env = match AMS_ENV.captures(class) {
      None => AmsEnv::Theorem, // simply "ltx_theorem" markup
      Some(caps) => normalize_env(caps.get(1).unwrap().as_str()),
    };
    Some(env)
  }
}

/// If known, maps a commonly used AMS environment to a shortlist of the 23 most notable
/// environments. Returns None if there is no AMS markup. Most experiments would be best off with
/// dropping the AmsEnv::Other resulting paragraphs, to avoid unintended dilution of the known
/// environments.
pub fn normalize_env(env: &str) -> AmsEnv {
  match env {
    "ack" | "ackn" | "ackno" | "acknow" | "acknowledge" | "acknowledgement"
    | "acknowledgements" | "acknowledgment" | "acknowledgments" | "acknowlegement" | "acks"
    | "thanks" => AmsEnv::Acknowledgement,
    "affirmation" => AmsEnv::Affirmation,
    "algm" | "alg" | "algo" | "algo1" | "algor" | "algor0" | "algorithm" | "algorithm1"
    | "algorithm2" | "algorithmdef" | "algorithme" | "algorithms" | "algoritmo"
    | "inneralgorithm" | "algx" | "heu" | "heur" | "heuristic" | "heuristics" | "myalgo"
    | "myalgorithm" | "meinalgorithmus" | "prealgorithm" | "protoalgorithm" => AmsEnv::Algorithm,
    "aassumption"
    | "as"
    | "asm"
    | "asmptn"
    | "asn"
    | "ass"
    | "ass1"
    | "asse"
    | "asser"
    | "assert"
    | "assertion"
    | "asslab"
    | "assm"
    | "assn"
    | "assnot"
    | "assp"
    | "assu"
    | "assuem"
    | "assum"
    | "assume"
    | "assump"
    | "assump1"
    | "assumpb"
    | "assumpc"
    | "assumpt"
    | "assumptio"
    | "assumption"
    | "assumption0"
    | "assumption1"
    | "assumption2"
    | "assumptiona"
    | "assumptionb"
    | "assumptionbis"
    | "assumptionc"
    | "assumptiond"
    | "assumptione"
    | "assumptionint"
    | "assumptionletter"
    | "assumptionm"
    | "assumptionmodel"
    | "assumptionparac"
    | "assumptionparad"
    | "assumptionstar"
    | "assumptionx"
    | "assums"
    | "asum"
    | "ax"
    | "axio"
    | "axiom"
    | "axioms"
    | "axm"
    | "gaxiom"
    | "innerassumption"
    | "itassumption"
    | "modasm"
    | "myass"
    | "myassmpt"
    | "myassump"
    | "myassumption"
    | "nnassumption"
    | "notationassumption"
    | "notationassumptions"
    | "number"
    | "postulate"
    | "postulation"
    | "prior"
    | "sassumption"
    | "shortassumption"
    | "sideassumption"
    | "simplifyingassumption"
    | "standingassumption"
    | "subsubaxiom" => AmsEnv::Assumption,
    "bound" => AmsEnv::Bound,
    "diag" | "fig" | "figcaption" | "figm" | "fignum" | "figure" | "figuretext" | "tab"
    | "tabel" | "tabl" | "tabla" | "table" | "tldiag" => AmsEnv::Caption,
    "case" | "case1" | "case2" | "case3" | "caseone" | "casestudy" | "casetwo"
    | "innercustomcase" | "mycase" | "scase" | "sscase" | "subcase" | "subcase2" | "subsubcase"
    | "subsubsubcase" | "tcase" => AmsEnv::Case,
    "aclaim" | "alphaclaim" | "boldclaim" | "cclaim" | "cdellsclaim" | "cdtopiclaim" | "cla" | "clai"
    | "claim" | "claim1" | "claim2" | "claim3" | "claim4" | "claim5" | "claima" | "claimapp"
    | "claimb" | "claimc" | "claimenv" | "claimfoo" | "claimi" | "claimlncs" | "claimm" | "claimn"
    | "claimnn" | "claimno" | "claimnr" | "claimnum" | "claimone" | "claimprop3" | "claimq"
    | "claims" | "claimstar" | "claimsub" | "claimx" | "clm" | "defclaim" | "innercustomclaim"
    | "internalclaim" | "itclaim" | "jmclaim" | "lblclaim" | "mainclaim" | "mclaim" | "megaclaim"
    | "misclaim" | "myclaim" |  "nclaim" |  "newclaim" |  "numberedclaim" |  "numclaim" |  "ourclaim"
    | "pclaim" | "prclaim" | "preclaim" | "procclaim" | "proclaim" | "proclaimmydef"
    | "quasiclaim" | "sclaim" | "proclaimmypreuve" | "subclai" | "subclaim" | "tclaim" | "tittoclaim"
    | "uclaim" | "varclaim" | "xclaim" => AmsEnv::Claim,
    "clcriterion" | "cnd" | "cond" | "condi" | "condition" | "conditiona" | "conditionb"
    | "conditionc" | "conditions" | "condn" | "conds" | "crit" | "criteria" | "innercondition"
    | "lcon" | "mycond" | "ncond" | "ocond" | "xcondition" => AmsEnv::Condition,
    "abconjecture" | "aconj" | "ansatz" | "cn" | "cnj" | "con" | "con1" | "con2" | "cona"
    | "conj" | "conj0" | "conj1" | "conj2" | "conja" | "conjb" | "conjc" | "conje" | "conjec"
    | "conject" | "conjecture" | "conjecture0" | "conjecture1" | "conjecture2" | "conjecturea"
    | "conjecturealpha" | "conjectureb" | "conjecturee" | "conjectureenv" | "conjectures"
    | "conjecturex" | "conjetura" | "conjintro" | "conjj" | "conjs" | "conjsn" | "conjstar"
    | "guess" | "guess1" | "guess2" | "guess3" | "guess8" | "hyp" | "hyp1" | "hypa" | "hypbase"
    | "hype" | "hypenglish" | "hypo" | "hypot" | "hypoth" | "hypothese" | "hypotheses"
    | "hypothesis" | "hyps" | "iconj" | "innerconjecture" | "innercustomhyp" | "introconj"
    | "introconjecture" | "itconjecture" | "mainconj" | "mainconjecture" | "mconj" | "myconj"
    | "myconjecture" | "ourconjecture" | "precon" | "preconj" | "rconjecture" | "sconj"
    | "sconjecture" | "subconj" => AmsEnv::Conjecture,
    "conv" | "conve" | "conventie" | "convention" | "conventionfoo" | "conventionn" | "conventions" => AmsEnv::Convention,
    "acorollary" | "apulause" | "bcorollary" | "bigcorollary" | "ccor" | "ccoro" | "ccorollary"
    | "cl" | "cllry" | "cnv" | "co" | "col" | "coll" | "collary" | "collolary" | "collorary"
    | "coly" | "comq" | "coor" | "cor" | "cor0" | "cor1" | "cor2" | "cor3" | "cor4" | "cor5"
    | "cora" | "corabc" | "coralph" | "corb" | "corbis" | "corc" | "cord" | "corl" | "cormy"
    | "cornr" | "coro" | "coro0" | "coro1" | "coro2" | "corob" | "coroc" | "corointro"
    | "corol" | "corolaire" | "corolario" | "corolary" | "coroll" | "coroll1" | "corolla"
    | "corollaire" | "corollaire2" | "corollaires" | "corollar" | "corollari" | "corollaries"
    | "corollario" | "corollarium" | "corollary" | "corollary0" | "corollary1" | "corollary2"
    | "corollary3" | "corollary4" | "corollarya" | "corollaryalpha" | "corollaryb"
    | "corollaryc" | "corollaryd" | "corollaryenv" | "corollaryfoo" | "corollaryi"
    | "corollaryinorder" | "corollaryint" | "corollaryintheorem" | "corollaryintro"
    | "corollaryk" | "corollarylemma" | "corollarylet" | "corollaryloc" | "corollarymain"
    | "corollaryn" | "corollarynn" | "corollarynonum" | "corollaryp" | "corollarys"
    | "corollaryst" | "corollaryth" | "corollaryx" | "corollaryy" | "corollory" | "corollp"
    | "coron" | "coroplain" | "coros" | "corqed" | "corr" | "correspondence"
    | "correspondence1" | "corrly" | "corro" | "corrol" | "corrolary" | "corrollary" | "corsub"
    | "cort" | "corx" | "cory" | "cri" | "crl" | "crl1" | "crl2" | "crll" | "crllr" | "crllry"
    | "crlr" | "crlre" | "crlry" | "crly" | "custom" | "cy" | "ecor" | "ecoro" | "exxe"
    | "gcorollary" | "icorollary" | "induction" | "inequality" | "innercorrep"
    | "innercustomcoro" | "introcorollary" | "itcorollary" | "ittheorem" | "kor" | "koro"
    | "korollar" | "lettercor" | "maincor" | "maincoro" | "maincorollary" | "mcoro"
    | "mcorollary" | "mcrl" | "mscorollary" | "mycol" | "mycor" | "mycoro" | "mycorol"
    | "mycorollary" | "mycorr" | "ncoro" | "newcorollary" | "newkorolari" | "nncorol"
    | "nncorollary" | "nonumbercorollary" | "ourcorollary" | "precor" | "precorol" | "rigor2"
    | "rmkk" | "scor" | "scorol" | "scorollary" | "subcorollary" | "supos" | "tcor"
    | "tcorollary" | "tem" | "theorex" | "thmcorollary" | "uncorollary" | "wn" | "xcor"
    | "xcorollary" => AmsEnv::Corollary,
    "criterion" => AmsEnv::Criterion,
    "1def"
    | "adefi"
    | "adefinition"
    | "adefinizione"
    | "adefn"
    | "appdefinition"
    | "bdefinition"
    | "bsubdefinition"
    | "citeddefn"
    | "cordef"
    | "cuhdef"
    | "d"
    | "d0"
    | "d2"
    | "dcldfn"
    | "ddd"
    | "ddefi"
    | "ddefinition"
    | "ddefn"
    | "de"
    | "deef"
    | "def"
    | "def1"
    | "def2"
    | "def21"
    | "def22"
    | "def3"
    | "def4"
    | "def5"
    | "def7"
    | "defa"
    | "defb"
    | "defc"
    | "defdefinition"
    | "defe"
    | "defen"
    | "defenglish"
    | "defenition"
    | "defex"
    | "deff"
    | "defff"
    | "deffie"
    | "defi"
    | "defi1"
    | "defi2"
    | "defif"
    | "defii"
    | "defin"
    | "defin1"
    | "defina"
    | "defination"
    | "definchapter"
    | "define"
    | "defined"
    | "definer"
    | "defini"
    | "definic1"
    | "definicao"
    | "definice"
    | "definicex"
    | "definicija"
    | "definicio"
    | "definicion"
    | "definicion2"
    | "definicja"
    | "definisjon"
    | "definit"
    | "definitia"
    | "definitie"
    | "definitin"
    | "definitio"
    | "definition"
    | "definition0"
    | "definition1"
    | "definition2"
    | "definition3"
    | "definition4"
    | "definition5"
    | "definitiona"
    | "definitionalph"
    | "definitionalpha"
    | "definitionat"
    | "definitionaux"
    | "definitionbase"
    | "definitioncore"
    | "definitioneng"
    | "definitionenv"
    | "definitionflat"
    | "definitionhack"
    | "definitionhead"
    | "definitionhelp"
    | "definitionint"
    | "definitionintro"
    | "definitionit"
    | "definitionk"
    | "definitionloc"
    | "definitionm"
    | "definitionn"
    | "definitionnonum"
    | "definitionnonumber"
    | "definitionplain"
    | "definitionrm"
    | "definitions"
    | "definitions1"
    | "definitionst"
    | "definitiont"
    | "definitiontemp"
    | "definitionvide"
    | "definitionx"
    | "definitn"
    | "definiton"
    | "definiz"
    | "definizione"
    | "definizioni"
    | "definrem"
    | "defins"
    | "defintion"
    | "defintro"
    | "defiplain"
    | "defipro"
    | "defis"
    | "defit"
    | "defiteo"
    | "defith"
    | "deflab"
    | "defm"
    | "defn"
    | "defn0"
    | "defn1"
    | "defn2"
    | "defn5"
    | "defna"
    | "defnc"
    | "defni"
    | "defnintro"
    | "defnm"
    | "defnn"
    | "defnonum"
    | "defnot"
    | "defnp"
    | "defnplain"
    | "defnrem"
    | "defns"
    | "defnsub"
    | "defnt"
    | "defo"
    | "defofentangidentical"
    | "defofentangidentical2"
    | "defqed"
    | "defs"
    | "defsatz"
    | "defstep"
    | "deft"
    | "deftemp"
    | "defx"
    | "defxxx"
    | "defy"
    | "defz"
    | "df2"
    | "dfa"
    | "dfafour"
    | "dfc"
    | "dfn"
    | "dfna"
    | "dfnbis"
    | "dfni"
    | "dfnl"
    | "dfnlem"
    | "dfnlm"
    | "dfnnr"
    | "dfns"
    | "dfnt"
    | "dfntn"
    | "dfnz"
    | "dfs"
    | "dft"
    | "dftemp"
    | "dftn"
    | "dn"
    | "dnt"
    | "dummydef"
    | "edef"
    | "edefi"
    | "edefin"
    | "edefinition"
    | "emdefi"
    | "emdefinition"
    | "engdef"
    | "envdef"
    | "fdef"
    | "fdefinition"
    | "fdefn"
    | "fed"
    | "fiebigdefinition"
    | "framednameddef"
    | "fsdefi"
    | "gdefinition"
    | "hdefn"
    | "idefinition"
    | "idfn"
    | "importantdefinition"
    | "innercustomdef"
    | "introdefi"
    | "introdefinition"
    | "introdefn"
    | "introdfn"
    | "italdeff"
    | "italicdefinition"
    | "itdef"
    | "itdefinition"
    | "jsvdef"
    | "knowndefinition"
    | "ldefinition"
    | "lemdefn"
    | "lemdfn"
    | "locdef"
    | "madef"
    | "maindef"
    | "maindefin"
    | "maindefinition"
    | "maindefn"
    | "mdefinition"
    | "mdefn"
    | "metadefinition"
    | "mydef"
    | "mydef1"
    | "mydef11"
    | "mydef12"
    | "mydef13"
    | "mydef2"
    | "mydef3"
    | "mydef4"
    | "mydef41"
    | "mydef42"
    | "mydef43"
    | "mydef5"
    | "mydef51"
    | "mydef52"
    | "mydef53"
    | "mydef6"
    | "mydef7"
    | "mydefc"
    | "mydefi"
    | "mydefine"
    | "mydefinition"
    | "mydefn"
    | "mydefname"
    | "mydefp"
    | "mydefs"
    | "nameddef"
    | "ndefi"
    | "ndefinition"
    | "nekdef"
    | "newdefine"
    | "newdefinition"
    | "nndefinition"
    | "numdef"
    | "opr"
    | "opred"
    | "owndefinition"
    | "pdef"
    | "pdefinition"
    | "peudefigura"
    | "predef"
    | "predefi"
    | "predefin"
    | "predefinition"
    | "predefn"
    | "predfn"
    | "prodef"
    | "prodefi"
    | "protodefinition"
    | "qtheorem"
    | "quasidefinition"
    | "rigdef"
    | "rmdefinitionplain"
    | "romandefinition"
    | "satzdef"
    | "sdef"
    | "sdefin"
    | "sdefinition"
    | "sdefn"
    | "ssdefn"
    | "stdef"
    | "stdefn"
    | "subdefinition"
    | "subdefn"
    | "szdfn"
    | "tdef"
    | "tdefinition"
    | "tempdefn"
    | "textofdefinition"
    | "thdef"
    | "thdefin"
    | "thedef"
    | "udefin"
    | "udefinition"
    | "udefn"
    | "vdef"
    | "xdef"
    | "xdefinition"
    | "xdefn"
    | "xtdef" => AmsEnv::Definition,
    "demonstration" => AmsEnv::Demonstration,
    "disc" | "discussion" | "notationanddiscussion" => AmsEnv::Discussion,
    "appexample" | "backtheorem" | "baseexample" | "beispiel" | "bexample" | "bigexample"
    | "bp" | "bsp" | "bsp1" | "bspe" | "cexample" | "cexpl" | "conda" | "counterexample"
    | "csexample" | "dexample" | "e1" | "eexam" | "eexample" | "eexemples" | "eg" | "ejem"
    | "emexample" | "emp" | "ex" | "ex1" | "ex2" | "ex3" | "ex4" | "exa" | "exaa" | "exam"
    | "exam1" | "exama" | "examfoo" | "examm" | "examp" | "exampl" | "example" | "example0"
    | "example1" | "example2" | "example3" | "examplea" | "exampleapp" | "exampleaux"
    | "exampleb" | "examplebase" | "examplec" | "examplecon" | "examplecore" | "exampledef"
    | "exampledummy" | "examplee" | "exampleem" | "exampleenv" | "examplehidden" | "examplehlp"
    | "examplei" | "exampleit" | "examplelist" | "exampleme" | "examplen" | "examplenn"
    | "examplenodiamond" | "examplenorm" | "exampleold" | "examplep" | "examplepf"
    | "exampleplain" | "examplerm" | "examples" | "examplescenario" | "exampletemp"
    | "exampleth" | "examplex" | "examplit" | "examps" | "exams" | "exas" | "exaxxx" | "exe"
    | "exe1" | "exem" | "exem2" | "exemp" | "exempl" | "exemple" | "exemplo" | "exex" | "exm"
    | "exmatmul" | "exmp" | "exmp3" | "exmpl" | "exmple" | "exmples" | "exmps" | "exp"
    | "expe" | "expl" | "expl2"
    | "exple" | "explo" | "expls" | "expltemap" | "exz" | "fexample" | "hexample" | "iexample"
    | "innercontexample" | "innerexample" | "introexample" | "itexample" | "mainex"
    | "mainexample" | "mexample" | "miex" | "minorexmp" | "myexam" | "myexample"
    | "myexampleplain" | "myexmp" | "newexample" | "nexample" | "nexp" | "nnexmp"
    | "nonexample" | "numberedexample" | "numexample" | "nxmpl" | "orexample" | "plcexample"
    | "preex" | "preexample" | "preexamples" | "prexample" | "proexample" | "rexample"
    | "rrexampleraw" | "runex" | "runningexample" | "sexample" | "subexample" | "texample"
    | "textofexample" | "theexample" | "theoremnl" | "varexample" | "xexample" | "xmpl" =>
      AmsEnv::Example,
    "experiment" => AmsEnv::Experiment,
    "explanation" => AmsEnv::Explanation,
    "expansion" => AmsEnv::Expansion,
    "expectation" => AmsEnv::Expectation,
    "principle" => AmsEnv::Principle,
    "algrule"
    | "arule"
    | "branchrule"
    | "brrule"
    | "brule"
    | "coordinates"
    | "crule"
    | "drule"
    | "grule"
    | "intruler"
    | "intrules"
    | "kernelrule"
    | "krule"
    | "mrule"
    | "myrule"
    | "polyrule"
    | "pruningrule"
    | "qrule"
    | "redrule"
    | "redrulebgvd"
    | "reducerule"
    | "reductionrule"
    | "rerule"
    | "rle"
    | "rrule"
    | "rul"
    | "rule"
    | "rule0"
    | "rules"
    | "rull"
    | "syntaxrule"
    | "trule"
    | "validrule"
    | "edgerule" => AmsEnv::Rule,
    | "definitionandfact"
    | "fac"
    | "fact"
    | "fact2"
    | "facta"
    | "factenv"
    | "factnum"
    | "factorizabilityidentical2"
    | "factpart"
    | "facts"
    | "factsub"
    | "factt"
    | "fakt"
    | "factfoo"
    | "faktum"
    | "fct"
    | "ffact"
    | "hfakt"
    | "myfact"
    | "nfact"
    | "ourfact"
    | "profact"
    | "romanfact"
    | "sfact"
    | "stylizedfact"
    | "subfact" => AmsEnv::Fact,
    "issue" => AmsEnv::Issue,
    "keywords" => AmsEnv::Keywords,
    "alemma" | "aplemma" | "appendixlemma" | "applemma" | "approximationlemma" | "appxlem"
    | "appxlemma" | "aslemma" | "assumptions" | "bigclm" | "blemma" | "defilemma"
    | "definitionlemma" | "deflemma" | "defnlem" | "dlemma" | "elem" | "elemme" | "envlem"
    | "exampletheoremenv" | "fiebiglemma" | "flemma" | "frmlemmasup" | "glemma"
    | "innercustomlem" | "innercustomlemma" | "intlemnp" | "itlemma" | "keylem" | "keylemma"
    | "klemma" | "la" | "laemma" | "lalpha" | "lam" | "le" | "le1" | "le2" | "lelele" | "lem"
    | "lem0" | "lem1" | "lem2" | "lem21" | "lem22" | "lem23" | "lem3" | "lem31" | "lem32"
    | "lema" | "lema1" | "lema2" | "lemanom" | "lemanonum" | "lemap" | "lemapp" | "lemas"
    | "lemat" | "lembr" | "lemenglish" | "lemenum" | "lemf" | "lemm" | "lemm1" | "lemm2"
    | "lemma" | "lemma0" | "lemma1" | "lemma10" | "lemma2" | "lemma23" | "lemma3" | "lemma4"
    | "lemma41" | "lemma5" | "lemma6" | "lemma7" | "lemma8" | "lemmaa" | "lemmaa1" | "lemmaam"
    | "lemmaapp" | "lemmaappendix" | "lemmaaux" | "lemmabase" | "lemmabis" | "lemmabody"
    | "lemmac" | "lemmacase" | "lemmad" | "lemmadef" | "lemmadefinition" | "lemmae"
    | "lemmaeng" | "lemmaenv" | "lemmaf" | "lemmafoo" | "lemmai" | "lemmain" | "lemmaint"
    | "lemmaintro" | "lemmait" | "lemmak" | "lemmaloc" | "lemman" | "lemmann" | "lemmanonum"
    | "lemmaprime" | "lemmas" | "lemmasec" | "lemmast" | "lemmastar" | "lemmastyles"
    | "lemmasub" | "lemmasubs" | "lemmasubsect" | "lemmata" | "lemmatweak" | "lemmaun"
    | "lemmaux" | "lemmax" | "lemme" | "lemme1" | "lemme2" | "lemming" | "lemmino" | "lemmm"
    | "lemmma" | "lemms" | "lemmx" | "lemmy" | "lemo" | "lemqed" | "lems" | "lemsec"
    | "letterlemma" | "lkadlemma" | "ll" | "llemma" | "lm" | "lma" | "lmb" | "lmm" | "lmm1"
    | "lmm2" | "lmma" | "lmmno" | "lms" | "locallemma" | "mainlem" | "mainlemma" | "mlem" | "mlemma"
    | "monlem" | "mydlem3" | "mylem" | "mylemm" | "mylemma" | "mylm" | "mylma" | "mylmm"
    | "newlemma" | "nlemma" | "nnlemma" | "nolem" | "nonolemma" | "nonumberlemma"
    | "nonumlemma" | "ntlemma" | "oldlemma" | "ourlemma" | "palemma" | "plemma" | "prealphlem"
    | "prelem" | "prelemm" | "prelemma" | "prolemma" | "quasilemma" | "quot"
    | "replemma" | "rmlemma" | "rmlemmaplain" | "seclemma" | "slemma" | "slemme" | "souslemme"
    | "starex" | "steplemma" | "sub" | "sublem" | "sublema" | "sublemm" | "sublemma" | "sublm"
    | "subsublemma" | "suplemma" | "technicallemma" | "textoflemma" | "theirlemma" | "thmlemma"
    | "tlemma" | "twistinglemma" | "unnumberedlemma" | "xlem" | "xlemm" | "xlemma"
    | "zamechanie" => AmsEnv::Lemma,
    // | "config" ??
    // | "defbemerkung" ??
    // | "definite" ??
    // | "df" ??
    "bsubnotation"
    | "definitionnotation"
    | "localnotation"
    | "name"
    | "naming"
    | "nb"
    | "not"
    | "notac"
    | "notacao"
    | "notacion"
    | "notarem"
    | "notas"
    | "notat"
    | "notation"
    | "notation0"
    | "notationa"
    | "notationandreminder"
    | "notationdefinition"
    | "notationn"
    | "notationnum"
    | "notations"
    | "notification"
    | "notn"
    | "notns"
    | "nt"
    | "ntn"
    | "prenotac"
    | "prenotation"
    | "prerem"
    | "protobody"
    | "remarkaux"
    | "remnotation"
    | "setting"
    | "setup"
    | "snotation"
    | "subcounter"
    | "term"
    | "terminology" => AmsEnv::Notation,
    "explain"
    | "parab"
    | "parag"
    | "paragr"
    | "paragrafonumerato"
    | "paragrafonumeratonome"
    | "paragraph"
    | "paragraphe"
    | "pargrph"
    | "ppar"
    | "ppara"
    | "pppar"
    | "restate"
    | "restateenv"
    | "sbpara"
    | "sect"
    | "subparag"
    | "subsec" => AmsEnv::Paragraph,
    "classproblem"
    | "condb"
    | "coreproblem"
    | "corollaryin"
    | "cproblem"
    | "eioproblem"
    | "bwexerc"
    | "exc"
    | "exer"
    | "exercice"
    | "exercise"
    | "exercise0"
    | "exercisee"
    | "exercises"
    | "exes"
    | "exrc"
    | "exs"
    | "fproblem"
    | "hwproblem"
    | "introproblem"
    | "lem44"
    | "mainprob"
    | "mainproblem"
    | "myexe"
    | "myprob"
    | "myproblem"
    | "obv"
    | "open"
    | "openpb"
    | "openprob"
    | "openproblem"
    | "openq"
    | "opprob"
    | "oprob"
    | "oproblem"
    | "optimizationproblem"
    | "pb"
    | "pblm"
    | "pbm"
    | "prb"
    | "prbl"
    | "prblm"
    | "prbm"
    | "preprb"
    | "preprob"
    | "prob"
    | "prob1"
    | "prob2"
    | "proba"
    | "probalph"
    | "probdefi"
    | "probl"
    | "problem"
    | "problem1"
    | "problema"
    | "problemb"
    | "probleme"
    | "probleml"
    | "problems"
    | "problemz"
    | "problm"
    | "probs"
    | "probstatement"
    | "resprob"
    | "rhproblem"
    | "subproblem"
    | "testproblem"
    | "thmalg"
    | "tprob"
    | "xca"
    | "xopen" => AmsEnv::Problem,
    "afirmativa" | "beweis" | "claimproof" | "clproof" | "cproof" | "cproofa" | "cproofb"
    | "eproof" | "lemproof" | "mproof" | "myproo" | "myproof" | "namedproof"
    | "notationinproof" | "oldproof" | "pf" | "pprf" | "preproof" | "preprooff" | "prf"
    | "proof" | "proof0" | "proof1" | "proof2" | "proof3" | "proof4" | "proof5" | "proofa"
    | "proofaux" | "proofcase" | "proofclaim" | "prooff" | "prooffact" | "proofhead"
    | "proofidea" | "prooflem" | "proofn" | "proofof" | "proofoftheorem" | "proofpart"
    | "proofprop" | "proofsketch" | "proofth" | "prooftheorem" | "proofthm" | "proofx"
    | "sketchofproof" | "theoremproof" | "xproof" => AmsEnv::Proof,
    "5proposition"
    // | "a4"
    // | "a5"
    | "approp"
    | "appxprop"
    | "aprop"
    | "aproposition"
    | "bproposition"
    | "dclprop"
    | "definitionproposition"
    | "defiprop"
    | "defnprop"
    | "defprop"
    | "demosprop"
    | "dfnprop"
    | "dfprop"
    | "dproposition"
    | "eprop"
    | "eproposition"
    | "fait"
    | "fiebigproposition"
    | "fprop"
    | "fsprop"
    | "gproposition"
    | "gstatement"
    | "statement"
    | "stmt"
    | "subprop"
    | "hprop"
    | "hproposition"
    | "hyllprop"
    | "innercustomprop"
    | "innercustomproposition"
    | "introprop"
    | "introproposition"
    | "iprop"
    | "iproposition"
    | "itproposition"
    | "kmprop"
    | "kspproposition"
    // | "l3"
    | "lprop"
    | "lproposition"
    | "mainprop"
    | "mainproposition"
    | "maprop"
    | "mathproposition"
    | "mdprop"
    | "mpro"
    | "mprop"
    | "mproposition"
    | "msproposition"
    | "myprop"
    | "mypropd"
    | "myproposition"
    | "myprp"
    | "myproperty"
    | "namedprop"
    | "newprop"
    | "newproposition"
    | "nnprop"
    | "nnproposition"
    | "nonumberproposition"
    | "nprop"
    | "nproposition"
    // | "num" ?
    | "numprop"
    | "ourproposition"
    | "p"
    | "pn"
    | "pp"
    | "ppn"
    | "ppro"
    | "pproposition"
    | "ppsn"
    | "pr"
    | "pr1"
    | "pr4"
    | "pr5"
    | "pred"
    | "predl"
    | "prepos"
    | "preposition"
    | "preprop"
    | "preproposition"
    | "pro"
    | "pro1"
    | "pro2"
    | "pro3"
    | "pro4"
    | "pro5"
    | "prop"
    | "prop0"
    | "prop1"
    | "prop2"
    | "prop22"
    | "prop23"
    | "prop3"
    | "prop31"
    | "prop33"
    | "prop4"
    | "prop5"
    | "prop52"
    | "prop6"
    | "prop7"
    | "propa"
    | "propaat"
    | "propal"
    | "propalg"
    | "propalph"
    | "propalpha"
    | "propandef"
    | "propasympcap"
    | "propaux"
    | "propb"
    | "propbibl"
    | "propbis"
    | "propc"
    | "propconstr"
    | "propd"
    | "propdef"
    | "propdefi"
    | "propdefn"
    | "propdfn"
    | "prope"
    | "proper"
    | "properti"
    | "properties"
    | "property"
    | "propf"
    | "propgl"
    | "propi"
    | "propie"
    | "propiedad"
    | "proping"
    | "propint"
    | "propintro"
    | "propm"
    | "propmy"
    | "propn"
    | "propnon"
    | "propnonnum"
    | "propnr"
    | "propns"
    | "propnum"
    | "propo"
    | "propo1"
    | "propoa"
    | "propoplain"
    | "propos"
    | "proposal"
    | "propose"
    | "proposi"
    | "proposicao"
    | "proposicio"
    | "proposicion"
    | "proposit"
    | "propositin"
    | "propositio"
    | "proposition"
    | "proposition0"
    | "proposition1"
    | "proposition2"
    | "proposition3"
    | "propositiona"
    | "propositionalpha"
    | "propositionam"
    | "propositionapp"
    | "propositionappendix"
    | "propositionb"
    | "propositionbase"
    | "propositioncommand"
    | "propositiondefinition"
    | "propositionenv"
    | "propositionloc"
    | "propositionn"
    | "propositionnoadvance"
    | "propositionnon"
    | "propositionnum"
    | "propositionnumthm"
    | "propositionp"
    | "propositions"
    | "propositionst"
    | "propositionsubsect"
    | "propositionx"
    | "proposiz"
    | "proposizione"
    | "propozycja"
    | "propp"
    | "propposition"
    | "propr"
    | "propri"
    | "propriedade"
    | "propriete"
    | "proprietes"
    | "proprop"
    | "props"
    | "propstar"
    | "propsub"
    | "propsubs"
    | "propt"
    | "proptweak"
    | "propty"
    | "propx"
    | "propy"
    | "pros"
    | "prp"
    | "prp1"
    | "prp2"
    | "prpd"
    | "prpl"
    | "prpn"
    | "prpp"
    | "prps"
    | "prpstn"
    | "prpsub"
    | "prpsubf"
    | "prpt"
    | "prrop"
    | "refprop"
    | "sat"
    | "sbprop"
    | "secprop"
    | "sectionprop"
    | "spprop"
    | "spropo"
    | "sproposition"
    | "refine"
    | "restatement"
    | "state"
    | "statem"
    | "statement1"
    | "statm"
    | "statment"
    | "stprop"
    // | "strengths"
    | "stw"
    | "subproperty"
    | "subproposition"
    | "subprops"
    | "supproposition"
    | "surprop"
    | "tempprop"
    | "thp"
    | "thprop"
    | "tprop"
    | "varprop"
    // | "weaknesses"
    | "wproposition"
    | "xprop"
    | "xproposition" => AmsEnv::Proposition,
    "boldquestion" | "emquestion" | "innerquestion" | "introquestion" | "mainquestion"
    | "myquest" | "myquestion" | "op" | "openquestion" | "prequestion" | "puzzle" | "q" | "qn"
    | "qst" | "qstn" | "qtn" | "qu" | "que" | "que1" | "query" | "ques" | "quesb" | "quess"
    | "quest" | "quest0" | "question" | "question0" | "question1" | "question2" | "question3"
    | "questionapp" | "questionb" | "questioni" | "questionintro" | "questions" | "queststar"
    | "researchquestion" | "rquestion" | "subquestion" | "varquestion" | "vopros" => AmsEnv::Question,
    // Extracted from original big parent Remark category:
    "mycomment"| "comment" | "commentary" | "comments" => AmsEnv::Comment,
    "note" | "lnote" | "mynote" => AmsEnv::Note,
    "notice" => AmsEnv::Notice,
    "hint" => AmsEnv::Hint,
    | "localobservation"
    | "myobservation"
    | "myoss"
    | "ob"
    | "obs"
    | "observ"
    | "observacao"
    | "observacio"
    | "observacion"
    | "observation"
    | "observations"
    | "observe"
    | "observen"
    | "obss"
    | "os"
    | "oss"
    | "osse"
    | "osserv"
    | "osserva"
    | "osservazione"
    | "preobserv" => AmsEnv::Observation,
    "summary" => AmsEnv::Summary,
    "a3remark"
    // | "advice" ???
    | "aremark"
    | "auxremark"
    | "baseremark"
    | "bem"
    | "bemerkung"
    | "bigremark"
    | "bremark"
    | "bremarknote"
    | "brmk"
    | "bsubremarknote"
    // | "context" ???
    // | "definitionandremark" ??? (between classes)
    | "definitionremark"
    | "defremark"
    | "deno"
    | "dummyrem"
    | "emrem"
    | "emremark"
    | "eremark"
    | "introremark"
    | "iremark"
    | "itremark"
    | "kmremark"
    | "localremark"
    | "mcc"
    | "miniremark"
    | "mirem"
    | "mrem"
    | "mremark"
    | "myrek"
    | "myrem"
    | "myrema"
    | "myremark"
    | "myremarks"
    | "myrems"
    | "myrm"
    | "myrmk"
    | "newremark"
    | "nnremark"
    // | "nota" ???
    // | "notationandremark" ??? (between classes)
    | "nremark"
    | "nrmk"
    | "nrmks"
    | "nrmrk"
    | "ntremark"
    | "numberedremark"
    | "numremark"
    | "numrk"
    | "numrmk"
    // | "para" ???
    // "point" => AmsEnv::Point,
    | "plainremarks"
    | "preremark"
    | "preremark2"
    | "proremark"
    | "protoremark"
    // | "punto"
    | "r"
    | "re"
    | "reem"
    | "rek"
    | "rem"
    | "rem0"
    | "rem1"
    | "rem3"
    | "rem5"
    | "rema"
    | "remak"
    | "remar"
    | "remark"
    | "remark0"
    | "remark1"
    | "remark2"
    | "remark3"
    | "remark4"
    | "remark5"
    | "remark6"
    | "remarka"
    | "remarkapp"
    | "remarkat"
    | "remarkb"
    | "remarkbase"
    | "remarkdef"
    | "remarkdefinition"
    | "remarke"
    | "remarkeng"
    | "remarkenv"
    | "remarkf"
    | "remarkhelp"
    | "remarki"
    | "remarkinorder"
    | "remarkintro"
    | "remarkk"
    | "remarkl"
    | "remarkm"
    | "remarkn"
    | "remarknodiamond"
    | "remarknon"
    | "remarknonum"
    | "remarknonumber"
    | "remarknorm"
    | "remarknum"
    | "remarknumb"
    | "remarko"
    | "remarkplain"
    | "remarkpro"
    | "remarkr"
    | "remarks"
    | "remarkstar"
    | "remarksub"
    | "remarktemp"
    | "remarkth"
    | "remarkthe"
    | "remarku"
    | "remarkunnumbered"
    | "remarkwr"
    | "remarkx"
    | "remarq" | "remarque" | "remarques" | "remarquesubsect"
    | "remarque2"
    | "rembold"
    | "reme"
    | "remf"
    | "remk"
    | "remnonumber"
    | "remrk"
    | "rems"
    | "remsgl"
    | "remxxx"
    | "rk"
    | "rm"
    | "rmk"
    | "rmk0"
    | "rmk1"
    | "rmk2"
    | "rmka"
    | "rmkks"
    | "rmknr"
    | "rmkp"
    | "rmks"
    | "rmksub"
    | "rmktemp"
    | "rmq"
    | "rmqs"
    | "rmr"
    | "rmr2"
    | "rmrk"
    | "rmrq"
    | "romanremark"
    | "rq"
    | "rqe1"
    | "rque"
    | "rremark"
    // | "say" ???
    | "sideremark"
    | "sidermk"
    // | "sit" ???
    | "smallremark"
    | "sremark"
    | "srmk"
    | "subremark"
    | "thremark"
    | "topology"
    | "torsionremark"
    | "unnumberedremark"
    | "unnumrem"
    | "unremark"
    | "unremarks"
    | "uremark"
    | "uw"
    | "varremark"
    | "vetremark"
    | "xrem"
    | "xremark"
    | "xrmk"
    | "zero" => AmsEnv::Remark,
    "answer" => AmsEnv::Answer,
    "conclude" | "conclusion" | "conclusions" => AmsEnv::Conclusion,
    "final" | "mainresult" | "mdresult" | "numres" | "priorresults" | "res" | "resu" | "resul" | "result" | "resultat" | "results" | "resump" => AmsEnv::Result,
    "solution" | "solutions" => AmsEnv::Solution,
    "cons" | "constr" | "constraint" | "constraints" | "constrinternal" => AmsEnv::Constraint,
    "art" | "astep" | "chunk" | "construct" | "construction" | "constructions" | "cstep" | "emf"
    | "nothing" | "ournothing" | "pstep" | "reduction" | "require" | "stage" | "step" | "step1"
    | "step2" | "step3" | "step4" | "step5" | "stepa" | "stepb" | "stepmain" | "stepn"
    | "stepnamed" | "stepnn" | "stepp" | "stepwise" | "substep" | "ttt" => AmsEnv::Step,
    "5theorem"
    | "a3theorem"
    | "abctheorem"
    | "abcthm"
    | "algthm"
    | "alphatheorem"
    | "alphathm"
    | "alphtheorem"
    | "alphthm"
    | "alpthm"
    | "appendthm"
    | "apptheorem"
    | "appthm"
    | "appxthm"
    | "astheorem"
    | "atheo"
    | "atheorem"
    | "athm"
    | "bbthm"
    | "bigteo"
    | "bigtheo"
    | "bigtheorem"
    | "bigthm"
    | "blanktheorem"
    | "btheo"
    | "btheorem"
    | "bthm"
    | "bwtheorem"
    | "cbthm"
    | "citedtheorem"
    | "citedthm"
    | "citetheorem"
    | "citethm"
    | "citingtheorem"
    | "citingthm"
    | "classicaltheorem"
    | "corthm"
    | "ctheorem"
    | "cthm"
    | "cuhtheorem"
    | "dclthm"
    | "definitiontheorem"
    | "defitheo"
    | "defithm"
    | "defnthm"
    | "deftheorem"
    | "defthm"
    | "dthm"
    | "emptytheorem"
    | "emptythm"
    | "envthm"
    | "etheo"
    | "etheorem"
    | "ethm"
    | "examplethm"
    | "externaltheorem"
    | "exthm"
    | "extthm"
    | "fiebigtheorem"
    | "ftheo"
    | "ftheorem"
    | "fthm"
    | "generictheorem"
    | "genericthm"
    | "globaltheorem"
    | "gtheorem"
    | "hthm"
    | "informaltheorem"
    | "infthm"
    | "innercustomtheorem"
    | "innercustomthm"
    | "innertheorem"
    | "innerthm"
    | "introth"
    | "introtheo"
    | "introtheorem"
    | "introthm"
    | "inttheorem"
    | "intthm"
    | "intthmnp"
    | "itheorem"
    | "ithm"
    | "jthm"
    | "keytheorem"
    | "keythm"
    | "knownthm"
    | "kthm"
    | "letteredtheorem"
    | "lettertheorem"
    | "letterthm"
    | "letthm"
    | "lgrthm"
    | "localtheorem"
    | "ltheorem"
    | "lthm"
    | "main"
    | "main1"
    | "main2"
    | "main3"
    | "maina"
    | "mainb"
    | "maint"
    | "mainteo"
    | "mainth"
    | "maintheo"
    | "maintheorem"
    | "maintheorem1"
    | "maintheorem2"
    | "maintheorema"
    | "maintheoremb"
    | "maintheorems"
    | "mainthm"
    | "mainthm2"
    | "mainthma"
    | "mainthmb"
    | "mainthmintro"
    | "mainthms"
    | "mainthrm"
    | "metatheorem"
    | "metathm"
    | "minithm"
    | "mmtheorem"
    | "montheo"
    | "mstheorem"
    | "mtheo"
    | "mtheorem"
    | "mthm"
    | "mthm2"
    | "mthma"
    | "mthmb"
    | "mydeftheorem"
    | "myptheorem"
    | "myteo"
    | "myth"
    | "mytheo"
    | "mytheorem"
    | "mythm"
    | "mythmd"
    | "mythname"
    | "mythrm"
    | "namedtheorem"
    | "namedthm"
    | "newteorem"
    | "newtheorem"
    | "newthm"
    | "nntheorem"
    | "nnthm"
    | "nonumbertheorem"
    | "nonumberthm"
    | "nonumtheorem"
    | "nonumthm"
    | "normaltheorem"
    | "nostheorem"
    | "notheorem"
    | "nteo"
    | "ntheorem"
    | "nthm"
    | "ntt"
    | "nttn"
    | "numlessthm"
    | "numthm"
    | "oldtheorem"
    | "oldthm"
    | "otheorem"
    | "other"
    | "otherl"
    | "otherth"
    | "othertheorem"
    | "otherthm"
    | "othm"
    | "ourtheorem"
    | "ourthm"
    | "pkt"
    | "prealphthm"
    | "pretheo"
    | "pretheorem"
    | "pretheorema"
    | "prethm"
    | "prevtheorem"
    | "primetheorem"
    | "priteo"
    | "proclaimmythm"
    | "proctheorem"
    | "prothe"
    | "psfiguretheo"
    | "ptctheorem"
    | "ptheorem"
    | "qthm"
    | "quasitheorem"
    | "quotethm"
    | "rawnamedtheorem"
    | "referencetheorem"
    | "reftheorem"
    | "refthm"
    | "remarkaftertheorem"
    | "remthm"
    | "repeatthm"
    | "reptheorem"
    | "repthm"
    | "retheorem"
    | "rethm"
    | "rmtheorem"
    | "rmtheoremplain"
    | "roughtheorem"
    | "rteorema"
    | "rtheorem"
    | "rthm"
    | "satz"
    | "sbthm"
    | "sec3thm1"
    | "secondtheorem"
    | "secthm"
    | "smalltheorem"
    | "snthm"
    | "specialthm"
    | "sstheorem"
    | "ssthm"
    | "st"
    | "stat"
    | "stheorem"
    | "stheoreme"
    | "sthm"
    | "stthm"
    | "subth"
    | "subtheorem"
    | "subthm"
    | "surtheoreme"
    | "surthm"
    | "szthm"
    | "t"
    | "t1"
    | "t32"
    | "t41"
    | "t5"
    | "taggedtheoremx"
    | "talpha"
    | "te"
    | "tempthm"
    | "teo"
    | "teo1"
    | "teo2"
    | "teoa"
    | "teoalpha"
    | "teob"
    | "teoc"
    | "teoi"
    | "teointro"
    | "teon"
    | "teononum"
    | "teoo"
    | "teor"
    | "teor2"
    | "teora"
    | "teore"
    | "teoreema"
    | "teorem"
    | "teorema"
    | "teorema1"
    | "teoremab"
    | "teoremac"
    | "teoru"
    | "teos"
    | "textoftheorem"
    | "th"
    | "th1"
    | "th11"
    | "th12"
    | "th13"
    | "th2"
    | "th3"
    | "th4"
    | "th5"
    | "th6"
    | "th7"
    | "th8"
    | "th9"
    | "the"
    | "the1"
    | "theirtheorem"
    | "them"
    | "theo"
    | "theo1"
    | "theo2"
    | "theo3"
    | "theo4"
    | "theoa"
    | "theoaa"
    | "theoalph"
    | "theoalpha"
    | "theoapp"
    | "theoaux"
    | "theob"
    | "theobis"
    | "theoc"
    | "theocite"
    | "theod"
    | "theodef"
    | "theoe"
    | "theoenglish"
    | "theoext"
    | "theof"
    | "theog"
    | "theoi"
    | "theoi1"
    | "theoint"
    | "theointr"
    | "theointro"
    | "theom"
    | "theomain"
    | "theon"
    | "theononnum"
    | "theoo"
    | "theop"
    | "theor"
    | "theor1"
    | "theore"
    | "theorem"
    | "theorem0"
    | "theorem1"
    | "theorem11"
    | "theorem14"
    | "theorem2"
    | "theorem3"
    | "theorem31"
    | "theorem4"
    | "theorem5"
    | "theorem6"
    | "theorem7"
    | "theorema"
    | "theoremabc"
    | "theoremain"
    | "theoremalph"
    | "theoremalpha"
    | "theoremanddefinition"
    | "theoremann"
    | "theoremapp"
    | "theoremappendix"
    | "theoremaux"
    | "theoremb"
    | "theorembase"
    | "theorembk"
    | "theoremc"
    | "theoremcite"
    | "theoremcited"
    | "theoremconstruction"
    | "theoremd"
    | "theoremdef"
    | "theoremdefinition"
    | "theoremdemo"
    | "theoreme"
    | "theoremempty"
    | "theoremeng"
    | "theoremenv"
    | "theoremf"
    | "theoremfoo"
    | "theoremg"
    | "theoremh"
    | "theoremi"
    | "theoremii"
    | "theoremin"
    | "theoreminorder"
    | "theoremint"
    | "theoremintro"
    | "theoreml"
    | "theoremlet"
    | "theoremletter"
    | "theoremletters"
    | "theoremloc"
    | "theoremm"
    | "theoremmain"
    | "theoremmm"
    | "theoremn"
    | "theoremname"
    | "theoremnamed"
    | "theoremnew1"
    | "theoremnn"
    | "theoremno"
    | "theoremnon"
    | "theoremnonum"
    | "theoremnonumber"
    | "theoremnum"
    | "theoremone"
    | "theoremothers"
    | "theoremp"
    | "theoremq"
    | "theoremquote"
    | "theoremrinner"
    | "theoremroman"
    | "theorems"
    | "theorems1"
    | "theoremsec"
    | "theoremsection"
    | "theoremsn"
    | "theoremst"
    | "theoremstar"
    | "theoremsubsubsect"
    | "theoremu"
    | "theoremun"
    | "theoremunnum"
    | "theoremvoid"
    | "theoremx"
    | "theoremz"
    | "theorintro"
    | "theorm"
    | "theorsect"
    | "theory"
    | "theos"
    | "theosec"
    | "theostar"
    | "theoun"
    | "theoy"
    | "ther"
    | "therm"
    | "thero"
    | "thh"
    | "thhilf"
    | "thintro"
    | "thm"
    | "thm0"
    | "thm1"
    | "thm10"
    | "thm11"
    | "thm12"
    | "thm13"
    | "thm1a"
    | "thm2"
    | "thm21"
    | "thm3"
    | "thm31"
    | "thm4"
    | "thm41"
    | "thm42"
    | "thm5"
    | "thm51"
    | "thm6"
    | "thm7"
    | "thm8"
    | "thm9"
    | "thma"
    | "thma1"
    | "thma2"
    | "thmaa"
    | "thmab"
    | "thmabc"
    | "thmain"
    | "thmalph"
    | "thmalpha"
    | "thmap"
    | "thmapp"
    | "thmappsec"
    | "thmasmp"
    | "thmast"
    | "thmb"
    | "thmb1"
    | "thmbibl"
    | "thmbis"
    | "thmbody"
    | "thmc"
    | "thmchapter"
    | "thmcite"
    | "thmcor"
    | "thmd"
    | "thmdef"
    | "thmdefinition"
    | "thmdefn"
    | "thme"
    | "thmeg"
    | "thmempty"
    | "thmenv"
    | "thmetoile"
    | "thmext"
    | "thmf"
    | "thmg"
    | "thmgl"
    | "thmh"
    | "thmi"
    | "thmii"
    | "thmiii"
    | "thmin"
    | "thmint"
    | "thmintr"
    | "thmintro"
    | "thmk"
    | "thml"
    | "thmlabel"
    | "thmlem"
    | "thmletter"
    | "thmlit"
    | "thmm"
    | "thmmain"
    | "thmn"
    | "thmnn"
    | "thmno"
    | "thmnodot"
    | "thmnon"
    | "thmnonnum"
    | "thmnonum"
    | "thmnonumber"
    | "thmnr"
    | "thmns"
    | "thmnum"
    | "thmo"
    | "thmothers"
    | "thmp"
    | "thmpart"
    | "thmplain"
    | "thmprime"
    | "thmq"
    | "thmqed"
    | "thmquote"
    | "thmr"
    | "thmref"
    | "thmresult"
    | "thmrlwe"
    | "thmrule"
    | "thms"
    | "thmsec"
    | "thmsect"
    | "thmspec"
    | "thmspecial"
    | "thmstar"
    | "thmsub"
    | "thmszn"
    | "thmt"
    | "thmtheorem"
    | "thmthm"
    | "thmtweak"
    | "thmtwo"
    | "thmu"
    | "thmuncount"
    | "thmw"
    | "thmwn"
    | "thmx"
    | "thmy"
    | "thmz"
    | "thr"
    | "thrm"
    | "thrm1"
    | "thrm2"
    | "thrma"
    | "thrmb"
    | "ths"
    | "thtable"
    | "ththm"
    | "titletheo"
    | "tm"
    | "tm2"
    | "tm3"
    | "tm4"
    | "tm5"
    | "tm6"
    | "tm7"
    | "tm8"
    | "tmnl"
    | "trm"
    | "trma"
    | "tteo"
    | "ttheo"
    | "ttheorem"
    | "tthm"
    | "ttm"
    | "tw"
    | "unnumberedtheorem"
    | "unnumberedthm"
    | "unnumthm"
    | "untheorem"
    | "unthm"
    | "utheorem"
    | "uthm"
    | "vartheorem"
    | "varthm"
    | "varthrm"
    | "void"
    | "vthm"
    | "xtheo"
    | "xtheorem"
    | "xthm" => AmsEnv::Theorem,
    _ => AmsEnv::Other,
  }
}
