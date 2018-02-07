//! A module for pattern matching in mathematical documents

mod rules;
mod utils;
mod matching;

pub use self::rules::{MarkerEnum, MathMarker, PatternFile, PatternMarker, TextMarker};
pub use self::matching::{match_sentence, Match};
