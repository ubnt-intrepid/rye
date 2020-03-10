fn main() {}

#[rye::test]
#[rye(crate = 10)]
fn invalid_path_type() {}

#[rye::test]
#[rye(crate = "rye")]
#[rye(crate = "rye")]
fn duplicated() {}

#[rye::test]
#[rye(the_quick_fox = "lazy")]
fn unknown_param() {}
