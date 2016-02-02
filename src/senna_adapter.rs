//! This module can be used to apply `rustsenna`'s tokenization
//! and tagging to the `DNM`.


extern crate rustsenna;

use rustsenna::senna;
use tokenizer::Tokenizer;
use std::io::{self, Write};
use dnm::*;


/// Settings for the `SennaAdapter`
pub struct SennaSettings {
    /// Don't use the default senna path
    pub senna_path: Option<String>,
    /// do part-of-speech tagging (required for `do_psg`)
    pub do_pos: bool,
    /// do syntactic parsing (requires `dp_pos` to be `true`)
    pub do_psg: bool,
}

impl SennaSettings {
    /// Checks the validity of the settings
    fn check(&mut self) {
        if self.do_psg && !self.do_pos {
            self.do_pos = true;
            writeln!(&mut io::stderr(), "rust-llamapun: senna_adapter: Warning:\
                        PSG requires POS (set do_pos to true)").unwrap();
        }
    }
}

impl Default for SennaSettings {
    fn default() -> SennaSettings {
        SennaSettings {
            senna_path: None,
            do_pos: true,
            do_psg: true,
        }
    }
}


/// Makes `rustsenna` work with the `dnmlib`
pub struct SennaAdapter<'t> {
    senna: senna::Senna,
    tokenizer: Tokenizer<'t>,
    settings: SennaSettings,
}

impl<'t> Default for SennaAdapter<'t> {
    fn default() -> SennaAdapter<'t> {
        SennaAdapter::new(SennaSettings::default())
    }
}


impl<'t> SennaAdapter<'t> {
    /// construct a new `SennaAdapter` with some settings
    pub fn new(mut settings: SennaSettings) -> SennaAdapter<'t> {
        settings.check();
        SennaAdapter {
            senna: senna::Senna::new(if settings.senna_path == None {
                                          rustsenna::sennapath::SENNA_PATH
                                      } else { settings.senna_path.as_ref().unwrap() }),
            tokenizer: Tokenizer::default(),
            settings: settings,
        }
    }

    /// Changes the settings (changes of the `senna_path` have no effect)
    pub fn change_settings(&mut self, settings: SennaSettings) {
        self.settings = settings;
    }

    /// processes a sentence according to the settings
    /// *Important*: The `range` is assumed to represent exactly one sentence
    pub fn process_sentence<'a>(&mut self, range: DNMRange<'a>) -> Sentence<'a> {
        let parseoption = {
            if self.settings.do_psg {
                senna::ParseOption::GeneratePSG
            } else if self.settings.do_pos {
                senna::ParseOption::GeneratePOS
            } else {
                senna::ParseOption::TokenizeOnly
            }
        };

        {
            let senna_sentence = self.senna.parse((&range).get_plaintext(), parseoption);
        }

        Sentence {
            range: range,
            words: vec![],
        }
    }

    /// processes an entire `DNM`
    pub fn process_dnm<'a>(&mut self, dnm: &'a DNM) -> Vec<Sentence<'a>> {
        let sentence_ranges : Vec<DNMRange<'a>> = self.tokenizer.sentences(dnm);
        let mut results : Vec<Sentence<'a>> = Vec::with_capacity(sentence_ranges.len());

        for sentence in sentence_ranges {
            results.push(self.process_sentence(sentence));
        }

        results
    }
}


pub struct Sentence<'t> {
    range: DNMRange<'t>,
    words: Vec<Word<'t>>,
}


pub struct Word<'t> {
    range: DNMRange<'t>,
}



