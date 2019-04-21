//! # The `LLaMaPUn` library in Rust
//! Language and Mathematics Processing and Understanding
//! Common data structures and algorithms for semi-structured NLP on math-rich
//! documents.

#![deny(
  missing_docs,
  trivial_casts,
  trivial_numeric_casts,
  unused_import_braces,
  unused_qualifications
)]

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
pub mod ams;
pub mod data;
pub mod dnm;
pub mod ngrams;
pub mod parallel_data;
pub mod patterns;
pub mod stopwords;
pub mod tokenizer;

pub mod extern_use;
