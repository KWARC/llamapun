The **llamapun** library hosts common _language and mathematics processing_ algorithms, used by the KWARC research group.

[![Build Status](https://secure.travis-ci.org/KWARC/llamapun.png?branch=master)](http://travis-ci.org/KWARC/llamapun)
[![API Documentation](https://img.shields.io/badge/docs-API-blue.svg)](http://kwarc.github.io/llamapun/llamapun/index.html)
[![license](http://img.shields.io/badge/license-GPLv3-blue.svg)](https://raw.githubusercontent.com/KWARC/llamapun/master/LICENSE)

---
At its core, **llamapun** is a [Rust](http://rust-lang.org/) implementation that aims at minimal footprint and optimal runtime, in order to safely scale to corpora of millions of documents and tens of billions ot tokens.

### Features

 * **Source Data**
   * Built-in support for STEM documents in ([LaTeXML-flavoured](https://github.com/brucemiller/LaTeXML/)) HTML5.

 * **Preprocessing**
   * Unicode normalization,
   * Stopwords - based on widely accepted lists, enhanced for STEM texts,
   * Semi-structured to plain text normalization (math, citations, tables, etc.),
   * [TODO] Purification of text and math modality (e.g. move trailing dots left in math back into the sentence text),
   * Stemming - adaptation of the [Morpha](http://www.sussex.ac.uk/Users/johnca/morph.html) stemmer,
   * Tokenization - rule-based sentence segmentation, and [SENNA](http://ml.nec-labs.com/senna/) word tokenization

 * **Shallow Analysis**
   * Part-of-speech tagging (via [SENNA](http://ml.nec-labs.com/senna/)),
   * Named Entity recognition (via [SENNA](http://ml.nec-labs.com/senna/)),
   * Chunking and shallow parsing (via [SENNA](http://ml.nec-labs.com/senna/)),
   * Extract token models for [GloVe](http://nlp.stanford.edu/projects/glove/)
   * [TODO] Language identification (via [libTextCat](http://software.wise-guys.nl/libtextcat/)),
   * N-gram footprints,

 * **Representation Toolkit**
   * Document Narrative Model (DNM) addition to the XML DOM
   * [TODO] XPointer and string offset annotation support
   * [TOPORT] Shared Packed parse forests for mathematical formulas (aka "disjunctive logical forms")

 * **Programming API**
   * High-level iterators over the narrative elements of scientific documents
   * Zero-cost abstractions over the source data, as well as over linguistic annotations of various granularity.

---

**Getting started**

Run
```bash
cargo build
```
in the project directory. 

In case of errors, it's recommended to switch to the nightly builds of rust (https://github.com/rust-lang-nursery/rustup.rs#working-with-nightly-rust), 
i.e. using rustup (www.rustup.rs) and keep it updated (run 'rustup update' on a regular basis).

For problems with libxml, it helps to install its development headers (libxml2-dev is the package name for a Debian-based Linux).

---

**Disclaimers:**

  1. Please remember that all third-party tools (such as the [SENNA](http://ml.nec-labs.com/senna/) NLP toolkit) enforce their own licensing constraints.

  2. This Github repository is a successor to the now deprecated [C+Perl LLaMaPUn implementation](https://github.com/KWARC/deprecated-LLaMaPUn).
