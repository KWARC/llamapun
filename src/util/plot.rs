extern crate gnuplot;
use gnuplot::*;
use std::hash::Hash;

pub fn plot_simple<T : Clone+Eq+Hash+DataType>(
  map : &Vec<(T,usize)>, x_label: &str, y_label: &str, title: &str, pathname : &str) {

  let keys : Vec<T> = map.clone().into_iter().map(|entry| entry.0).collect();
  let log_values = map.clone().into_iter().map(|entry| (entry.1.clone() as f64).log2());
  let mut fg = Figure::new();
  fg.axes2d()
  .points(keys, log_values, &[PointSymbol('O'), Color("#ffaa77"), PointSize(1.2)])
  .set_x_label(x_label, &[Rotate(45.0)])
  .set_y_label(y_label, &[Rotate(90.0)])
  .set_title(&(title), &[]);

  fg.set_terminal("pngcairo", pathname);
  fg.show();
}