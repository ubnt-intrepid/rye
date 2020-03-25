fn main() {}

#[rye::test]
#[rye(crate = 10)]
fn invalid_path_type(_: &mut rye::Context<'_>) {}

#[rye::test]
#[rye(the_quick_fox = "lazy")]
fn unknown_param(_: &mut rye::Context<'_>) {}
