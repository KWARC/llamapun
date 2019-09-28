use libxml::readonly::RoNode;
use llamapun::parallel_data::*;
use llamapun::util::test::RESOURCE_DOCUMENTS;
use std::collections::HashMap;

#[test]
fn can_iterate_corpus() {
  let corpus = Corpus::new("tests".to_string());
  let catalog = corpus.catalog_with_parallel_walk(|document| {
    let mut t_catalog = HashMap::new();
    t_catalog.insert(String::from("doc_count"), 1);
    let mut word_count = 0;
    for mut paragraph in document.paragraph_iter() {
      for mut sentence in paragraph.iter() {
        for word in sentence.word_iter() {
          word_count += 1;
          assert!(!word.range.is_empty());
        }
      }
    }
    t_catalog.insert(String::from("word_count"), word_count);
    t_catalog
  });

  let word_count = catalog.get("word_count").unwrap_or(&0);
  let doc_count = catalog.get("doc_count").unwrap_or(&0);
  println!("Words iterated on: {:?}", word_count);
  assert_eq!(
    *doc_count,
    RESOURCE_DOCUMENTS.len() as u64,
    "found {:?} documents to iterate over",
    doc_count
  );
  assert!(
    *word_count > 8400,
    "expected more than 8400 words, found {:?}",
    word_count
  );
}

#[test]
fn can_iterate_xpath() {
  let corpus = Corpus::new("tests".to_string());
  let catalog = corpus.catalog_with_parallel_walk(|document| {
    let mut t_catalog = HashMap::new();
    let mut contacts = 0;
    for _contact in
      document.xpath_selector_iter("//*[contains(@class,'ltx_contact') and (local-name()='span')]")
    {
      contacts += 1;
    }
    t_catalog.insert(String::from("contact_count"), contacts);
    t_catalog
  });
  let contact_count = catalog.get("contact_count").unwrap_or(&0);
  assert_eq!(
    *contact_count, 18,
    "expected 18 contact elements, found {:?}",
    contact_count
  );
  // emails
  let email_catalog = corpus.catalog_with_parallel_walk(|document| {
    let mut t_catalog = HashMap::new();
    let emails : Vec<ItemDNM> = document.xpath_selector_iter("//*[contains(@class,'ltx_contact') and contains(@class,'ltx_role_email') and (local-name()='span')]").collect();
    t_catalog.insert(String::from("email_count"), emails.len() as u64);
    t_catalog
  });
  let email_count = email_catalog.get("email_count").unwrap_or(&0);
  assert_eq!(
    *email_count, 5,
    "expected 5 email elements, found {:?}",
    email_count
  );
}

#[test]
fn can_iterate_custom() {
  let corpus = Corpus::new("tests".to_string());
  let email_filter = |node: &RoNode| {
    node.get_name() == "span"
      && node
        .get_attribute("class")
        .unwrap_or_default()
        .contains("ltx_contact")
  };
  let catalog = corpus.catalog_with_parallel_walk(|document| {
    let mut t_catalog = HashMap::new();
    let mut contacts = 0;
    for _contact in document.filter_iter(&email_filter) {
      contacts += 1;
    }
    t_catalog.insert(String::from("contact_count"), contacts);
    t_catalog
  });
  let contact_count = catalog.get("contact_count").unwrap_or(&0);
  assert_eq!(
    *contact_count, 18,
    "expected 18 contact elements, found {:?}",
    contact_count
  );
}
