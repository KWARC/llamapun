mod vocab_count;
mod cooccur;
mod shuffle;

use data::Corpus;

pub struct Glove {
  tokens : usize
}

impl Glove {
  pub fn train(corpus : Corpus) -> Glove {
    let model = Glove {tokens : 0};
    model
  }
}