//! Test utilities for llamapun's crate
use lazy_static::lazy_static;
use walkdir::WalkDir;

lazy_static! { // preload a list of the resources we have for testing, for easy corpus sanity checks
  ///  shorthand global for all usable documents in the tests/resources mini-corpus
  pub static ref RESOURCE_DOCUMENTS: Vec<String> = WalkDir::new("./tests/resources")
    .into_iter()
    .filter_entry(|entry| {
      if entry.file_type().is_dir() {
        true
      } else if let Some(name_os) = entry.path().extension() {
        let name = name_os.to_str();
        name == Some("html") || name == Some("xhtml")
      } else {
        false
      }
    })
    .filter_map(|e| e.ok())
    .filter(|e| !e.file_type().is_dir())
    .map(|entry| entry
      .path()
      .file_stem()
      .unwrap()
      .to_str()
      .unwrap()
      .to_string())
    .collect();
}
