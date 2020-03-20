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

The type that implements `TestResult` can be specified as the output type of the
test function. Currently, the implementors of this trait are only `()` and
`Result<(), E: Debug>`.

```
#[rye::test]
fn fallible() -> std::io::Result<()> {
    Ok(())
}
# fn main() {}
```

```compile_fail
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

# Generating Test Harness

On the current stable compiler, test cases annotated by `#[rye::test]` attribute are not
implicitly registered for the execution.
Therefore, the test applications must explicitly specify the test cases to be executed
and call the test runner to running them by disabling the default test harness.

```toml
[[test]]
name = "tests"
harness = false
```

```
# fn main() {}
# mod inner {
#[rye::test]
fn case1() {
    // ...
}

rye::test_harness! {
    #![test_runner(path::to::runner)]
    #![test_cases(case1)]
}
# mod path { pub mod to { pub fn runner(_: &[&dyn rye::test::TestSet]) {} } }
# }
```

## (Advanced) Using `custom_test_frameworks` Feature

If you are a nightly pioneer, the unstable feature `custom_test_frameworks` can be used
to automate the registration of test cases.

```toml
[dev-dependencies]
rye = { ..., features = [ "frameworks" ] }
```

```ignore
#![feature(custom_test_frameworks)]
#![test_runner(path::to::runner)]

#[rye::test]
fn case1() { ... }

mod sub {
    #[rye::test]
    fn case2() { ... }
}
```

!*/

#![doc(html_root_url = "https://docs.rs/rye/0.1.0-dev")]
#![deny(missing_docs)]
#![forbid(clippy::unimplemented, clippy::todo)]

pub mod executor;
pub mod reporter;
pub mod test;

mod args;
mod exit_status;
mod global;
mod session;

pub use crate::{args::Args, exit_status::ExitStatus, session::Session};

#[allow(missing_docs)]
pub fn install() {
    crate::global::install();
}

/// Generate a single test case.
pub use rye_macros::test;

/// Generate the main function for running the test cases.
pub use rye_macros::test_harness;

/// Define a set of test cases onto the current module.
///
/// # Example
///
/// ```ignore
/// rye::test_harness! {
///     #![test_runner(path::to::runner)]
///     #![test_cases(case1, sub1)]
/// }
///
/// #[rye::test]
/// fn case1() {
///     // ...
/// }
///
/// mod sub1 {
///     rye::test_module! {
///         #![test_cases(case2, case3, sub2)]
///     }
///
///     #[rye::test]
///     fn case2() {
///         // ...
///     }
///
///     #[rye::test]
///     fn case3() {
///         // ...
///     }
///
///     #[path = "sub2.rs"]
///     mod sub2;
/// }
/// ```
///
/// ```ignore
/// // sub2.rs
///
/// rye::test_module! {
///     #![test_cases(case4)]
/// }
///
/// #[rye::test]
/// fn case4() {
///     // ...
/// }
/// ```
pub use rye_macros::test_module;

/// Mark the current test case as having been skipped and terminate its execution.
#[macro_export]
macro_rules! skip {
    () => ( $crate::skip!("explicitly skipped") );
    ($($arg:tt)+) => {
        $crate::_internal::skip(format_args!($($arg)+))
    };
}

#[doc(hidden)] // private API.
pub mod _internal {
    pub use crate::{
        __async_local_test_fn as async_local_test_fn, //
        __async_test_fn as async_test_fn,
        __blocking_test_fn as blocking_test_fn,
        __cfg_frameworks as cfg_frameworks,
        __declare_section as declare_section,
        __enter_section as enter_section,
        __location as location,
        __test_name as test_name,
        test::{
            imp::{Section, TestFn, TestFuture},
            Registry, RegistryError, TestDesc, TestSet,
        },
    };
    pub use maplit::hashset;
    pub use std::{module_path, result::Result, stringify};

    use crate::{
        executor::{Context, EnterSection},
        test::{imp::SectionId, Fallible},
    };
    use std::{borrow::Cow, fmt};

    #[inline]
    pub fn test_result<T: Fallible + 'static>(res: T) -> Box<dyn Fallible + 'static> {
        Box::new(res)
    }

    #[inline]
    pub fn enter_section(id: SectionId) -> EnterSection {
        Context::with(|ctx| ctx.enter_section(id))
    }

    #[inline]
    pub fn skip(reason: fmt::Arguments<'_>) -> ! {
        Context::with(|ctx| ctx.mark_skipped(reason));
        panic!("skipped")
    }

    #[inline]
    pub fn test_name(module_path: &'static str, name: &'static str) -> Cow<'static, str> {
        module_path
            .splitn(2, "::")
            .nth(1)
            .map_or(name.into(), |m| format!("{}::{}", m, name).into())
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __test_name {
        ($name:ident) => {
            $crate::_internal::test_name(
                $crate::_internal::module_path!(),
                $crate::_internal::stringify!($name),
            )
        };
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __enter_section {
        ( $id:expr, $(#[$attr:meta])* $block:block ) => {
            $(#[$attr])*
            {
                let section = $crate::_internal::enter_section($id);
                if section.enabled() {
                    $block
                }
                section.leave();
            }
        };
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __declare_section {
        (@single $($x:tt)*) => (());
        (@count $($rest:expr),*) => {
            <[()]>::len(&[$($crate::__declare_section!(@single $rest)),*])
        };

        ($( $key:expr => ($name:expr, { $($ancestors:tt)* }); )*) => {
            {
                let _cap = $crate::__declare_section!(@count $($key),*);
                #[allow(clippy::let_and_return)]
                let mut _map = ::std::collections::HashMap::with_capacity(_cap);
                $(
                    let _ = _map.insert($key, $crate::_internal::Section {
                        name: $name,
                        ancestors: $crate::_internal::hashset!($($ancestors)*),
                    });
                )*
                _map
            }
        };
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __async_local_test_fn {
        ($path:path) => {
            $crate::_internal::TestFn::Async {
                f: || $crate::_internal::TestFuture::new_local($path()),
                local: true,
            }
        };
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __async_test_fn {
        ($path:path) => {
            $crate::_internal::TestFn::Async {
                f: || $crate::_internal::TestFuture::new($path()),
                local: false,
            }
        };
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __blocking_test_fn {
        ($path:path) => {
            $crate::_internal::TestFn::Blocking {
                f: || $crate::_internal::test_result($path()),
            }
        };
    }

    #[doc(hidden)] // private API.
    #[cfg(not(feature = "frameworks"))]
    #[macro_export]
    macro_rules! __cfg_frameworks {
        ($($t:tt)*) => {};
    }

    #[doc(hidden)] // private API.
    #[cfg(feature = "frameworks")]
    #[macro_export]
    macro_rules! __cfg_frameworks {
        ($($t:tt)*) => ( $($t)* );
    }
}
