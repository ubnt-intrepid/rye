use std::cell::RefCell;

#[test]
fn no_section() {
    let history = RefCell::new(vec![]);
    rye::test_case(|| {
        history.borrow_mut().push("test");
    });
    assert_eq!(*history.borrow(), vec!["test"]);
}

#[test]
fn one_section() {
    let history = RefCell::new(vec![]);
    rye::test_case(|| {
        history.borrow_mut().push("setup");

        rye::section!("section1", {
            history.borrow_mut().push("section1");
        });

        history.borrow_mut().push("teardown");
    });
    assert_eq!(*history.borrow(), vec!["setup", "section1", "teardown"]);
}

#[test]
fn multi_section() {
    let history = RefCell::new(vec![]);
    rye::test_case(|| {
        history.borrow_mut().push("setup");

        rye::section!("section1", {
            history.borrow_mut().push("section1");
        });

        rye::section!("section2", {
            history.borrow_mut().push("section2");
        });

        history.borrow_mut().push("teardown");
    });
    assert_eq!(
        *history.borrow(),
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
    let history = RefCell::new(vec![]);
    rye::test_case(|| {
        history.borrow_mut().push("setup");

        rye::section!("section1", {
            history.borrow_mut().push("section1:setup");

            rye::section!("section2", {
                history.borrow_mut().push("section2");
            });

            rye::section!("section3", {
                history.borrow_mut().push("section3");
            });

            history.borrow_mut().push("section1:teardown");
        });

        history.borrow_mut().push("test");

        rye::section!("section4", {
            history.borrow_mut().push("section4");
        });

        history.borrow_mut().push("teardown");
    });
    assert_eq!(
        *history.borrow(),
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
    use futures_test::future::FutureTestExt as _;

    futures_executor::block_on(rye::test_case_async(|| async {
        println!("setup");
        async {}.pending_once().await;

        rye::section!("section1", {
            println!("section1:setup");
            async {}.pending_once().await;

            rye::section!("section2", {
                async {}.pending_once().await;
                println!("section2");
            });

            rye::section!("section3", {
                async {}.pending_once().await;
                println!("section3");
            });

            async {}.pending_once().await;
            println!("section1:teardown");
        });

        println!("test");
        async {}.pending_once().await;

        rye::section!("section4", {
            async {}.pending_once().await;
            println!("section4");
        });

        async {}.pending_once().await;
        println!("teardown");
        println!("----------");
    }));
}
