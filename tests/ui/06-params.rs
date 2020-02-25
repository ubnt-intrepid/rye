fn main() {}

#[rye::test]
#[rye(rye_path = 10)]
fn invalid_path_type() {}

#[rye::test]
#[rye(rye_path = "rye")]
#[rye(rye_path = "rye")]
fn duplicated() {}

#[rye::test]
#[rye(the_quick_fox = "lazy")]
fn unknown_param() {}
