//! A module for pattern matching in mathematical documents

mod matching;
mod rules;
mod utils;

pub use self::matching::{match_sentence, Match};
pub use self::rules::{MarkerEnum, MathMarker, PatternFile, PatternMarker, TextMarker};
