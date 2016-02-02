The **llamapun** library hosts common _language and mathematics processing_ algorithms, used by the KWARC research group.

[![Build Status](https://secure.travis-ci.org/KWARC/llamapun.png?branch=master)](http://travis-ci.org/KWARC/llamapun)

---
At its core, **lamapun** is a [Rust](http://rust-lang.org/) implementation that aims at minimal footprint and optimal runtime, in order to safely scale to corpora of millions of documents and tens of billions ot tokens.

## High-level Overview
 * **Source Data**
   * Built-in support for STEM documents in ([LaTeXML-flavoured](https://github.com/brucemiller/LaTeXML/)) HTML5.
 * **Preprocessing**
   * Unicode normalization,
   * Stopwords - based on widely accepted lists, enhanced for STEM texts,
   * Semi-structured to plain text normalization (math, citations, tables, etc.),
   * Purification of text and math modality (e.g. move trailing dots left in math back into the sentence text),
   * Stemming - adaptation of the [Morpha](http://www.sussex.ac.uk/Users/johnca/morph.html) stemmer,
   * Tokenization - rule-based sentence segmentation, and [SENNA](http://ml.nec-labs.com/senna/) word tokenization
 
 * **Shallow Analysis**
   * Language identification (via [libTextCat](http://software.wise-guys.nl/libtextcat/)),
   * N-gram footprints,
   * Part-of-speech tagging (via [SENNA](http://ml.nec-labs.com/senna/)),
   * Named Entity recognition (via [SENNA](http://ml.nec-labs.com/senna/)),
   * Chunking and shallow parsing (via [SENNA](http://ml.nec-labs.com/senna/)),
   * [TODO] "Definition" paragraph discrimination task (training SVM classifiers, based on TF/IDF and Ngram BoW features, via [libsvm](https://github.com/cjlin1/libsvm))
   * [TODO] "Declaration" sentence discrimination task (training CRF models via [CRFsuite](http://www.chokkan.org/software/crfsuite/)).
 
 * **Representation Toolkit**
   * Document Narrative Model (DNM) addition to the XML DOM
   * XPointer and string offset annotation support
   * Integration with the [CorTeX](https://github.com/dginev/CorTeX) processing framework
   * [TOPORT] Shared Packed parse forests for mathematical formulas (aka "disjunctive logical forms")

 * **Programming API**
   * High-level iterators over the narrative elements of scientific documents
   * Zero-cost abstractions over the source data, as well as over linguistic annotations of various granularity.

 
## Contact
Feel free to send any feedback to the project maintainer at d.ginev@jacobs-university.de

---

Please remember that all third-party tools (such as the [SENNA](http://ml.nec-labs.com/senna/) NLP toolkit) enforce their own licensing constraints.
**Disclaimer:** This Github repository is a successor to the now deprecated [C+Perl LLaMaPUn implementation](https://github.com/KWARC/deprecated-LLaMaPUn).
