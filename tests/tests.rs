use scoped_tls::scoped_thread_local;
use std::cell::RefCell;

scoped_thread_local!(static HISTORY: RefCell<Vec<&'static str>>);

fn append_history(v: &'static str) {
    HISTORY.with(|history| history.borrow_mut().push(v));
}

#[test]
fn no_section() {
    #[rye::test_case]
    fn test_case() {
        append_history("test");
    }

    let mut history = RefCell::new(vec![]);
    HISTORY.set(&history, test_case);

    assert_eq!(*history.get_mut(), vec!["test"]);
}

#[test]
fn one_section() {
    #[rye::test_case]
    fn test_case() {
        append_history("setup");

        section!("section1", {
            append_history("section1");
        });

        append_history("teardown");
    }

    let mut history = RefCell::new(vec![]);
    HISTORY.set(&history, test_case);

    assert_eq!(*history.get_mut(), vec!["setup", "section1", "teardown"]);
}

#[test]
fn multi_section() {
    #[rye::test_case]
    fn test_case() {
        HISTORY.with(|history| history.borrow_mut().push("setup"));

        section!("section1", {
            append_history("section1");
        });

        section!("section2", {
            append_history("section2");
        });

        append_history("teardown");
    }

    let mut history = RefCell::new(vec![]);
    HISTORY.set(&history, test_case);

    assert_eq!(
        *history.get_mut(),
        vec![
            // phase 1
            "setup", "section1", "teardown", //
            // phase 2
            "setup", "section2", "teardown",
        ]
    );
}

#[test]
fn nested_section() {
    #[rye::test_case]
    fn test_case() {
        append_history("setup");

        section!("section1", {
            append_history("section1:setup");

            section!("section2", {
                append_history("section2");
            });

            section!("section3", {
                append_history("section3");
            });

            append_history("section1:teardown");
        });

        append_history("test");

        section!("section4", {
            append_history("section4");
        });

        append_history("teardown");
    }

    let mut history = RefCell::new(vec![]);
    HISTORY.set(&history, test_case);

    assert_eq!(
        *history.get_mut(),
        vec![
            // phase 1
            "setup",
            "section1:setup",
            "section2",
            "section1:teardown",
            "test",
            "teardown",
            // phase 2
            "setup",
            "section1:setup",
            "section3",
            "section1:teardown",
            "test",
            "teardown",
            // phase 3
            "setup",
            "test",
            "section4",
            "teardown",
        ]
    );
}

#[cfg(feature = "futures")]
#[test]
fn smoke_async() {
    use futures_core::{
        future::Future,
        task::{self, Poll},
    };
    use scoped_tls::ScopedKey;
    use std::pin::Pin;

    trait LocalKeyAsyncExt<T> {
        fn set_async<'a, Fut>(&'static self, t: &'a T, fut: Fut) -> SetAsync<'a, T, Fut>
        where
            T: 'static,
            Fut: Future;
    }

    impl<T> LocalKeyAsyncExt<T> for ScopedKey<T> {
        fn set_async<'a, Fut>(&'static self, t: &'a T, fut: Fut) -> SetAsync<'a, T, Fut>
        where
            T: 'static,
            Fut: Future,
        {
            SetAsync { key: self, t, fut }
        }
    }

    #[pin_project::pin_project]
    struct SetAsync<'a, T: 'static, Fut> {
        key: &'static ScopedKey<T>,
        t: &'a T,
        #[pin]
        fut: Fut,
    }

    impl<T, Fut> Future for SetAsync<'_, T, Fut>
    where
        Fut: Future,
    {
        type Output = Fut::Output;

        fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
            let me = self.project();
            let key = me.key;
            let t = *me.t;
            let fut = me.fut;
            key.set(t, || fut.poll(cx))
        }
    }

    #[rye::test_case]
    async fn test_case() {
        use futures_test::future::FutureTestExt as _;

        append_history("setup");
        async {}.pending_once().await;

        section!("section1", {
            append_history("section1:setup");
            async {}.pending_once().await;

            section!("section2", {
                async {}.pending_once().await;
                append_history("section2");
            });

            section!("section3", {
                async {}.pending_once().await;
                append_history("section3");
            });

            async {}.pending_once().await;
            append_history("section1:teardown");
        });

        append_history("test");
        async {}.pending_once().await;

        section!("section4", {
            async {}.pending_once().await;
            append_history("section4");
        });

        async {}.pending_once().await;
        append_history("teardown");
    }

    let mut history = RefCell::new(vec![]);
    futures_executor::block_on(HISTORY.set_async(&history, test_case()));

    assert_eq!(
        *history.get_mut(),
        vec![
            // phase 1
            "setup",
            "section1:setup",
            "section2",
            "section1:teardown",
            "test",
            "teardown",
            // phase 2
            "setup",
            "section1:setup",
            "section3",
            "section1:teardown",
            "test",
            "teardown",
            // phase 3
            "setup",
            "test",
            "section4",
            "teardown",
        ]
    );
}
