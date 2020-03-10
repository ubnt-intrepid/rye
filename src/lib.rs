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

pub mod cli;
pub mod reporter;
pub mod runner;
pub mod test;

mod global;

#[doc(hidden)] // private API.
pub mod _internal {
    pub use crate::test::{
        imp::{Section, TestFn, TestFuture},
        Registry, RegistryError, Test, TestDesc, TestSet,
    };
    pub use lazy_static::lazy_static;
    pub use maplit::hashset;
    pub use std::{module_path, result::Result, vec};

    use crate::{
        runner::{Context, EnterSection},
        test::{imp::SectionId, Fallible},
    };

    #[inline]
    pub fn test_result<T: Fallible>(res: T) -> Box<dyn Fallible> {
        Box::new(res)
    }

    #[inline]
    pub fn enter_section(id: SectionId) -> EnterSection {
        Context::with(|ctx| ctx.enter_section(id))
    }

    #[doc(hidden)] // private API.
    #[macro_export]
    macro_rules! __enter_section {
        ($id:expr, $block:block) => {{
            let section = $crate::_internal::enter_section($id);
            if section.enabled() {
                $block
            }
            section.leave();
        }};
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
    macro_rules! __declare_test_module {
        (
            name = $name:ident;
            sections = { $($sections:tt)* };
            leaf_sections = { $($leaf_sections:tt)* };
            $( [async(local = false)] test_fn = $async_test_fn:ident; )?
            $( [async(local = true)]  test_fn = $async_local_test_fn:ident; )?
            $( [blocking]             test_fn = $blocking_test_fn:ident; )?
        ) => {
            pub(crate) mod $name {
                use super::*;

                $crate::_internal::lazy_static! {
                    static ref DESC: $crate::_internal::TestDesc = $crate::_internal::TestDesc {
                        module_path: $crate::_internal::module_path!(),
                        sections: $crate::__declare_section!($($sections)*),
                        leaf_sections: $crate::_internal::vec![ $($leaf_sections)* ],
                    };
                }

                #[allow(non_camel_case_types)]
                struct __tests(());

                impl $crate::_internal::TestSet for __tests {
                    fn register(&self, __registry: &mut dyn $crate::_internal::Registry) -> $crate::_internal::Result<(), $crate::_internal::RegistryError> {
                        __registry.add_test($crate::_internal::Test {
                            desc: &*DESC,
                            $( test_fn: $crate::_internal::TestFn::Async { f: || $crate::_internal::TestFuture::new($async_test_fn()), local: false }, )?
                            $( test_fn: $crate::_internal::TestFn::Async { f: || $crate::_internal::TestFuture::new_local($async_local_test_fn()), local: true }, )?
                            $( test_fn: $crate::_internal::TestFn::Blocking { f: || $crate::_internal::test_result($blocking_test_fn()) }, )?
                        })?;
                        $crate::_internal::Result::Ok(())
                    }
                }

                $crate::__annotate_test_case! {
                    pub(crate) static __TESTS: &dyn $crate::_internal::TestSet = &__tests(());
                }
            }
        };
    }

    #[doc(hidden)] // private API.
    #[cfg(not(feature = "frameworks"))]
    #[macro_export]
    macro_rules! __annotate_test_case {
        ($item:item) => {
            $item
        };
    }

    #[doc(hidden)] // private API.
    #[cfg(feature = "frameworks")]
    #[macro_export]
    macro_rules! __annotate_test_case {
        ($item:item) => {
            #[test_case]
            $item
        };
    }
}

/// Generate a single test case.
pub use rye_macros::test;

/// Define a set of test cases onto the current module.
///
/// # Example
///
/// ```ignore
/// // tests.rs
///
/// #[rye::test]
/// fn case1() {
///     // ...
/// }
///
/// mod sub1 {
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
///
///     rye::test_set! {
///         case2,
///         case3,
///         sub2,
///     }
/// }
/// ```
///
/// ```ignore
/// // sub2.rs
///
/// #[rye::test]
/// fn case4() {
///     // ...
/// }
///
/// rye::test_set! { case4 }
/// ```
pub use rye_macros::test_set;

/// Generate the main function for running the test cases.
pub use rye_macros::test_harness;
