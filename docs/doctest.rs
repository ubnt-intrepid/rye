#![cfg(doctest)]

use doc_comment::doc_comment;

doc_comment!(include_str!("src/section.md"), pub mod section {});
doc_comment!(include_str!("src/test_case.md"), pub mod test_case {});
