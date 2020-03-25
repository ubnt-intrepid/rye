#![feature(custom_test_frameworks)]
#![test_runner(rye_runner_futures::runner)]
#![allow(clippy::len_zero)]

#[rye::test]
fn case_sync(ctx: &mut rye::Context<'_>) {
    let mut vec = vec![0usize; 5];

    require!(ctx, vec.len() == 5);
    require!(ctx, vec.capacity() >= 5);

    section!(ctx, "resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        require!(ctx, vec.len() == 10);
        require!(ctx, vec.capacity() >= 5);
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
}
