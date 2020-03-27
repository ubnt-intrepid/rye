/*!
A custom unit testing framework for Rust.

The concept is heavily influenced by the section mechanism in [`Catch2`](https://github.com/catchorg/Catch2),
a C++ unit testing framework library.

# Writing Test Cases

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

The type that implements `Termination` can be specified as the output type of the
test function.

```
#[rye::test]
fn fallible() -> std::io::Result<()> {
    Ok(())
}
# fn main() {}
```

```ignore
#[rye::test] //~ ERROR E0277
fn return_int() -> i32 {
    0
}
# fn main() {}
```

## Asynchronous Test Cases

The asynchronous functions could be used in test cases.

```
# fn main() {}
#[rye::test]
async fn case_async() {
    let mut counter = 0usize;

    async {
        counter += 1;
    }
    .await;

    assert_eq!(counter, 1);
}
```

By default, the future returned from the async functions are assumed to be `Send`
and non-`Send` local variables cannot be captured across the `.await` in the test
case. To annotate that the future is `!Send`, you need to specify the parameter to
the attribute `#[test]` as follows:

```
# use std::{cell::Cell, rc::Rc};
# fn main() {}
#[rye::test(?Send)]
async fn case_async_nosend() {
    let counter = Rc::new(Cell::new(0usize));

    async {
        counter.set(counter.get() + 1);
    }
    .await;

    assert_eq!(counter.get(), 1);
}
```

## Section

`rye` supports the scope-based code sharing mechanism inspired by Catch2.
Test cases could distinguish specific code blocks during test execution by
enclosing a particular code block in the test body with `section!()`.
Here, `section!()` is an expression-style procedural macro expanded by `#[test]`
and has the following syntax:

```ignore
$( #[ $META:meta ] )*
section!( $NAME:expr , $BODY:block );
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

#![doc(html_root_url = "https://docs.rs/rye/0.1.0-dev")]
#![deny(missing_docs)]
#![forbid(clippy::unimplemented, clippy::todo)]

pub mod executor;

mod context;
mod location;
mod report;
mod runner;
mod termination;
mod test;

pub use crate::{context::Context, runner::TestRunner, termination::Termination};
pub use rye_macros::{test, test_main};

/// Generate test harness.
#[macro_export]
macro_rules! test_harness {
    ( $(block_on = $block_on:path)? ) => {
        #[$crate::test_main]
        #[rye(crate = $crate)]
        $( #[rye(block_on = $block_on)] )?
        async fn main(runner: &mut $crate::TestRunner<'_>) -> impl $crate::Termination {
            runner.run().await
        }
    };
}

#[doc(hidden)] // private API.
pub mod _test_reexports {
    pub use crate::{
        __fail as fail, //
        __location as location,
        __register_test_case as register_test_case,
        __require as require,
        __section as section,
        __skip as skip,
        __test_fn as test_fn,
        __test_name as test_name,
        context::{Context, Section},
        location::Location,
        termination::Termination,
        test::{TestCase, TestDesc, TestFn, TestName, TestPlan, TEST_CASES},
    };
    pub use linkme::{self, distributed_slice};
    pub use paste::item as paste_item;
    pub use std::{
        boxed::Box, column, concat, file, format_args, line, module_path, result::Result, stringify,
    };
}

#[doc(hidden)] // private API.
pub mod _test_main_reexports {
    pub use crate::{
        executor::block_on as default_block_on, //
        runner::TestRunner,
        termination::exit,
    };
}
