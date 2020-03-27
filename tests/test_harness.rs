#![allow(clippy::len_zero)]

rye::test_harness!();

macro_rules! require {
    ($ctx:ident, $e:expr) => {{
        if !($e) {
            rye::fail!($ctx, concat!("assertion failed: ", stringify!($e)));
        }
    }};
}

#[rye::test]
fn case_sync(ctx: &mut rye::Context<'_>) {
    let mut vec = vec![0usize; 5];

    require!(ctx, vec.len() == 5);
    require!(ctx, vec.capacity() >= 5);

    section!(ctx, "resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        require!(ctx, vec.len() == 10);
        require!(ctx, vec.capacity() >= 10);
    });
}

#[rye::test]
fn nested(ctx: &mut rye::Context<'_>) {
    let mut vec = vec![0usize; 5];

    require!(ctx, vec.len() == 5);
    require!(ctx, vec.capacity() >= 5);

    section!(ctx, "resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        require!(ctx, vec.len() == 10);
        require!(ctx, vec.capacity() >= 10);

        section!(ctx, "shrinking smaller does not changes capacity", {
            vec.resize(0, 0);

            require!(ctx, vec.len() == 0);
            require!(ctx, vec.capacity() >= 10);
        });
    });
}

#[rye::test]
async fn case_async(ctx: &mut rye::Context<'_>) {
    let mut vec = vec![0usize; 5];

    require!(ctx, vec.len() == 5);
    require!(ctx, vec.capacity() >= 5);

    section!(ctx, "resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        require!(ctx, vec.len() == 10);
        require!(ctx, vec.capacity() >= 10);
    });
}

#[rye::test(?Send)]
async fn case_async_nosend(ctx: &mut rye::Context<'_>) {
    let mut vec = vec![0usize; 5];
    let _rc = std::rc::Rc::new(());

    require!(ctx, vec.len() == 5);
    require!(ctx, vec.capacity() >= 5);

    section!(ctx, "resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        require!(ctx, vec.len() == 10);
        require!(ctx, vec.capacity() >= 5);
    });
}

mod sub {
    #[rye::test]
    fn sub_test(ctx: &mut rye::Context<'_>) {
        let mut vec = vec![0usize; 5];

        require!(ctx, vec.len() == 5);
        require!(ctx, vec.capacity() >= 5);

        section!(ctx, "resizing bigger changes size and capacity", {
            vec.resize(10, 0);

            require!(ctx, vec.len() == 10);
            require!(ctx, vec.capacity() >= 5);
        });
    }

    use rye as catcher_in_the_rye;

    #[rye::test]
    #[rye(crate = catcher_in_the_rye)]
    fn modified_rye_path(ctx: &mut rye::Context<'_>) {
        let mut vec = vec![0usize; 5];

        require!(ctx, vec.len() == 5);
        require!(ctx, vec.capacity() >= 5);

        section!(ctx, "resizing bigger changes size and capacity", {
            vec.resize(10, 0);

            require!(ctx, vec.len() == 10);
            require!(ctx, vec.capacity() >= 10);
        });
    }
}

#[rye::test]
fn return_result(ctx: &mut rye::Context<'_>) -> anyhow::Result<()> {
    let mut vec = vec![0usize; 5];

    require!(ctx, vec.len() == 5);
    require!(ctx, vec.capacity() >= 5);

    anyhow::ensure!(!vec.is_empty(), "vec is empty");

    section!(ctx, "resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        require!(ctx, vec.len() == 10);
        require!(ctx, vec.capacity() >= 10);
    });

    #[cfg(FALSE)]
    section!(ctx, "with unused variables", {
        let a = 10;
    });

    Ok(())
}

#[rye::test]
fn expensive_test(ctx: &mut rye::Context<'_>) {
    if std::env::var("RUN_EXPENSIVE_TESTS").is_err() {
        rye::skip!(ctx, "set RUN_EXPENSIVE_TESTS=true to be enabled");
    }

    // do expensive tests ...
}

#[rye::test]
fn expensive_test_fallible(ctx: &mut rye::Context<'_>) -> anyhow::Result<()> {
    if std::env::var("RUN_EXPENSIVE_TESTS").is_err() {
        rye::skip!(ctx, "set RUN_EXPENSIVE_TESTS=true to be enabled");
    }

    // do expensive tests ...

    Ok(())
}
