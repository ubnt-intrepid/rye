fn main() {}

#[rye::test(rye_path = 10)]
fn invalid_path_type() {}

#[rye::test(rye_path = "catcher_in_the_rye")]
fn unresolved_path() {}

#[rye::test(rye_path = "rye", rye_path = "rye")]
fn duplicated() {}

#[rye::test(the_quick_fox = "lazy")]
fn unknown_param() {}
