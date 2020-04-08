#![cfg(doctest)]

use doc_comment::doc_comment;

doc_comment!(include_str!("src/section.md"));
doc_comment!(include_str!("src/test_case.md"));
