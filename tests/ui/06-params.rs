fn main() {}

#[rye::test_case(rye_path = 10)]
fn invalid_path_type() {}

#[rye::test_case(rye_path = "catcher_in_the_rye")]
fn unresolved_path() {}

#[rye::test_case(rye_path = "rye", rye_path = "rye")]
fn duplicated() {}

#[rye::test_case(the_quick_fox = "lazy")]
fn unknown_param() {}
