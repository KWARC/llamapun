//! A module for pattern matching in mathematical documents

mod rules;
mod utils;
mod matching;

pub use self::rules::PatternFile;
pub use self::matching::match_sentence;
