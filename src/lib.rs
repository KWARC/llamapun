//! # The LLaMaPUn library in Rust
//! This is an attempt to reimplement the LLaMaPUn library in Rust.
//! The original library can be found at https://github.com/KWARC/LLaMaPUn

#![feature(collections)]
extern crate rustlibxml;
extern crate libc;
extern crate regex;
extern crate unidecode;

pub mod dnmlib;
pub mod stopwords;
pub mod tokenizer;
