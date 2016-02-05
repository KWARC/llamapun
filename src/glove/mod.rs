mod cooccur;
mod vocab_count;

use data::Corpus;

pub struct Glove {
  tokens : usize
}

impl Glove {
  pub fn train(corpus : Corpus) -> Glove {
    let mut model = Glove {tokens : 0};
    model
  }
}