/*!
A custom unit testing framework inspired by Catch2.

The concept is heavily influenced by the section mechanism in [`Catch2`](https://github.com/catchorg/Catch2),
a C++ unit testing framework library.

# Getting Started

Like the built-in test framework, a test case is simply written as a free functions.
The test case can be registered as a test target by applying the attribute-style macro
`#[rye::test]`.

```
# fn main() {}
# mod inner {
#[rye::test]
fn case1() {
    assert!(1 + 1 == 2);
}
# }
```

WIP

```
# fn main() {}
# mod inner {
# #[rye::test] fn case1() {}
rye::test_group! {
    case1,
}
rye::test_runner!(path::to::runner);
# mod path { pub mod to { pub fn runner(_: &[&dyn rye::registration::Registration]) {} } }
# }
```

# Asynchronous test cases

WIP

```
# fn main() {}
#[rye::test]
async fn case_async() {
    async {
        assert_eq!(1 + 1, 2);
    }.await;
}
```

# Organization of multiple test cases

WIP

```
# fn main() {}
# mod inner {
#[rye::test]
fn case1() {
    // ...
}

mod sub1 {
    #[rye::test]
    fn case2() {
        // ...
    }

    #[rye::test]
    fn case3() {
        // ...
    }

    mod sub2 {
        #[rye::test]
        fn case4() {
            // ...
        }

        rye::test_group! { case4 }
    }

    rye::test_group! {
        case2,
        case3,
        sub2,
    }
}

rye::test_group! {
    case1,
    sub1,
}
rye::test_runner!(path::to::runner);
# mod path { pub mod to { pub fn runner(_: &[&dyn rye::registration::Registration]) {} } }
# }
```

# Section

`rye` supports the scope-based code sharing mechanism inspired by Catch2.
Test cases could distinguish specific code blocks during test execution by
enclosing a particular code block in the test body with `section!()`. Here,
`section!()` is a procedural macro interpreted by `#[test]` and expands to
an `if` statement to toggle the section. For example, the following test case

```
# fn main() {}
#[rye::test]
fn with_section() {
    println!("setup");

    section!("section", {
        println!("section");
    });

    println!("teardown");
}
```

will be roughly expanded as the follows:

```ignore
fn with_section() {
    println!("setup");

    if /* is_section_enabled */ {
        println!("section");
    }

    println!("teardown");
}
```

If there are multiple sections in the same scope, enable them in order and
execute the test case until all sections are completed. Consider the following
test case:

```
# fn main() {}
#[rye::test]
fn has_multi_section() {
    println!("startup");

    section!("section 1", {
        println!("section 1");
    });

    section!("section 2", {
        println!("section 2");
    });

    println!("teardown");
    println!();
}
```

The above test case will produce the following result:

```txt
startup
section 1
teardown

startup
section 2
teardown
```
!*/

pub mod executor;
pub mod registration;

mod test;

#[doc(hidden)]
pub mod _internal {
    pub use crate::{
        registration::{Registration, Registry, RegistryError},
        test::{Section, Test, TestDesc, TestFn},
    };
    pub use maplit::{hashmap, hashset};
    pub use std::{boxed::Box, module_path, result::Result, vec};

    use crate::{executor::TestContext, test::SectionId};

    #[inline]
    pub fn is_target(id: SectionId) -> bool {
        TestContext::with(|ctx| ctx.is_target_section(id))
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! __annotate_test_case {
        ($item:item) => {
            $item
        };
    }
}

pub use crate::test::Test;

/// Generate a single test case.
pub use rye_macros::test;

/// Re-export the registration of test cases.
pub use rye_macros::test_group;

/// Generate the main function for running the test cases.
#[macro_export]
macro_rules! test_runner {
    ($runner:path) => {
        fn main() {
            $runner(&[&self::__REGISTRATION as &dyn $crate::_internal::Registration]);
        }
    };
}
