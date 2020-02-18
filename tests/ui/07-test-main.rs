fn main() {}

mod unexpected_path_patterns {
    rye::test_main! {
        a,
        b::c,
        {d, e, f},
        *,
        foo as bar,
    }
}

mod unresolved_test_path {
    rye::test_main! {
        a,
        b::c,
        {d, e},
        f::g::{h, i},
    }
}
