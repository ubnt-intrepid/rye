fn main() {}

#[rye::test]
fn has_type_param<T>() {}

#[rye::test]
fn has_const_param<const N: usize>() {}
