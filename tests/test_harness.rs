rye::test_harness! {
    #![test_runner(rye_runner_futures::runner)]
}

#[rye::test]
fn case_sync() {
    let mut vec = vec![0usize; 5];

    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}

#[rye::test]
fn nested() {
    let mut vec = vec![0usize; 5];

    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 10);

        section!("shrinking smaller does not changes capacity", {
            vec.resize(0, 0);

            assert_eq!(vec.len(), 0);
            assert!(vec.capacity() >= 10);
        });
    });
}

#[rye::test]
async fn case_async() {
    let mut vec = vec![0usize; 5];

    assert_eq!(vec.len(), 5);
    assert!(vec.capacity() >= 5);

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}

#[rye::test(?Send)]
async fn case_async_nosend() {
    let mut vec = vec![0usize; 5];
    let _rc = std::rc::Rc::new(());

    (async {
        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);
    })
    .await;

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 5);
    });
}

mod sub {
    #[rye::test]
    fn sub_test() {
        let mut vec = vec![0usize; 5];

        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        section!("resizing bigger changes size and capacity", {
            vec.resize(10, 0);

            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        });
    }

    use rye as catcher_in_the_rye;

    #[rye::test]
    #[rye(crate = catcher_in_the_rye)]
    fn modified_rye_path() {
        let mut vec = vec![0usize; 5];

        assert_eq!(vec.len(), 5);
        assert!(vec.capacity() >= 5);

        section!("resizing bigger changes size and capacity", {
            vec.resize(10, 0);

            assert_eq!(vec.len(), 10);
            assert!(vec.capacity() >= 5);
        });
    }
}

#[rye::test]
fn return_result() -> anyhow::Result<()> {
    macro_rules! require {
        ($e:expr) => {
            anyhow::ensure!(
                $e,
                "assertion failed at {}:{}:{}: {}",
                file!(),
                line!(),
                column!(),
                stringify!($e)
            )
        };
    }

    let mut vec = vec![0usize; 5];

    require!(vec.len() == 5);
    require!(vec.capacity() >= 5);

    section!("resizing bigger changes size and capacity", {
        vec.resize(10, 0);

        require!(vec.len() == 10);
        require!(vec.capacity() >= 10);
    });

    #[cfg(FALSE)]
    section!("with unused variables", {
        let a = 10;
    });

    Ok(())
}

#[rye::test]
fn expensive_test() {
    if std::env::var("RUN_EXPENSIVE_TESTS").is_err() {
        rye::skip!("set RUN_EXPENSIVE_TESTS=true to be enabled");
    }

    // do expensive tests ...
}
