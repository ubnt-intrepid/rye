# Writing Test Case

Like the built-in test framework, a test case is simply written as a free functions.
The test case can be registered as a test target by applying the attribute-style macro
`#[rye::test]`.

```rust
# fn main() {}
#[rye::test]
fn case1(cx: &mut rye::Context<'_>) {
    // ...
}
```

The type that implements `Termination` can be specified as the output type of the
test function.

```rust
# fn main() {}
fn do_something(counter: &mut i32) -> anyhow::Result<()> {
    // ...
#   *counter += 1;
#   Ok(())
}

#[rye::test]
fn fallible(cx: &mut rye::Context<'_>) -> anyhow::Result<()> {
    let mut counter = 0;

    do_something(&mut counter)?;
    if counter != 1 {
        rye::fail!(cx, "assertion failed: counter is not incremented");
    }

    Ok(())
}
```

```rust,ignore
#[rye::test] //~ ERROR E0277
fn return_int() -> i32 {
    0
}
# fn main() {}
```

## Asynchronous Test Cases

The asynchronous functions could be used in test cases.

```rust
# fn main() {}
#[rye::test]
async fn case_async(cx: &mut rye::Context<'_>) {
    let mut counter = 0usize;

    async {
        counter += 1;
    }
    .await;

    if counter != 1 {
        rye::fail!(cx, "assertion failed: count != 1");
    }
}
```

By default, the future returned from the async functions are assumed to be `Send`
and non-`Send` local variables cannot be captured across the `.await` in the test
case. To annotate that the future is `!Send`, you need to specify the parameter to
the attribute `#[test]` as follows:

```rust
# use std::{cell::Cell, rc::Rc};
# fn main() {}
#[rye::test(?Send)]
async fn case_async_nosend(cx: &mut rye::Context<'_>) {
    let counter = Rc::new(Cell::new(0usize));

    async {
        counter.set(counter.get() + 1);
    }
    .await;

    assert_eq!(counter.get(), 1);
}
```
