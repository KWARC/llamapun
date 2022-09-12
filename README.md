The **llamapun** library contains _language and mathematics processing_ algorithms, used by the KWARC research group.

As of 2022, this repository can be considered in **maintenance** mode, as no further development is planned.

[![Build Status](https://github.com/KWARC/llamapun/workflows/CI/badge.svg)](https://github.com/KWARC/llamapun/actions?query=workflow%3ACI)
[![API Documentation](https://img.shields.io/badge/docs-API-blue.svg)](http://kwarc.github.io/llamapun/llamapun/index.html)
[![license](http://img.shields.io/badge/license-GPLv3-blue.svg)](https://raw.githubusercontent.com/KWARC/llamapun/master/LICENSE)
![version](https://img.shields.io/badge/version-0.3.4-orange.svg)
---

At its core, **llamapun** is a [Rust](http://rust-lang.org/) implementation that aims at minimal footprint and optimal runtime, in order to safely scale to corpora of millions of documents and tens of billions ot tokens.

<img width="25%" src="https://i.imgur.com/GLL3SVc.png" alt="llamapun logo" align="right">

Requires **stable** rust, starting from `rustc 1.34.0 (91856ed52 2019-04-10)`.

### Features

 * **Source Data**
   * Built-in support for STEM documents in ([LaTeXML-flavoured](https://github.com/brucemiller/LaTeXML/)) HTML5.

 * **Preprocessing**
   * Unicode normalization,
   * Stopwords - based on widely accepted lists, enhanced for STEM texts,
   * Semi-structured to plain text normalization (math, citations, tables, etc.),
   * [TODO #3] Purification of text and math modality (e.g. move trailing dots left in math back into the sentence text),
   * Stemming - adaptation of the [Morpha](http://www.sussex.ac.uk/Users/johnca/morph.html) stemmer,
   * Tokenization - rule-based sentence segmentation, and [SENNA](http://web.archive.org/web/20140208134927/http://ml.nec-labs.com/senna/) word tokenization

 * **Shallow Analysis**
   * Part-of-speech tagging (via [SENNA](http://web.archive.org/web/20140208134927/http://ml.nec-labs.com/senna/)),
   * Named Entity recognition (via [SENNA](http://web.archive.org/web/20140208134927/http://ml.nec-labs.com/senna/)),
   * Chunking and shallow parsing (via [SENNA](http://web.archive.org/web/20140208134927/http://ml.nec-labs.com/senna/)),
   * Extract token models for [GloVe](http://nlp.stanford.edu/projects/glove/),
   * [Pattern-matching library](doc/pattern_matching.md) for rule-based extraction and/or bootstrapping,
   * Language identification (via [whatlang](https://crates.io/crates/whatlang)),
   * N-gram footprints

 * **Representation Toolkit**
   * Document Narrative Model (DNM) addition to the XML DOM
   * XPointer and string offset annotation support
   * [TOPORT] Shared Packed parse forests for mathematical formulas (aka "disjunctive logical forms")

 * **Programming API**
   * High-level iterators over the narrative elements of scientific documents
   * Zero-cost abstractions over the source data, as well as over linguistic annotations of various granularity.
   * High-throughput parallel processing via `rayon`, since [0.3.0](https://github.com/KWARC/llamapun/releases/tag/0.3.0).

 * **Additional included examples**
   * math-aware corpus token models, via DNM plain text normalization
   * math-aware [dataset extraction](examples/corpus_statement_paragraphs_model.rs) for "statement classification" of paragraphs
   * "node footprint" statistics for corpora, e.g. [informing the MathML4 effort](https://github.com/mathml-refresh/mathml/issues/55#issuecomment-475916070)
   * track sibling words to inline references in scientific articles, [informing LaTeXML development](https://github.com/brucemiller/LaTeXML/issues/1043#issuecomment-478249149)

---

**Disclaimers:**

  1. Please remember that all third-party tools (such as the [SENNA](http://web.archive.org/web/20140208134927/http://ml.nec-labs.com/senna/) NLP toolkit) enforce their own licensing constraints.

  2. This Github repository is a successor to the now deprecated [C+Perl LLaMaPUn implementation](https://github.com/KWARC/deprecated-LLaMaPUn).
