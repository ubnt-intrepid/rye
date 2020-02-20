fn main() {}

mod missing_test_cases {
    rye::test_main! {}
}

mod unsupported_patterns {
    rye::test_main! {
        test_cases = {
            a,
            b::c,
            {d, e, f},
            *,
            foo as bar,
        };
    }
}

mod unknown_parameter {
    rye::test_main! {
        foo = bar;
    }
}

mod unresolved_runner_path {
    rye::test_main! {
        test_cases = {};
        runner = the_quick_brown_fox_jumps_over_the_lazy_dog;
    }
}
