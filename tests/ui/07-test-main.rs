fn main() {}

fn dummy(_: &[&dyn rye::Registration]) {}

mod missing_test_cases {
    rye::test_main! {
        runner = crate::dummy;
    }
}

mod missing_runner {
    rye::test_main! {
        test_cases = {};
    }
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
        runner = crate::dummy;
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
