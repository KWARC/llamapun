//! Expose convenience calls to be used from non-Rust applications
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::util::path_helpers;

/// Interface function for vec2doc-expected word tokenization of a document path
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn word_tokenize_for_vec2doc(value: *const c_char) -> *mut c_char {
  let c_value = CStr::from_ptr(value);
  let tokenized = match c_value.to_str() {
    Ok(value) => path_helpers::path_to_words(value.to_string()),
    _ => String::new(),
  };
  CString::new(tokenized).unwrap().into_raw()
}
