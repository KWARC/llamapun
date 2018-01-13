//! # The `LLaMaPUn` library in Rust
//! Language and Mathematics Processing and Understanding
//! Common data structures and algorithms for semi-structured NLP on math-rich documents.

#![feature(slice_patterns)]
#![feature(type_ascription)]
#![deny(missing_docs, trivial_casts, trivial_numeric_casts, unused_import_braces,
        unused_qualifications)]

extern crate crypto;
extern crate gnuplot;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate libxml;
extern crate regex;
extern crate rustmorpha;
extern crate senna;
extern crate unidecode;
extern crate walkdir;

#[macro_use]
pub mod util;
pub mod dnm;
pub mod data;
pub mod stopwords;
pub mod tokenizer;
pub mod ngrams;
